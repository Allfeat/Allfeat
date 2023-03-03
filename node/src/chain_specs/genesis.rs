use super::*;
use allfeat_primitives::{AccountId, Balance};
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

pub fn symphonie_dev_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![authority_keys_from_seed("Alice")],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

fn _symphonie_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			(
				//5F6jrk6r15QvJPf7PvDESkZKpVnuc54oQsGKUFQvp2QKHWrQ
				hex!["8641ed9a03e0e4e679a2d36dfee489353a3a117a1830e6b4b5c4388ec7569e70"].into(),
				//5FNC5HnVzZwntjVJiNnA4999kAvjaym5F5E1uxGtCu2wULAL
				hex!["920adc764a443e5cd284de8c8cd3757539a801f1130e10412e94b2af7bc9492a"].into(),
				//5Eo8wbEB8zJb7wbGLMXcoAeYAZda5BLFyzzhgJkVMLAWDSgs
				hex!["78d537921b129fc335828edf3e50427c09087aea2231be87ec16dd10474817ae"]
					.unchecked_into(),
				//5G12PQX7EoQbmsPhMoDBeZpbKVUArk28Tf5iZ5PcESEW8wTN
				hex!["ae2254603f8c217c246f2af1cf1565caac39c3c2c37e5fc3360f03961607e230"]
					.unchecked_into(),
				//5DZLhGUd9MqMrGvx7GrwP4p5YagrqNDTMUJxTG7SVbufYhJz
				hex!["42131cbbd5420a035447ab28417eb783cdbf323c485cc9ba4b6a06b28f5cc464"]
					.unchecked_into(),
				//5Cns2WNjbAfHzJUZjsPYCXeqCPm8aXVrCtscAbYbJ3DRLVHC
				hex!["20272cb9b2b1629d1729bb6db5e702b36db50130059ef2005f444d62a6771260"]
					.unchecked_into(),
			),
			(
				//5HBURdeyRQoRAXqyJELrvwL4b6T4D3xfoSD52hERPfbLgy8j
				hex!["e256ab8cba4cf4c694057f846c203291dcb49ca105f906e2ea86a0d3a05f102a"].into(),
				//5ENqGczpMdfQMjbM1Loon8W9np9udhp5rc96K2iCJxhjMQKJ
				hex!["664bd6a760a9669d12662d569436c662e1f5ce0ad29ec7220ca47c4c72e75e17"].into(),
				//5FcWoNNkpa1GxQSc1HAnr3vRfbg22HVncQTcebpWnKzwKbsv
				hex!["9cf7551ed4f32981e562c4aeee0ae8a352bf4924b1b59497c2d380bee30018cc"]
					.unchecked_into(),
				//5HRC18W3EwxL8oWkPMvfNv2PADS7ajS5QTe1yXGvYZR3w3p1
				hex!["ecccd2a64fdfc10cc3a0d8aefea9cc4793dd11cf4fef0384c4d641a24e741c67"]
					.unchecked_into(),
				//5HEZAEdGzRCA6yhhyNj58dMSoD47W9nsbQ5kPvpWLAbgsEW7
				hex!["e4b0581cd0d33cae2d28855f1a44a7bef94060935d4aea13e2db37b02fd3ba69"]
					.unchecked_into(),
				//5CqJt7Bu2Vz7osbYxSxwV8HQReZ2sdr4fo7CAqeAWGSQxN8V
				hex!["2204b3a91d9c5c449a9d1d9305f1a031461c38205063571708f9ff613224c447"]
					.unchecked_into(),
			),
		],
		vec![],
		hex!["8a18186617e63cf6fb3e7aa1a5569c2964f7bf7b0fccbc684048bdbfc5260f5d"].into(),
		Some(vec![
			//5FBmd6CQsTo2KUhkw5KXfP9cFoYB57tPfBoA25WqZPmo26H7
			hex!["8a18186617e63cf6fb3e7aa1a5569c2964f7bf7b0fccbc684048bdbfc5260f5d"].into(),
			//5Fjztu41BFyuYbHjT5SvwJZC2TfWUArHeFmM6B8LqBGG7Vt2
			hex!["a2aca0ac60d20205b88827a78e2c36aac7897630df589e0901b2a272da71e75f"].into(),
		]),
	)
}
