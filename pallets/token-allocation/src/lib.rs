// This file is part of Allfeat.

// Copyright (C) 2022-2025 Allfeat.
// SPDX-License-Identifier: GPL-3.0-or-later

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

mod mock;
mod types;
mod weights;

use types::{
    AccountIdOf, AllocationStatus, BalanceOf, EnvelopeConfig, EnvelopeType, EnvelopeWallet,
    TokenAllocation,
};
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

mod benchmarking;

extern crate alloc;

use frame_support::{
    StorageHasher,
    pallet_prelude::*,
    sp_runtime::{
        Saturating,
        traits::{TrailingZeroInput, Zero},
    },
    traits::fungible::{Inspect, Mutate},
};
use frame_system::{pallet_prelude::BlockNumberFor, pallet_prelude::*};
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::sp_runtime::Percent;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Inspect<Self::AccountId> + Mutate<Self::AccountId>;

        type AllocationOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type EnvelopeWallets<T: Config> =
        StorageMap<_, Twox64Concat, EnvelopeType, EnvelopeWallet<BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    pub type Allocations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AccountIdOf<T>,
        Twox64Concat,
        u32,
        TokenAllocation<BalanceOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type NextAllocationId<T: Config> =
        StorageMap<_, Blake2_128Concat, AccountIdOf<T>, u32, ValueQuery>;

    #[pallet::storage]
    pub type EnvelopeConfigs<T: Config> =
        StorageMap<_, Twox64Concat, EnvelopeType, EnvelopeConfig<BlockNumberFor<T>>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AllocationCreated {
            envelope_type: EnvelopeType,
            beneficiary: AccountIdOf<T>,
            allocation_id: u32,
            amount: BalanceOf<T>,
        },
        TokensClaimed {
            beneficiary: AccountIdOf<T>,
            allocation_id: u32,
            amount: BalanceOf<T>,
        },
        EnvelopeWalletCreated {
            envelope_type: EnvelopeType,
            wallet_account: AccountIdOf<T>,
            total_allocation: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EnvelopeNotFound,
        AllocationNotFound,
        InsufficientEnvelopeBalance,
        NothingToClaim,
        UnauthorizedEnvelopeAccess,
        InvalidParameters,
    }

    /// Type alias for envelope wallet genesis tuple (envelope_type, initial_balance, config)
    pub type EnvelopeWalletGenesis<T> = (
        EnvelopeType,
        BalanceOf<T>,
        EnvelopeConfig<BlockNumberFor<T>>,
    );

    /// Type alias for allocation genesis tuple
    pub type AllocationGenesis<T> = (
        AccountIdOf<T>,
        TokenAllocation<BalanceOf<T>, BlockNumberFor<T>>,
    );

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub envelope_wallets: Vec<EnvelopeWalletGenesis<T>>,
        pub allocations: Vec<AllocationGenesis<T>>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                envelope_wallets: Default::default(),
                allocations: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Initialize envelope wallets and configs
            for (envelope_type, initial_balance, config) in &self.envelope_wallets {
                let envelope_wallet = EnvelopeWallet {
                    distributed_amount: Zero::zero(),
                };

                // Generate the envelope account automatically
                let envelope_account = Pallet::<T>::envelope_account_id(envelope_type);

                EnvelopeWallets::<T>::insert(envelope_type, &envelope_wallet);
                EnvelopeConfigs::<T>::insert(envelope_type, config);

                Pallet::<T>::deposit_event(Event::EnvelopeWalletCreated {
                    envelope_type: envelope_type.clone(),
                    wallet_account: envelope_account,
                    total_allocation: *initial_balance,
                });
            }

            // Initialize pre-allocated tokens (for TGE investors)
            for (beneficiary, allocation) in &self.allocations {
                let allocation_id = NextAllocationId::<T>::get(beneficiary);
                Allocations::<T>::insert(beneficiary, allocation_id, allocation);
                NextAllocationId::<T>::mutate(beneficiary, |id| *id += 1);

                Pallet::<T>::deposit_event(Event::AllocationCreated {
                    envelope_type: allocation.envelope_type.clone(),
                    beneficiary: beneficiary.clone(),
                    allocation_id,
                    amount: allocation.total_allocation,
                });
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::allocate_from_envelope())]
        pub fn allocate_from_envelope(
            origin: OriginFor<T>,
            envelope_type: EnvelopeType,
            beneficiary: AccountIdOf<T>,
            amount: BalanceOf<T>,
            start_block: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            T::AllocationOrigin::ensure_origin(origin)?;

            EnvelopeWallets::<T>::try_mutate(&envelope_type, |maybe_wallet| {
                let wallet = maybe_wallet.as_mut().ok_or(Error::<T>::EnvelopeNotFound)?;

                // Use the automatically generated account
                let envelope_account = Self::envelope_account_id(&envelope_type);
                let current_balance = T::Currency::balance(&envelope_account);

                // Check if we have enough tokens considering what's already distributed
                let available_balance = current_balance.saturating_sub(wallet.distributed_amount);
                ensure!(
                    available_balance >= amount,
                    Error::<T>::InsufficientEnvelopeBalance
                );

                wallet.distributed_amount = wallet.distributed_amount.saturating_add(amount);

                let allocation_id = NextAllocationId::<T>::get(&beneficiary);

                let current_block = <frame_system::Pallet<T>>::block_number();
                let activation_block = start_block.unwrap_or(BlockNumberFor::<T>::zero());

                let status = if activation_block <= current_block {
                    AllocationStatus::ActiveSinceGenesis
                } else {
                    AllocationStatus::ActivatedAt(activation_block)
                };

                let allocation = TokenAllocation {
                    total_allocation: amount,
                    envelope_type: envelope_type.clone(),
                    status,
                    claimed_amount: Zero::zero(),
                };

                // Don't transfer immediately - tokens stay in envelope for vesting/claiming

                Allocations::<T>::insert(&beneficiary, allocation_id, &allocation);
                NextAllocationId::<T>::mutate(&beneficiary, |id| *id += 1);

                Self::deposit_event(Event::AllocationCreated {
                    envelope_type: envelope_type.clone(),
                    beneficiary,
                    allocation_id,
                    amount,
                });

                Ok(())
            })
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::claim_tokens())]
        pub fn claim_tokens(origin: OriginFor<T>, allocation_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Allocations::<T>::try_mutate(&who, allocation_id, |maybe_allocation| {
                let allocation = maybe_allocation
                    .as_mut()
                    .ok_or(Error::<T>::AllocationNotFound)?;

                let current_block = <frame_system::Pallet<T>>::block_number();
                let unlocked_amount = Self::calculate_unlocked_amount(allocation, current_block);
                let claimable_amount = unlocked_amount.saturating_sub(allocation.claimed_amount);

                ensure!(!claimable_amount.is_zero(), Error::<T>::NothingToClaim);

                // Transfer tokens from envelope to beneficiary
                let envelope_account = Self::envelope_account_id(&allocation.envelope_type);
                T::Currency::transfer(
                    &envelope_account,
                    &who,
                    claimable_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;

                allocation.claimed_amount = unlocked_amount;

                if allocation.claimed_amount >= allocation.total_allocation {
                    allocation.status = AllocationStatus::Completed;
                }

                Self::deposit_event(Event::TokensClaimed {
                    beneficiary: who.clone(),
                    allocation_id,
                    amount: claimable_amount,
                });

                Ok(())
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// Generates the deterministic AccountId for an envelope type
        pub fn envelope_account_id(envelope_type: &EnvelopeType) -> AccountIdOf<T> {
            let entropy =
                (b"allfeat/envelope", envelope_type).using_encoded(Blake2_128Concat::hash);
            Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
                .expect("infinite length input; no invalid input for type; qed")
        }

        pub fn calculate_unlocked_amount(
            allocation: &TokenAllocation<BalanceOf<T>, BlockNumberFor<T>>,
            current_block: BlockNumberFor<T>,
        ) -> BalanceOf<T> {
            let activation_block = match allocation.status {
                AllocationStatus::ActiveSinceGenesis => BlockNumberFor::<T>::zero(),
                AllocationStatus::ActivatedAt(block) => block,
                AllocationStatus::Completed | AllocationStatus::Revoked => {
                    return Zero::zero();
                }
            };

            let envelope_config = match EnvelopeConfigs::<T>::get(&allocation.envelope_type) {
                Some(config) => config,
                None => return Zero::zero(),
            };

            let immediate_amount = envelope_config
                .immediate_unlock_percentage
                .mul_floor(allocation.total_allocation);

            let cliff_end = activation_block.saturating_add(envelope_config.cliff_duration);
            if current_block < cliff_end {
                return immediate_amount;
            }

            let vesting_end = activation_block.saturating_add(envelope_config.vesting_duration);
            let vesting_amount = allocation.total_allocation.saturating_sub(immediate_amount);

            if current_block >= vesting_end {
                allocation.total_allocation
            } else {
                let vesting_elapsed = current_block.saturating_sub(cliff_end);
                let vesting_remaining = vesting_end.saturating_sub(cliff_end);

                let vested_amount = if !vesting_remaining.is_zero() {
                    let progress = Percent::from_rational(vesting_elapsed, vesting_remaining);
                    progress.mul_floor(vesting_amount)
                } else {
                    vesting_amount
                };

                immediate_amount.saturating_add(vested_amount)
            }
        }
    }
}
