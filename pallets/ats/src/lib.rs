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

use ark_bn254::{Bn254, Fr};
use ark_ff::{BigInteger, PrimeField};
use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
use ark_serialize::{CanonicalDeserialize, SerializationError};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::prelude::vec::Vec;

#[frame_support::pallet()]
pub mod pallet {
    use super::*;
    #[cfg(feature = "runtime-benchmarks")]
    use frame_support::traits::fungible::Mutate;
    use frame_support::traits::fungible::MutateHold;

    pub type Hash256 = [u8; 32];
    pub type AtsId = u64;
    pub type VersionNumber = u32;

    /// Public inputs for the claim ZKP verification
    #[derive(
        Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, DecodeWithMemTracking, Debug,
    )]
    pub struct ZkpPublicInputs {
        pub hash_title: Hash256,
        pub hash_audio: Hash256,
        pub hash_creators: Hash256,
        pub hash_commitment: Hash256,
        pub zkp_timestamp: Hash256,
        pub nullifier: Hash256,
    }

    /// The in-code storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// Maximum size for a Groth16 proof (in bytes)
    /// A Groth16 proof consists of 3 G1 points (2 for π_A, π_B, π_C)
    /// Each G1 point is 32 bytes compressed = 96 bytes total
    /// We add some buffer for safety
    pub const MAX_PROOF_SIZE: u32 = 256;

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
            type RuntimeHoldReason = ();
            type AtsRegistrationCost = ConstU64<1>;
            type WeightInfo = ();
        }
    }

    #[pallet::config(with_default)]
    pub trait Config: frame_system::Config {
        #[pallet::no_default]
        #[cfg(not(feature = "runtime-benchmarks"))]
        /// The currency trait used to manage ATS payments.
        type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

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

        #[pallet::no_default]
        /// Origin that can update the verification key.
        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        #[pallet::constant]
        #[pallet::no_default_bounds]
        /// The fixed deposit cost for registering an ATS on-chain.
        type AtsRegistrationCost: Get<BalanceOf<Self>>;

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
        Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, DecodeWithMemTracking, Debug,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound(T: Config))]
    pub struct AtsWork<T: Config> {
        pub owner: T::AccountId,
        pub id: AtsId,
    }

    #[derive(
        Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, DecodeWithMemTracking, Debug,
    )]
    #[scale_info(skip_type_params(T))]
    #[codec(mel_bound(T: Config))]
    pub struct AtsVersion<T: Config> {
        pub version: VersionNumber,
        pub hash_commitment: Hash256,
        pub registered_at: BlockNumberFor<T>,
    }

    /// Counter for generating unique ATS IDs
    #[pallet::storage]
    pub type NextAtsId<T: Config> = StorageValue<_, AtsId, ValueQuery>;

    /// Maps ATS ID to AtsWork
    #[pallet::storage]
    pub type AtsWorks<T: Config> = StorageMap<_, Blake2_128Concat, AtsId, AtsWork<T>>;

    /// Maps (ATS ID, VersionNumber) to AtsVersion
    #[pallet::storage]
    pub type AtsVersions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        AtsId,
        Blake2_128Concat,
        VersionNumber,
        AtsVersion<T>,
    >;

    /// Maps hash_commitment to ATS ID (for backward compatibility and lookup)
    #[pallet::storage]
    pub type AtsIdByHash<T: Config> = StorageMap<_, Blake2_128Concat, Hash256, AtsId>;

    /// Latest version number for each ATS ID
    #[pallet::storage]
    pub type LatestVersion<T: Config> =
        StorageMap<_, Blake2_128Concat, AtsId, VersionNumber, ValueQuery>;

    /// Maps owner to their list of ATS IDs
    #[pallet::storage]
    #[pallet::unbounded]
    pub type AtsByOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<AtsId>>;

    /// Stores the verification key used for ZKP verification
    #[pallet::storage]
    #[pallet::unbounded]
    pub type VerificationKey<T: Config> = StorageValue<_, Vec<u8>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ATSRegistered {
            provider: T::AccountId,
            ats_id: AtsId,
            hash_commitment: Hash256,
        },
        ATSClaimed {
            old_owner: T::AccountId,
            new_owner: T::AccountId,
            ats_id: AtsId,
        },
        ATSUpdated {
            owner: T::AccountId,
            ats_id: AtsId,
            version: VersionNumber,
            hash_commitment: Hash256,
        },
        VerificationKeyUpdated {
            vk: Vec<u8>,
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
        /// Serialization or deserialization failed
        InvalidData,
        /// Verification failed
        VerificationFailed,
        /// Verification key has not been set
        VerificationKeyNotSet,
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T>
    where
        T::AccountId: core::fmt::Debug,
    {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register(0))]
        pub fn register(origin: OriginFor<T>, hash_commitment: Hash256) -> DispatchResult {
            let sender = T::ProviderOrigin::ensure_origin(origin)?;

            // Check if ATS with this hash commitment already exists
            ensure!(
                !AtsIdByHash::<T>::contains_key(hash_commitment),
                Error::<T>::AtsDataAlreadyExist
            );

            // Get current block number
            let registered_at = <frame_system::Pallet<T>>::block_number();

            // Get next ATS ID
            let ats_id = NextAtsId::<T>::get();
            NextAtsId::<T>::put(ats_id.saturating_add(1));

            // Create AtsWork
            let ats_work = AtsWork::<T> {
                owner: sender.clone(),
                id: ats_id,
            };

            // Create first AtsVersion (version = 1)
            let ats_version = AtsVersion::<T> {
                version: 1,
                hash_commitment,
                registered_at,
            };

            // Get fixed registration cost
            let registration_cost = T::AtsRegistrationCost::get();

            // Hold the deposit from the sender
            T::Currency::hold(
                &HoldReason::AtsRegistration.into(),
                &sender,
                registration_cost,
            )
            .map_err(|_| Error::<T>::CantHoldFunds)?;

            // Store AtsWork and AtsVersion
            AtsWorks::<T>::insert(ats_id, ats_work);
            AtsVersions::<T>::insert(ats_id, 1, ats_version);
            AtsIdByHash::<T>::insert(hash_commitment, ats_id);
            LatestVersion::<T>::insert(ats_id, 1);

            // Add ATS ID to owner's list
            AtsByOwner::<T>::mutate(&sender, |maybe_list| {
                let list = maybe_list.get_or_insert_with(Vec::new);
                list.push(ats_id);
            });

            // Emit event
            Self::deposit_event(Event::ATSRegistered {
                provider: sender,
                ats_id,
                hash_commitment,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update(0))]
        pub fn update(
            origin: OriginFor<T>,
            ats_id: AtsId,
            hash_commitment: Hash256,
        ) -> DispatchResult {
            let sender = T::ProviderOrigin::ensure_origin(origin)?;

            // Get the ATS work and verify ownership
            let ats_work = AtsWorks::<T>::get(ats_id).ok_or(Error::<T>::AtsNotFound)?;
            ensure!(ats_work.owner == sender, Error::<T>::VerificationFailed);

            // Get the latest version number and increment it
            let current_version = LatestVersion::<T>::get(ats_id);
            let new_version = current_version.saturating_add(1);

            // Get current block number
            let registered_at = <frame_system::Pallet<T>>::block_number();

            // Create new AtsVersion
            let ats_version = AtsVersion::<T> {
                version: new_version,
                hash_commitment,
                registered_at,
            };

            // Get fixed registration cost
            let registration_cost = T::AtsRegistrationCost::get();

            // Hold the deposit from the sender
            T::Currency::hold(
                &HoldReason::AtsRegistration.into(),
                &sender,
                registration_cost,
            )
            .map_err(|_| Error::<T>::CantHoldFunds)?;

            // Store the new version
            AtsVersions::<T>::insert(ats_id, new_version, ats_version);
            LatestVersion::<T>::insert(ats_id, new_version);

            // Update the hash lookup to point to this ATS ID
            AtsIdByHash::<T>::insert(hash_commitment, ats_id);

            // Emit event
            Self::deposit_event(Event::ATSUpdated {
                owner: sender,
                ats_id,
                version: new_version,
                hash_commitment,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::claim())]
        pub fn claim(
            origin: OriginFor<T>,
            public_inputs: ZkpPublicInputs,
            proof: BoundedVec<u8, ConstU32<MAX_PROOF_SIZE>>,
        ) -> DispatchResult {
            let sender = T::ProviderOrigin::ensure_origin(origin)?;

            // Fetch verification key from storage
            let vk = VerificationKey::<T>::get().ok_or(Error::<T>::VerificationKeyNotSet)?;

            // Extract hash_commitment from public inputs
            let hash_commitment = public_inputs.hash_commitment;

            // Verify the zero-knowledge proof
            ensure!(
                Self::verify_zkp(vk, public_inputs, proof)?,
                Error::<T>::VerificationFailed
            );

            // Get the ATS ID from hash commitment
            let ats_id = AtsIdByHash::<T>::get(hash_commitment).ok_or(Error::<T>::AtsNotFound)?;

            // Get the latest version number
            let latest_version = LatestVersion::<T>::get(ats_id);

            // Get the latest version to verify hash_commitment matches
            let latest_ats_version =
                AtsVersions::<T>::get(ats_id, latest_version).ok_or(Error::<T>::AtsNotFound)?;

            // Verify the hash_commitment matches the latest version
            ensure!(
                latest_ats_version.hash_commitment == hash_commitment,
                Error::<T>::InvalidData
            );

            // Get and update the ATS work
            let mut ats_work = AtsWorks::<T>::get(ats_id).ok_or(Error::<T>::AtsNotFound)?;

            let old_owner = ats_work.owner.clone();

            // Update the owner
            ats_work.owner = sender.clone();
            AtsWorks::<T>::insert(ats_id, ats_work);

            // Remove ATS ID from old owner's list
            AtsByOwner::<T>::mutate(&old_owner, |maybe_list| {
                if let Some(list) = maybe_list {
                    list.retain(|id| id != &ats_id);
                }
            });

            // Add ATS ID to new owner's list
            AtsByOwner::<T>::mutate(&sender, |maybe_list| {
                let list = maybe_list.get_or_insert_with(Vec::new);
                list.push(ats_id);
            });

            // Emit event
            Self::deposit_event(Event::ATSClaimed {
                old_owner,
                new_owner: sender,
                ats_id,
            });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::set_verification_key())]
        pub fn set_verification_key(origin: OriginFor<T>, vk: Vec<u8>) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            VerificationKey::<T>::put(vk.clone());

            Self::deposit_event(Event::VerificationKeyUpdated { vk });

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
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

    fn verify_zkp(
        vk: Vec<u8>,
        public_inputs: ZkpPublicInputs,
        proof: BoundedVec<u8, ConstU32<{ pallet::MAX_PROOF_SIZE }>>,
    ) -> Result<bool, Error<T>> {
        // 1) Deserialize
        let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk.as_slice())
            .map_err(|_| Error::<T>::InvalidData)?;
        let pvk: PreparedVerifyingKey<Bn254> = ark_groth16::prepare_verifying_key(&vk);

        let proof = Proof::<Bn254>::deserialize_compressed(proof.as_slice())
            .map_err(|_| Error::<T>::InvalidData)?;

        // Convert public inputs to field elements in the correct order
        let pubs_array = [
            public_inputs.hash_title,
            public_inputs.hash_audio,
            public_inputs.hash_creators,
            public_inputs.hash_commitment,
            public_inputs.zkp_timestamp,
            public_inputs.nullifier,
        ];

        let mut publics: Vec<Fr> = Vec::with_capacity(pubs_array.len());
        for b in &pubs_array {
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
