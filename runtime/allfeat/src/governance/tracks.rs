//! Track configurations for governance.

use super::*;
use pallet_referenda::Curve;

const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
	sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}

const APP_ROOT: Curve = Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_ROOT: Curve = Curve::make_linear(28, 28, percent(0), percent(50));
const APP_STAKING_ADMIN: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_STAKING_ADMIN: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_FELLOWSHIP_ADMIN: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_FELLOWSHIP_ADMIN: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_GENERAL_ADMIN: Curve =
	Curve::make_reciprocal(4, 28, percent(80), percent(50), percent(100));
const SUP_GENERAL_ADMIN: Curve =
	Curve::make_reciprocal(7, 28, percent(10), percent(0), percent(50));
const APP_REFERENDUM_CANCELLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_CANCELLER: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_REFERENDUM_KILLER: Curve = Curve::make_linear(17, 28, percent(50), percent(100));
const SUP_REFERENDUM_KILLER: Curve =
	Curve::make_reciprocal(12, 28, percent(1), percent(0), percent(50));
const APP_WHITELISTED_CALLER: Curve =
	Curve::make_reciprocal(16, 28 * 24, percent(96), percent(50), percent(100));
const SUP_WHITELISTED_CALLER: Curve =
	Curve::make_reciprocal(1, 28, percent(20), percent(5), percent(50));

const TRACKS_DATA: [(u16, pallet_referenda::TrackInfo<Balance, BlockNumber>); 7] = [
	(
		0,
		pallet_referenda::TrackInfo {
			name: "root",
			max_deciding: 1,
			decision_deposit: 100 * AFT,
			prepare_period: 8 * MINUTES,
			decision_period: 20 * MINUTES,
			confirm_period: 12 * MINUTES,
			min_enactment_period: 5 * MINUTES,
			min_approval: APP_ROOT,
			min_support: SUP_ROOT,
		},
	),
	(
		1,
		pallet_referenda::TrackInfo {
			name: "whitelisted_caller",
			max_deciding: 100,
			decision_deposit: 10 * AFT,
			prepare_period: 6 * MINUTES,
			decision_period: 20 * MINUTES,
			confirm_period: 4 * MINUTES,
			min_enactment_period: 3 * MINUTES,
			min_approval: APP_WHITELISTED_CALLER,
			min_support: SUP_WHITELISTED_CALLER,
		},
	),
	(
		10,
		pallet_referenda::TrackInfo {
			name: "staking_admin",
			max_deciding: 10,
			decision_deposit: 5 * AFT,
			prepare_period: 8 * MINUTES,
			decision_period: 20 * MINUTES,
			confirm_period: 8 * MINUTES,
			min_enactment_period: 3 * MINUTES,
			min_approval: APP_STAKING_ADMIN,
			min_support: SUP_STAKING_ADMIN,
		},
	),
	(
		13,
		pallet_referenda::TrackInfo {
			name: "fellowship_admin",
			max_deciding: 10,
			decision_deposit: 5 * AFT,
			prepare_period: 8 * MINUTES,
			decision_period: 20 * MINUTES,
			confirm_period: 8 * MINUTES,
			min_enactment_period: 3 * MINUTES,
			min_approval: APP_FELLOWSHIP_ADMIN,
			min_support: SUP_FELLOWSHIP_ADMIN,
		},
	),
	(
		14,
		pallet_referenda::TrackInfo {
			name: "general_admin",
			max_deciding: 10,
			decision_deposit: 5 * AFT,
			prepare_period: 8 * MINUTES,
			decision_period: 20 * MINUTES,
			confirm_period: 8 * MINUTES,
			min_enactment_period: 3 * MINUTES,
			min_approval: APP_GENERAL_ADMIN,
			min_support: SUP_GENERAL_ADMIN,
		},
	),
	(
		20,
		pallet_referenda::TrackInfo {
			name: "referendum_canceller",
			max_deciding: 1_000,
			decision_deposit: 10 * AFT,
			prepare_period: 8 * MINUTES,
			decision_period: 14 * MINUTES,
			confirm_period: 8 * MINUTES,
			min_enactment_period: 3 * MINUTES,
			min_approval: APP_REFERENDUM_CANCELLER,
			min_support: SUP_REFERENDUM_CANCELLER,
		},
	),
	(
		21,
		pallet_referenda::TrackInfo {
			name: "referendum_killer",
			max_deciding: 1_000,
			decision_deposit: 50 * AFT,
			prepare_period: 8 * MINUTES,
			decision_period: 20 * MINUTES,
			confirm_period: 8 * MINUTES,
			min_enactment_period: 3 * MINUTES,
			min_approval: APP_REFERENDUM_KILLER,
			min_support: SUP_REFERENDUM_KILLER,
		},
	),
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;

	fn tracks() -> &'static [(Self::Id, pallet_referenda::TrackInfo<Balance, BlockNumber>)] {
		&TRACKS_DATA[..]
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
			match custom_origin {
				origins::Origin::WhitelistedCaller => Ok(1),
				// General admin
				origins::Origin::StakingAdmin => Ok(10),
				origins::Origin::FellowshipAdmin => Ok(13),
				origins::Origin::GeneralAdmin => Ok(14),
				// Referendum admins
				origins::Origin::ReferendumCanceller => Ok(20),
				origins::Origin::ReferendumKiller => Ok(21),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}
pallet_referenda::impl_tracksinfo_get!(TracksInfo, Balance, BlockNumber);
