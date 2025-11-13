use crate::{Balances, Vec};
use crate::{MusicalWorks, Recordings, Releases, Runtime};
use allfeat_primitives::AccountId;
use frame_support::traits::PalletInfoAccess;
use frame_support::traits::fungible::MutateHold;
use frame_support::{migration::clear_storage_prefix, traits::OnRuntimeUpgrade};
use pallet_midds::HoldReason;
use sp_runtime::Weight;

pub struct MiddsCleaner;
impl OnRuntimeUpgrade for MiddsCleaner {
    fn on_runtime_upgrade() -> sp_runtime::Weight {
        let providers: Vec<AccountId> =
            pallet_midds::MiddsInfoOf::<Runtime, pallet_midds::Instance2>::iter_values()
                .map(|x| x.provider)
                .collect();

        let removals = clear_storage_prefix(MusicalWorks::name().as_bytes(), &[], &[], None, None);
        log::info!("Removed {} musical works.", removals.unique);
        let removals = clear_storage_prefix(Releases::name().as_bytes(), &[], &[], None, None);
        log::info!("Removed {} releases.", removals.unique);
        let removals = clear_storage_prefix(Recordings::name().as_bytes(), &[], &[], None, None);
        log::info!("Removed {} recordings.", removals.unique);

        let mut count = 0u32;
        providers.iter().for_each(|who| {
            let _ = <Balances as MutateHold<AccountId>>::release_all(
                &HoldReason::<pallet_midds::Instance2>::MiddsRegistration.into(),
                who,
                frame_support::traits::tokens::Precision::BestEffort,
            );
            count += 1;
        });

        log::info!("Unlocked MIDDS collateral for {count} accounts");

        Weight::zero()
    }
}
