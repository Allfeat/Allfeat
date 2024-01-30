//! Custom origins for governance interventions.

pub use pallet_custom_origins::*;

#[frame_support::pallet]
pub mod pallet_custom_origins {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(PartialEq, Eq, Clone, MaxEncodedLen, Encode, Decode, TypeInfo, RuntimeDebug)]
	#[pallet::origin]
	pub enum Origin {
		/// Origin for cancelling slashes.
		StakingAdmin,
		/// Origin for managing the composition of the fellowship.
		FellowshipAdmin,
		/// Origin for managing the registrar.
		GeneralAdmin,
		/// Origin able to cancel referenda.
		ReferendumCanceller,
		/// Origin able to kill referenda.
		ReferendumKiller,
		/// Origin able to dispatch a whitelisted call.
		WhitelistedCaller,
		/// Origin commanded by any members of the Allfeat Fellowship (no Dan grade needed).
		FellowshipInitiates,
		/// Origin commanded by Allfeat Fellows (3rd Dan fellows or greater).
		Fellows,
		/// Origin commanded by Allfeat Experts (5th Dan fellows or greater).
		FellowshipExperts,
		/// Origin commanded by Allfeat Masters (7th Dan fellows of greater).
		FellowshipMasters,
		/// Origin commanded by rank 1 of the Allfeat Fellowship and with a success of 1.
		Fellowship1Dan,
		/// Origin commanded by rank 2 of the Allfeat Fellowship and with a success of 2.
		Fellowship2Dan,
		/// Origin commanded by rank 3 of the Allfeat Fellowship and with a success of 3.
		Fellowship3Dan,
		/// Origin commanded by rank 4 of the Allfeat Fellowship and with a success of 4.
		Fellowship4Dan,
		/// Origin commanded by rank 5 of the Allfeat Fellowship and with a success of 5.
		Fellowship5Dan,
		/// Origin commanded by rank 6 of the Allfeat Fellowship and with a success of 6.
		Fellowship6Dan,
		/// Origin commanded by rank 7 of the Allfeat Fellowship and with a success of 7.
		Fellowship7Dan,
		/// Origin commanded by rank 8 of the Allfeat Fellowship and with a success of 8.
		Fellowship8Dan,
		/// Origin commanded by rank 9 of the Allfeat Fellowship and with a success of 9.
		Fellowship9Dan,
	}

	macro_rules! decl_unit_ensures {
		( $name:ident: $success_type:ty = $success:expr ) => {
			pub struct $name;
			impl<O: Into<Result<Origin, O>> + From<Origin>>
				EnsureOrigin<O> for $name
			{
				type Success = $success_type;
				fn try_origin(o: O) -> Result<Self::Success, O> {
					o.into().and_then(|o| match o {
						Origin::$name => Ok($success),
						r => Err(O::from(r)),
					})
				}
				#[cfg(feature = "runtime-benchmarks")]
				fn try_successful_origin() -> Result<O, ()> {
					Ok(O::from(Origin::$name))
				}
			}
		};
		( $name:ident ) => { decl_unit_ensures! { $name : () = () } };
		( $name:ident: $success_type:ty = $success:expr, $( $rest:tt )* ) => {
			decl_unit_ensures! { $name: $success_type = $success }
			decl_unit_ensures! { $( $rest )* }
		};
		( $name:ident, $( $rest:tt )* ) => {
			decl_unit_ensures! { $name }
			decl_unit_ensures! { $( $rest )* }
		};
		() => {}
	}
	decl_unit_ensures!(
		StakingAdmin,
		FellowshipAdmin,
		GeneralAdmin,
		ReferendumCanceller,
		ReferendumKiller,
		WhitelistedCaller,
		FellowshipInitiates: u16 = 0,
		Fellows: u16 = 3,
		FellowshipExperts: u16 = 5,
		FellowshipMasters: u16 = 7,
	);

	macro_rules! decl_ensure {
		(
			$vis:vis type $name:ident: EnsureOrigin<Success = $success_type:ty> {
				$( $item:ident = $success:expr, )*
			}
		) => {
			$vis struct $name;
			impl<O: Into<Result<Origin, O>> + From<Origin>>
				EnsureOrigin<O> for $name
			{
				type Success = $success_type;
				fn try_origin(o: O) -> Result<Self::Success, O> {
					o.into().and_then(|o| match o {
						$(
							Origin::$item => Ok($success),
						)*
						r => Err(O::from(r)),
					})
				}
				#[cfg(feature = "runtime-benchmarks")]
				fn try_successful_origin() -> Result<O, ()> {
					// By convention the more privileged origins go later, so for greatest chance
					// of success, we want the last one.
					let _result: Result<O, ()> = Err(());
					$(
						let _result: Result<O, ()> = Ok(O::from(Origin::$item));
					)*
					_result
				}
			}
		}
	}

	decl_ensure! {
		pub type EnsureFellowship: EnsureOrigin<Success = u16> {
			Fellowship1Dan = 1,
			Fellowship2Dan = 2,
			Fellowship3Dan = 3,
			Fellowship4Dan = 4,
			Fellowship5Dan = 5,
			Fellowship6Dan = 6,
			Fellowship7Dan = 7,
			Fellowship8Dan = 8,
			Fellowship9Dan = 9,
		}
	}
}
