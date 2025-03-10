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

extern crate alloc;

use allfeat_support::traits::Certifier;
use alloc::collections::VecDeque;
use core::marker::PhantomData;
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::SaturatedConversion;
use frame_support::traits::fungible::Inspect;
use frame_support::traits::fungible::MutateHold;
use frame_support::traits::DefensiveSaturating;
use frame_support::traits::UnixTime;
use frame_system::pallet_prelude::*;
use types::{CertifState, CertifStatus};

mod mock;
mod tests;
pub mod types;

pub use pallet::*;

impl<T: Config> Certifier<MiddsHashIdOf<T>> for Pallet<T> {
	fn add_to_certif_process(midds_id: MiddsHashIdOf<T>) -> DispatchResult {
		// We do not accept it if the same midds ID is already in this pallet logic. (This likely
		// should never happened, this is unexpected !!)
		ensure!(Certifications::<T>::get(midds_id).is_none(), Error::<T>::CertifStateAlreadyExist);
		Certifications::<T>::insert(midds_id, CertifState::new());
		Ok(())
	}

	fn is_voting_period(midds_id: MiddsHashIdOf<T>) -> bool {
		Certifications::<T>::get(midds_id)
			.is_some_and(|state| matches!(state.status, CertifStatus::Voting(_)))
	}
	fn is_precertified(midds_id: MiddsHashIdOf<T>) -> bool {
		Certifications::<T>::get(midds_id)
			.is_some_and(|state| matches!(state.status, CertifStatus::Precertif(_)))
	}
	fn is_certified(midds_id: MiddsHashIdOf<T>) -> bool {
		Certifications::<T>::get(midds_id)
			.is_some_and(|state| matches!(state.status, CertifStatus::Certif(_)))
	}
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use allfeat_support::types::certification::PrecertifInfos;

	use super::*;

	pub type MiddsHashIdOf<T> = <T as frame_system::Config>::Hash;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
	pub type CertifStateOf<T> = CertifState<BalanceOf<T>>;

	/// The in-code storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	/// TODO: runtime default implementation
	#[pallet::config(with_default)]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		#[pallet::no_default]
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency type used to stake for votes.
		#[pallet::no_default]
		type Currency: MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;

		/// Time used for computing duration.
		#[pallet::no_default]
		type Time: UnixTime;

		/// Overarching hold reason.
		#[pallet::no_default]
		type RuntimeHoldReason: From<HoldReason>;

		/// The origin who can vote for MIDDS certification (e.g trusters collective).
		#[pallet::no_default]
		type VoteOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// The minimum that a voter need to stake as a vote on a single MIDDS.
		type MinStakePerVote: Get<BalanceOf<Self>>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// The limit that a voter can stake vote on a single MIDDS.
		type MaxStakePerVote: Get<BalanceOf<Self>>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// Amount which is required in total for a MIDDS to be vote staked to be pre-certified.
		type ThresholdCertifiedAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		#[pallet::no_default_bounds]
		/// Amount of milliseconds that need to pass to certify a MIDDS in pre-certification status.
		type WaitTimeForPreToCertif: Get<u128>;
	}

	/// Container for different types that implement [`DefaultConfig`]` of this pallet.
	pub mod config_preludes {
		// This will help use not need to disambiguate anything when using `derive_impl`.
		use super::*;
		use frame_support::{
			derive_impl,
			traits::{ConstU128, ConstU64},
		};

		/// A type providing default configurations for this pallet in testing environment.
		pub struct TestDefaultConfig;

		#[derive_impl(frame_system::config_preludes::TestDefaultConfig, no_aggregated_types)]
		impl frame_system::DefaultConfig for TestDefaultConfig {}

		#[frame_support::register_default_impl(TestDefaultConfig)]
		impl DefaultConfig for TestDefaultConfig {
			type MinStakePerVote = ConstU64<10>;
			type MaxStakePerVote = ConstU64<500>;
			type ThresholdCertifiedAmount = ConstU64<1000>;
			type WaitTimeForPreToCertif = ConstU128<100>; // 100 milliseconds
		}
	}

	/// A reason for placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Funds on stake by a nominator or a validator.
		CertificationVote,
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(PhantomData<T>);

	/// Votes are registered by mapping the AccountId of trusters + the MiddsId that the truster
	/// voted for, to the total amount that the truster vote staked.
	#[pallet::storage]
	pub type Votes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId, // Truster address
		Twox64Concat,
		MiddsHashIdOf<T>, // Midds identifier
		BalanceOf<T>,     // Total vote staked
		OptionQuery,
	>;

	#[pallet::storage]
	pub type Certifications<T: Config> =
		StorageMap<_, Blake2_128Concat, MiddsHashIdOf<T>, CertifStateOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type PreCertificationsQueue<T: Config> =
		StorageValue<_, VecDeque<MiddsHashIdOf<T>>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn integrity_test() {
			// Ensure MinStakePerVote is not 0.
			assert!(
				!T::MinStakePerVote::get().is_zero(),
				"The minimum vote amount must be greater then zero!"
			);
			// Ensure MaxStakePerVote is not less than MinStakePerVote.
			assert!(
				T::MaxStakePerVote::get() >= T::MinStakePerVote::get(),
				"The maximum vote amount should be greater or equal to the minimum vote amount!"
			);
			// Ensure ThresholdCertifiedAmount is not less than MaxStakePerVote.
			assert!(
				T::ThresholdCertifiedAmount::get() >= T::MaxStakePerVote::get(),
				"The maximum vote amount can't be bigger than the required threshold to certify."
			);
		}

		fn on_idle(n: BlockNumberFor<T>, remaining_weights: Weight) -> Weight {
			Self::process_precertif_upgrades(n);
			remaining_weights
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The vote amount can't be zero.
		NullVoteAmount,
		/// The truster tried to vote without enough required amount.
		MinVoteAmountRequired,
		/// The truster tried to submit a vote amount that exceed the maximum amount authorized per
		/// truster.
		MaxVoteAmountExceeded,
		/// The specified MIDDS isn't registered in the certification process (more likely the MIDDS
		/// do not exist!)
		NoCertifState,
		/// The specified MIDDS being tried to be add to the certification process is already
		/// existing.
		CertifStateAlreadyExist,
		/// The specified MIDDS isn't in a voting state.
		NoVotingPeriod,
		/// The current block is the genesis and the action can't happen (e.g time provider not ready)
		ProhibitedOnGenesis,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A truster has voted for the certification of a MIDDS.
		Voted { truster: T::AccountId, midds_id: MiddsHashIdOf<T>, vote_amount: BalanceOf<T> },

		/// A MIDDS have been upgraded to Pre-certified state.
		Precertified { midds_id: MiddsHashIdOf<T> },

		/// A MIDDS have been upgraded to Certified state.
		Certified { midds_id: MiddsHashIdOf<T> },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		pub fn vote_for(
			origin: OriginFor<T>,
			midds_id: MiddsHashIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			// First, we ensure that the vote is emitted from a truster.
			let truster_id = T::VoteOrigin::ensure_origin(origin)?;
			let min_stake_per_vote = T::MinStakePerVote::get();
			let max_stake_per_vote = T::MaxStakePerVote::get();

			// We ensure that the vote amount is not zero.
			ensure!(!amount.is_zero(), Error::<T>::NullVoteAmount);

			// A flag that decide if we should process the upgrade to precertif status at the end of
			// this call.
			let mut should_upgrade_to_precertif: bool = false;

			// Certif state mutation
			Certifications::<T>::try_mutate_exists(midds_id, |maybe_state| -> DispatchResult {
				if let Some(state) = maybe_state {
					match state.status {
						CertifStatus::Voting(ref mut infos) => {
							infos.add_staked(amount);
							// We check if the threshold for the certification is triggered or not
							// and upgrade to pre-certified state if it is.
							if infos.total_staked() >= T::ThresholdCertifiedAmount::get() {
								should_upgrade_to_precertif = true;
							};
							Ok(())
						},
						_ => Err(Error::<T>::NoVotingPeriod.into()),
					}
				} else {
					Err(Error::<T>::NoCertifState.into())
				}
			})?;

			// Funds reservation for the vote.
			T::Currency::hold(&HoldReason::CertificationVote.into(), &truster_id, amount)?;

			// Votes registry mutation
			Votes::<T>::mutate(&truster_id, midds_id, |maybe_vote| -> DispatchResult {
				if let Some(vote) = maybe_vote {
					// If the truster has already voted, we update his total vote amount by adding
					// the new specified amount value and we re-check that the amount is still
					// correct.
					let new_vote_value = vote.defensive_saturating_add(amount);

					// As in this case, the truster has already voted before this one, we do not
					// enforce the minimum value check, but only the maximum.
					ensure!(
						new_vote_value <= max_stake_per_vote,
						Error::<T>::MaxVoteAmountExceeded
					);
					*vote = new_vote_value;
					Ok(())
				} else {
					// Else, the truster has never voted for this MIDDS and so we simply check that
					// the amount is correct and set it.

					// We ensure that the vote have the required minimum vote value.
					ensure!(amount >= min_stake_per_vote, Error::<T>::MinVoteAmountRequired);
					ensure!(amount <= max_stake_per_vote, Error::<T>::MaxVoteAmountExceeded);
					*maybe_vote = Some(amount);
					Ok(())
				}
			})?;

			if should_upgrade_to_precertif {
				Self::upgrade_to_precertif(midds_id)?;
			};

			// Deposit an Event for the new vote.
			Self::deposit_event(Event::<T>::Voted {
				truster: truster_id,
				midds_id,
				vote_amount: amount,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Pre-certification upgrade check.
		/// Note: This shouldn't run on genesis to ensure timestamp is set.
		fn process_precertif_upgrades(current_block: BlockNumberFor<T>) {
			log::log!(log::Level::Debug, "Starting Pre-certified to Certified upgrade checks...");

			// TODO: limit iteration possibility to ensure weights integrity ?
			if current_block > 0u8.saturated_into() {
				PreCertificationsQueue::<T>::mutate(|queue: &mut VecDeque<MiddsHashIdOf<T>>| {
					while Self::precertif_queue_last_should_be_removed(queue) {
						// We remove it from the precertif queue.
						let midds_id = queue.pop_front().expect("Option already checked before");
						// Then we upgrade it to Certified status.
						Certifications::<T>::mutate(midds_id, |maybe_certif| {
							if let Some(ref mut certif) = maybe_certif {
								certif.status = CertifStatus::Certif(());
							} else {
								unreachable!("Certif state should exist. This is unexpected.")
							}
						});

						Self::deposit_event(Event::<T>::Certified { midds_id });
					}
				});
			};
		}

		/// Return if the last element of the Precertification FIFO queue should be removed and
		/// certified by checking the elapsed time since the upgrade to pre-certification status.
		fn precertif_queue_last_should_be_removed(queue: &mut VecDeque<MiddsHashIdOf<T>>) -> bool {
			let wait_time = T::WaitTimeForPreToCertif::get();

			if let Some(midds_id) = queue.front() {
				log::log!(
					log::Level::Trace,
					"Found midds ID {} as the front element of the queue",
					midds_id
				);
				let certif_data = Certifications::<T>::get(midds_id);
				let certif_status =
					certif_data.expect("certif data not found, this is highly unexpected.").status;
				match certif_status {
					CertifStatus::Precertif(x) => {
						let now = T::Time::now();
						let elapsed_time = now
							.as_millis()
							.defensive_saturating_sub(x.precertif_timestamp().as_millis());

						elapsed_time > wait_time
					},
					_ => unreachable!(
						"Should never be not Precertif status here, this is highly unexpected."
					),
				}
			} else {
				false
			}
		}

		/// Upgrade the specified `midds_id` to the precertified status.
		fn upgrade_to_precertif(midds_id: MiddsHashIdOf<T>) -> DispatchResult {
			Certifications::<T>::try_mutate_exists(midds_id, |maybe_state| -> DispatchResult {
				if let Some(state) = maybe_state {
					state.status = CertifStatus::Precertif(PrecertifInfos {
						precertif_timestamp: T::Time::now(),
					});
					Ok(())
				} else {
					unreachable!("State should exist in this context, this is unexpected.")
				}
			})?;

			PreCertificationsQueue::<T>::mutate(|queue: &mut VecDeque<MiddsHashIdOf<T>>| {
				queue.push_back(midds_id)
			});

			Self::deposit_event(Event::<T>::Precertified { midds_id });

			Ok(())
		}
	}
}
