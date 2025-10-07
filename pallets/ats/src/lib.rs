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
use types::BalanceOf;
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

mod benchmarking;

extern crate alloc;

use ark_bn254::{Bn254, Fr};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, SerializationError};
use frame_support::{pallet_prelude::*, sp_runtime::Saturating};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::prelude::vec::Vec;

#[frame_support::pallet()]
pub mod pallet {
    use super::*;
    use allfeat_primitives::Moment;
    #[cfg(feature = "runtime-benchmarks")]
    use frame_support::traits::fungible::Mutate;
    use frame_support::{
        PalletId,
        traits::{Time, fungible::MutateHold},
    };

    pub type Hash256 = [u8; 32];

    /// The in-code storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Default implementations of [`DefaultConfig`], which can be used to implement [`Config`].
    pub mod config_preludes {
        use super::*;
        use frame_support::{derive_impl, traits::ConstU64};

        pub struct TestDefaultConfig;

        #[derive_impl(frame_system::config_preludes::TestDefaultConfig, no_aggregated_types)]
        impl frame_system::DefaultConfig for TestDefaultConfig {}

        #[frame_support::register_default_impl(TestDefaultConfig)]
        impl DefaultConfig for TestDefaultConfig {
            #[inject_runtime_type]
            type RuntimeEvent = ();
            #[inject_runtime_type]
            type RuntimeHoldReason = ();
            type ByteDepositCost = ConstU64<1>;
            type WeightInfo = ();
        }
    }

    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config {
        /// The ATS pallet instance pallet id
        #[pallet::no_default]
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::no_default_bounds]
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::no_default]
        #[cfg(not(feature = "runtime-benchmarks"))]
        /// The currency trait used to manage ATS payments.
        type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

        #[pallet::no_default]
        type Timestamp: Time<Moment = Moment>;

        #[pallet::no_default]
        #[cfg(feature = "runtime-benchmarks")]
        /// The way to handle the storage deposit cost of Artist creation
        /// Include Currency trait to have access to NegativeImbalance
        type Currency: Mutate<Self::AccountId>
            + MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

        #[pallet::no_default_bounds]
        /// The overarching HoldReason type.
        type RuntimeHoldReason: From<HoldReason>;

        #[pallet::no_default]
        /// The origin which may provide new ATS to register on-chain for this instance.
        type ProviderOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        #[pallet::constant]
        #[pallet::no_default_bounds]
        /// The per-byte deposit cost when depositing ATS on-chain.
        type ByteDepositCost: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    /// A reason for the pallet ATS placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// A new ATS has been deposited and require colateral data value hold.
        AtsRegistration,
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[derive(
        Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, DecodeWithMemTracking,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound(T: Config))]
    pub struct AtsData<T: Config> {
        pub owner: T::AccountId,
        pub hash_commitment: Hash256,
        pub timestamp: Moment,
    }

    impl<T: Config> core::fmt::Debug for AtsData<T>
    where
        T::AccountId: core::fmt::Debug,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("AtsData")
                .field("owner", &self.owner)
                .field("hash_commitment", &self.hash_commitment)
                .field("timestamp", &self.timestamp)
                .finish()
        }
    }

    #[pallet::storage]
    pub type AtsOf<T: Config> = StorageMap<_, Blake2_128Concat, Hash256, AtsData<T>>;

    #[pallet::storage]
    pub type AtsByOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<Hash256, ConstU32<1000>>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ATSRegistered {
            provider: T::AccountId,
            hash_commitment: Hash256,
            data_collateral: BalanceOf<T>,
        },
        ATSClaimed {
            old_owner: T::AccountId,
            new_owner: T::AccountId,
            hash_commitment: Hash256,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A ATS with the same hash commitment is already registered.
        AtsDataAlreadyExist,
        /// The specified ATS hash commitment is not related to any pending ATS.
        AtsNotFound,
        /// Funds can't be held at this moment.
        CantHoldFunds,
        /// The owner has reached the maximum number of ATS entries.
        MaxAtsPerOwnerReached,
        /// Serialization or deserialization failed
        InvalidData,
        /// Verification failed
        VerificationFailed,
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: core::fmt::Debug,
    {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register((vk.len() + proof.len()) as u32))]
        pub fn register(
            origin: OriginFor<T>,
            vk: Vec<u8>,
            pubs: Vec<[u8; 32]>,
            proof: Vec<u8>,
        ) -> DispatchResult {
            let sender = T::ProviderOrigin::ensure_origin(origin)?;

            // Extract hash_commitment from the 4th element of pubs
            let hash_commitment = *pubs.get(3).ok_or(Error::<T>::InvalidData)?;

            // Verify the zero-knowledge proof
            ensure!(
                Self::verify_zkp(vk, pubs, proof)?,
                Error::<T>::VerificationFailed
            );

            // Check if ATS with this hash commitment already exists
            ensure!(
                !AtsOf::<T>::contains_key(hash_commitment),
                Error::<T>::AtsDataAlreadyExist
            );

            // Get current timestamp
            let timestamp = T::Timestamp::now();

            // Calculate storage deposit based on encoded size
            let ats_data = AtsData::<T> {
                owner: sender.clone(),
                hash_commitment,
                timestamp,
            };
            let size = ats_data.encoded_size() as u32;
            let data_cost = Self::calculate_ats_colateral(size);

            // Hold the deposit from the sender
            T::Currency::hold(&HoldReason::AtsRegistration.into(), &sender, data_cost)
                .map_err(|_| Error::<T>::CantHoldFunds)?;

            // Store ATS data
            AtsOf::<T>::insert(hash_commitment, ats_data);

            // Add hash commitment to owner's list
            AtsByOwner::<T>::try_mutate(&sender, |maybe_list| -> DispatchResult {
                let list = maybe_list.get_or_insert_with(|| BoundedVec::default());
                list.try_push(hash_commitment)
                    .map_err(|_| Error::<T>::MaxAtsPerOwnerReached)?;
                Ok(())
            })?;

            // Emit event
            Self::deposit_event(Event::ATSRegistered {
                provider: sender,
                hash_commitment,
                data_collateral: data_cost,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            vk: Vec<u8>,
            pubs: Vec<[u8; 32]>,
            proof: Vec<u8>,
        ) -> DispatchResult {
            let sender = T::ProviderOrigin::ensure_origin(origin)?;

            // Extract hash_commitment from the 4th element of pubs
            let hash_commitment = *pubs.get(3).ok_or(Error::<T>::InvalidData)?;

            // Verify the zero-knowledge proof
            ensure!(
                Self::verify_zkp(vk, pubs, proof)?,
                Error::<T>::VerificationFailed
            );

            // Get the ATS data
            let mut ats_data = AtsOf::<T>::get(hash_commitment).ok_or(Error::<T>::AtsNotFound)?;

            let old_owner = ats_data.owner.clone();

            // Update the owner
            ats_data.owner = sender.clone();
            AtsOf::<T>::insert(hash_commitment, ats_data);

            // Remove hash commitment from old owner's list
            AtsByOwner::<T>::mutate(&old_owner, |maybe_list| {
                if let Some(list) = maybe_list {
                    list.retain(|h| h != &hash_commitment);
                }
            });

            // Add hash commitment to new owner's list
            AtsByOwner::<T>::try_mutate(&sender, |maybe_list| -> DispatchResult {
                let list = maybe_list.get_or_insert_with(|| BoundedVec::default());
                list.try_push(hash_commitment)
                    .map_err(|_| Error::<T>::MaxAtsPerOwnerReached)?;
                Ok(())
            })?;

            // Emit event
            Self::deposit_event(Event::ATSClaimed {
                old_owner,
                new_owner: sender,
                hash_commitment,
            });

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn calculate_ats_colateral(size: u32) -> types::BalanceOf<T> {
        T::ByteDepositCost::get().saturating_mul(types::BalanceOf::<T>::from(size))
    }

    fn fr_from_be32(bytes: &[u8; 32]) -> Result<Fr, SerializationError> {
        // reduce mod p from BE bytes
        let x = Fr::from_be_bytes_mod_order(bytes);

        // re-encode canonically (minimal BE) and compare with input
        let be_min = x.into_bigint().to_bytes_be();
        if be_min.len() > 32 {
            return Err(SerializationError::InvalidData);
        }
        let mut be32 = [0u8; 32];
        be32[32 - be_min.len()..].copy_from_slice(&be_min);

        if &be32 == bytes {
            Ok(x)
        } else {
            Err(SerializationError::InvalidData)
        }
    }

    fn verify_zkp(vk: Vec<u8>, pubs: Vec<[u8; 32]>, proof: Vec<u8>) -> Result<bool, Error<T>> {
        // 1) Deserialize
        let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk.as_slice())
            .map_err(|_| Error::<T>::InvalidData)?;
        let pvk: PreparedVerifyingKey<Bn254> = ark_groth16::prepare_verifying_key(&vk);

        let proof = Proof::<Bn254>::deserialize_compressed(proof.as_slice())
            .map_err(|_| Error::<T>::InvalidData)?;

        let mut publics: Vec<Fr> = Vec::with_capacity(pubs.len());
        for b in &pubs {
            publics.push(Self::fr_from_be32(b).map_err(|_| Error::<T>::InvalidData)?);
        }

        // 2) Verify
        let proof_ok = match Groth16::<Bn254>::verify_proof(&pvk, &proof, &publics) {
            Ok(true) => true,
            Ok(false) => false,
            Err(_) => false,
        };

        Ok(proof_ok)
    }
}
