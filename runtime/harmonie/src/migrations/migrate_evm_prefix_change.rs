use frame_support::{migration::move_pallet, traits::OnRuntimeUpgrade, weights::Weight};

pub struct MigrateEvmAlias;

impl OnRuntimeUpgrade for MigrateEvmAlias {
	fn on_runtime_upgrade() -> Weight {
		log::info!("Starting moving prefix storage from Evm to EVM pallet !");
		move_pallet(b"Evm", b"EVM");
		log::info!("successfuly moved storage prefix !");
		Weight::zero()
	}
}
