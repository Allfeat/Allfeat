#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*; // Re-export the pallet for external access

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::traits::ExistenceRequirement::AllowDeath;
use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use sp_runtime::Percent;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum EnvelopeId {
    Founders,
    KoL,
    Private1,
    Private2,
    ICO1,
    Seed,
    ICO2,
    SerieA,
}

impl EnvelopeId {
    pub fn account<T: pallet::Config>(&self) -> T::AccountId {
        let pid = <T as pallet::Config>::PalletId::get();
        pid.into_sub_account_truncating(*self as u8)
    }
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// Per-envelope configuration describing budget and vesting policy
pub struct EnvelopeConfig<Balance, Moment> {
    pub total_cap: Balance,
    pub upfront_rate: Percent,
    pub cliff: Moment,
    pub vesting_duration: Moment,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// Per-beneficiary allocation state
pub struct Allocation<Balance> {
    pub total: Balance,
    pub upfront: Balance,
    pub vested_total: Balance,
    pub released: Balance,
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::PalletId;
    use frame_system::pallet_prelude::OriginFor;

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: Currency<Self::AccountId>;

        type AdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type Envelopes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        EnvelopeId,
        EnvelopeConfig<BalanceOf<T>, frame_system::pallet_prelude::BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type EnvelopeDistributed<T: Config> =
        StorageMap<_, Blake2_128Concat, EnvelopeId, BalanceOf<T>, ValueQuery>;

    #[pallet::storage]
    pub type Allocations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        EnvelopeId,
        Blake2_128Concat,
        T::AccountId,
        Allocation<BalanceOf<T>>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub envelopes: Vec<(EnvelopeId, EnvelopeConfig<BalanceOf<T>, u64>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                envelopes: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (id, cfg_in) in &self.envelopes {
                assert!(
                    !Envelopes::<T>::contains_key(id),
                    "duplicate envelope in genesis"
                );
                let cliff: frame_system::pallet_prelude::BlockNumberFor<T> = cfg_in
                    .cliff
                    .try_into()
                    .ok()
                    .expect("cliff fits into BlockNumber");
                let vesting: frame_system::pallet_prelude::BlockNumberFor<T> = cfg_in
                    .vesting_duration
                    .try_into()
                    .ok()
                    .expect("vesting fits into BlockNumber");
                let cfg = EnvelopeConfig::<
                    BalanceOf<T>,
                    frame_system::pallet_prelude::BlockNumberFor<T>,
                > {
                    total_cap: cfg_in.total_cap,
                    upfront_rate: cfg_in.upfront_rate,
                    cliff,
                    vesting_duration: vesting,
                };
                Envelopes::<T>::insert(id, cfg);
                EnvelopeDistributed::<T>::insert(id, BalanceOf::<T>::zero());
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AllocationAdded(EnvelopeId, T::AccountId, BalanceOf<T>),
        UpfrontPaid(EnvelopeId, T::AccountId, BalanceOf<T>),
        VestedReleased(EnvelopeId, T::AccountId, BalanceOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        EnvelopeUnknown,
        AllocationExists,
        EnvelopeCapExceeded,
        NothingToClaim,
        ArithmeticOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn add_allocation(
            origin: OriginFor<T>,
            id: EnvelopeId,
            who: T::AccountId,
            total: BalanceOf<T>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;
            let cfg = Envelopes::<T>::get(id).ok_or(Error::<T>::EnvelopeUnknown)?;
            ensure!(
                !Allocations::<T>::contains_key(id, &who),
                Error::<T>::AllocationExists
            );
            let distributed = EnvelopeDistributed::<T>::get(id);
            let new_distributed = distributed.saturating_add(total);
            ensure!(
                new_distributed <= cfg.total_cap,
                Error::<T>::EnvelopeCapExceeded
            );

            let source = id.account::<T>();
            let source_free = <T as Config>::Currency::free_balance(&source);
            // total liability introduced now is upfront + vested_total (to be paid over time)
            let upfront = cfg.upfront_rate.mul_floor(total);
            let vested_total = total.saturating_sub(upfront);
            let required = upfront.saturating_add(vested_total);
            ensure!(source_free >= required, Error::<T>::EnvelopeCapExceeded);

            let alloc = Allocation {
                total,
                upfront,
                vested_total,
                released: Zero::zero(),
            };
            Allocations::<T>::insert(id, &who, alloc.clone());
            EnvelopeDistributed::<T>::insert(id, new_distributed);

            if !upfront.is_zero() {
                <T as Config>::Currency::transfer(&source, &who, upfront, AllowDeath)?;
                Self::deposit_event(Event::UpfrontPaid(id, who.clone(), upfront));
            }
            Self::deposit_event(Event::AllocationAdded(id, who, total));
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn claim(origin: OriginFor<T>, id: EnvelopeId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let cfg = Envelopes::<T>::get(id).ok_or(Error::<T>::EnvelopeUnknown)?;
            let mut alloc = Allocations::<T>::get(id, &who).ok_or(Error::<T>::EnvelopeUnknown)?;
            let now = <frame_system::Pallet<T>>::block_number();

            let claimable =
                Self::claimable_amount(&cfg, &alloc, now).ok_or(Error::<T>::ArithmeticOverflow)?;
            ensure!(!claimable.is_zero(), Error::<T>::NothingToClaim);

            alloc.released = alloc.released.saturating_add(claimable);
            Allocations::<T>::insert(id, &who, &alloc);

            let source = id.account::<T>();
            <T as Config>::Currency::transfer(&source, &who, claimable, AllowDeath)?;
            Self::deposit_event(Event::VestedReleased(id, who, claimable));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn claimable_amount(
            cfg: &EnvelopeConfig<BalanceOf<T>, frame_system::pallet_prelude::BlockNumberFor<T>>,
            alloc: &Allocation<BalanceOf<T>>,
            now: frame_system::pallet_prelude::BlockNumberFor<T>,
        ) -> Option<BalanceOf<T>> {
            if now <= cfg.cliff {
                return Some(Zero::zero());
            }
            let elapsed = now.saturating_sub(cfg.cliff);
            if elapsed >= cfg.vesting_duration {
                return alloc.vested_total.saturating_sub(alloc.released).into();
            }
            let vested = Self::mul_div(alloc.vested_total, elapsed, cfg.vesting_duration)?;
            let available = vested.saturating_sub(alloc.released);
            Some(available)
        }

        pub fn mul_div(
            a: BalanceOf<T>,
            b: frame_system::pallet_prelude::BlockNumberFor<T>,
            c: frame_system::pallet_prelude::BlockNumberFor<T>,
        ) -> Option<BalanceOf<T>> {
            // naive: (a * b) / c with saturating casts via u128
            let a128: u128 = a.try_into().ok()?;
            let b128: u128 = b.try_into().ok()?;
            let c128: u128 = c.try_into().ok()?;
            if c128 == 0 {
                return None;
            }
            let res = a128.saturating_mul(b128).checked_div(c128)?;
            res.try_into().ok()
        }
    }
}
