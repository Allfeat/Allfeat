use frame_support::traits::OnRuntimeUpgrade;
use log::{log, Level};

use crate::*;

fn register_known_authorities<T: pallet_validator_set::Config>() -> Weight {
    let validator_1_account_id: AccountId = [67, 44, 84, 131, 201, 96, 74, 102, 81, 219, 116, 118, 44, 187, 169, 229, 202, 163, 3, 113].into();
    let validator_2_account_id: AccountId = [45, 195, 39, 44, 51, 133, 238, 196, 152, 210, 10, 51, 164, 220, 158, 53, 28, 3, 115, 211].into();

    let validator_1_id = T::ValidatorId::decode(&mut validator_1_account_id.0.as_slice()).unwrap();
    let validator_2_id = T::ValidatorId::decode(&mut validator_2_account_id.0.as_slice()).unwrap();

    let validators = vec![validator_1_id, validator_2_id];

    pallet_validator_set::Validators::<T>::set(validators);
    T::DbWeight::get().reads_writes(1, 2)
}

pub struct MigrateToPoA<T>(PhantomData<T>);
impl<T: pallet_validator_set::Config> OnRuntimeUpgrade for MigrateToPoA<T> {
    fn on_runtime_upgrade() -> Weight {
        log!(
            Level::Info,
            "Running consensus migration to PoA...",
        );

        register_known_authorities::<T>()
    }
}