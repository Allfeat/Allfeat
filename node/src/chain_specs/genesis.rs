use super::*;
use allfeat_primitives::{AccountId, Balance};
use sp_core::crypto::UncheckedInto;
use sp_runtime::Perbill;
use symphonie_runtime::{
	constants::currency::*, wasm_binary_unwrap, ArtistsConfig, AuthorityDiscoveryConfig,
	BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig, ImOnlineConfig, IndicesConfig,
	MaxNominations, MusicStylesConfig, SessionConfig, StakerStatus, StakingConfig, SudoConfig,
	SystemConfig,
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

	const ENDOWMENT: Balance = 100_000 * DOLLARS;
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

pub fn symphonie_dev_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

pub fn _symphonie_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			(
				hex!["643867ab67490163b0ec8813a294757ab39f6999e3dd83546c2d61a1aed07558"].into(),
				hex!["1c4abecab438dff6f1343065de1e8ba1fc17d21ef9e247c53107b7175982f539"].into(),
				hex!["68396700f406341a36d0a7686deba733c938601998dac68db92ea2ab8f3311f3"]
					.unchecked_into(),
				hex!["34bb8dd25ce0847a558b50c4228476b329b5b79321e471ad0255a336984dd341"]
					.unchecked_into(),
				hex!["aa0ef284b5dcb96e3985654c1defcb16d3ae71b1ed1293b493af53c54d63fd2b"]
					.unchecked_into(),
				hex!["e4ec920e578255c40f5d179f41bd122f5ede8e3adfb7bd54d986e251d1ea4a6d"]
					.unchecked_into(),
			),
			(
				hex!["348bcc84ce42b6863c0e99f8327710f10baacf0252d06e55738408a742bde96c"].into(),
				hex!["d60885354e8e0a8133994bc0b30ff4370bfcc8783c55124c2294914ade39f918"].into(),
				hex!["f002a840f5093c8d9fdf9faad6935d8deba174e13700b9534b227e034abd2e80"]
					.unchecked_into(),
				hex!["34c6f6eb2e72be52dda6974aff76d1bd6dbda3e689a783775baabafb3a25b00f"]
					.unchecked_into(),
				hex!["40d182d90f5ca765a0164ce4a7345bd63e3ba803ad8f140c662cc67aad190975"]
					.unchecked_into(),
				hex!["e065654b8496e584235a4ac5aa63986d567494fd1011b5a0811bdbb742a5ad7c"]
					.unchecked_into(),
			),
		],
		vec![],
		//5DJQgzCnS8BYf7UvjgKzWUeHjAVvaA5wjVvoa6Ni4aZNPfdn
		hex!["36afe14db16edbe5bd861515bbf9d6513980418d83b6c005d63bff6f2e2d706a"].into(),
		Some(vec![
			//5FBmd6CQsTo2KUhkw5KXfP9cFoYB57tPfBoA25WqZPmo26H7
			hex!["36afe14db16edbe5bd861515bbf9d6513980418d83b6c005d63bff6f2e2d706a"].into(),
			//5Fjztu41BFyuYbHjT5SvwJZC2TfWUArHeFmM6B8LqBGG7Vt2
			hex!["a2aca0ac60d20205b88827a78e2c36aac7897630df589e0901b2a272da71e75f"].into(),
		]),
	)
}
