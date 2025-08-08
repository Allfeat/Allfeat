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

//! # Pallet Validators
//!
//! This pallet provides dynamic management of the validator set for a Proof-of-Authority (PoA) consensus chain.
//!
//! ## Features
//! - Add or remove validators via Root or governance origin.
//! - Integration with `pallet-session` to update the validator set at each new session.
//! - Compatible with `pallet-session::historical` for tracking validators across sessions.
//!
//! ## Security
//! - Configurable maximum number of validators (`MaxValidators`).
//! - Checks for duplicates and existence before modification.
//! - Strict origin enforcement for validator set updates.
//!
//! ## Typical Use Case
//! Designed for PoA-based blockchains requiring on-chain validator management with session-based rotation.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::vec;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

pub const LOG_TARGET: &str = "runtime::validators-set";

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_session::Config {
        /// Max number of validators in the set
        #[pallet::constant]
        type MaxValidators: Get<u32>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Validators<T: Config> =
        StorageValue<_, BoundedVec<T::ValidatorId, T::MaxValidators>, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_validators: Vec<T::ValidatorId>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let bounded_validators: BoundedVec<_, _> = self
                .initial_validators
                .clone()
                .try_into()
                .expect("Too many validators in genesis");
            Validators::<T>::put(bounded_validators);
        }
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_validators: vec![],
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValidatorAdded(T::ValidatorId),
        ValidatorRemoved(T::ValidatorId),
        ValidatorSetUpdated,
    }

    #[pallet::error]
    pub enum Error<T> {
        ValidatorAlreadyPresent,
        ValidatorNotFound,
        TooManyValidators,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a new validator (Root or governance controlled)
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::add_validator())]
        pub fn add_validator(origin: OriginFor<T>, validator: T::ValidatorId) -> DispatchResult {
            ensure_root(origin)?;

            log::debug!(target: LOG_TARGET, "Validator addition initiated.");

            let mut current = Validators::<T>::get();
            if current.contains(&validator) {
                return Err(Error::<T>::ValidatorAlreadyPresent.into());
            }
            current
                .try_push(validator.clone())
                .map_err(|_| Error::<T>::TooManyValidators)?;
            Validators::<T>::put(&current);
            Self::deposit_event(Event::ValidatorAdded(validator));
            Ok(())
        }

        /// Remove a validator
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_validator())]
        pub fn remove_validator(origin: OriginFor<T>, validator: T::ValidatorId) -> DispatchResult {
            ensure_root(origin)?;

            log::debug!(target: LOG_TARGET, "Validator removal initiated.");

            let mut current = Validators::<T>::get();

            if !current.contains(&validator) {
                return Err(Error::<T>::ValidatorNotFound.into());
            }
            current.retain(|v| v != &validator);
            Validators::<T>::put(&current);
            Self::deposit_event(Event::ValidatorRemoved(validator));
            Ok(())
        }
    }
}

use sp_runtime::Vec;
use sp_staking::SessionIndex;

impl<T: Config> pallet_session::SessionManager<T::ValidatorId> for Pallet<T> {
    fn new_session(_index: SessionIndex) -> Option<Vec<T::ValidatorId>> {
        log::debug!(target: LOG_TARGET, "New session called; updating validator set provided.");
        Some(Validators::<T>::get().to_vec())
    }

    fn start_session(_index: SessionIndex) {}
    fn end_session(_index: SessionIndex) {}
}

impl<T: Config> pallet_session::historical::SessionManager<T::ValidatorId, T::ValidatorId>
    for Pallet<T>
{
    fn new_session(new_index: SessionIndex) -> Option<Vec<(T::ValidatorId, T::ValidatorId)>> {
        <Self as pallet_session::SessionManager<_>>::new_session(new_index)
            .map(|validators| validators.into_iter().map(|v| (v.clone(), v)).collect())
    }

    fn new_session_genesis(
        new_index: SessionIndex,
    ) -> Option<Vec<(T::ValidatorId, T::ValidatorId)>> {
        <Self as pallet_session::SessionManager<_>>::new_session(new_index)
            .map(|validators| validators.into_iter().map(|v| (v.clone(), v)).collect())
    }

    fn end_session(end_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::end_session(end_index)
    }

    fn start_session(start_index: SessionIndex) {
        <Self as pallet_session::SessionManager<_>>::start_session(start_index)
    }
}
