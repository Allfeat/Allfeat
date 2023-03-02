use super::*;
use allfeat_primitives::{AccountId, Balance};
use sp_runtime::Perbill;
use symphonie_runtime::{
	constants::currency::*, wasm_binary_unwrap, ArtistsConfig, AuthorityDiscoveryConfig,
	BabeConfig, BalancesConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig, MaxNominations,
	MusicStylesConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig, SystemConfig,
};

use crate::chain_specs::helpers::{get_account_id_from_seed, session_keys};

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		]
	});
	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let _num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 1_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 10;

	GenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: 3u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		music_styles: MusicStylesConfig {
			styles: vec![(
				b"Test Style".to_vec(),
				vec![b"Test Sub".to_vec(), b"Test Sub 2".to_vec()],
			)],
			phantom: Default::default(),
		},
		artists: ArtistsConfig { artists: Default::default(), candidates: Default::default() },
		assets: Default::default(),
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(symphonie_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		transaction_payment: Default::default(),
		nomination_pools: Default::default(),
	}
}
