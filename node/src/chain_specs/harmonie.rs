use super::{
	authority_keys_from_seed, generate_accounts, AuthorityDiscoveryId, BabeId, GrandpaId,
	ImOnlineId,
};
use allfeat_primitives::{AccountId, Balance};
use harmonie_runtime::{
	constants::currency::AFT, opaque::SessionKeys, wasm_binary_unwrap, BabeConfig, BalancesConfig,
	EVMChainIdConfig, ImOnlineConfig, MaxNominations, RuntimeGenesisConfig, SessionConfig,
	StakerStatus, StakingConfig, SudoConfig, SystemConfig,
};
use hex_literal::hex;
use sc_chain_spec::ChainType;
use sp_runtime::Perbill;

use super::Extensions;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

pub fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

/// Generate a chain spec for use with the development service.
pub fn development_chain_spec(mnemonic: Option<String>, num_accounts: Option<u32>) -> ChainSpec {
	// Default mnemonic if none was provided
	let parent_mnemonic = mnemonic.unwrap_or_else(|| {
		"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string()
	});
	// We prefund the standard dev accounts plus Gerald
	let mut accounts = generate_accounts(parent_mnemonic, num_accounts.unwrap_or(10));
	accounts.push(AccountId::from(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")));

	// Prefund the benchmark account for frontier, if compiling for benchmarks
	#[cfg(feature = "runtime-benchmarks")]
	accounts.push(AccountId::from(hex!("1000000000000000000000000000000000000001")));

	ChainSpec::from_genesis(
		"Harmonie Testnet Development",
		"harmonie_live",
		ChainType::Development,
		move || {
			testnet_genesis(vec![authority_keys_from_seed("Alice")], vec![], accounts[0], None, 42)
		},
		vec![],
		None,
		Some("aft"),
		None,
		Some(
			serde_json::json!({
				"isEthereum": true,
				"ss58Format": 42,
				"tokenDecimals": 18,
				"tokenSymbol": "HMY",
			})
			.as_object()
			.expect("Map given; qed")
			.clone(),
		),
		Default::default(),
	)
}

/// Generate a default spec for the parachain service. Use this as a starting point when launching
/// a custom chain.
pub fn get_chain_spec() -> ChainSpec {
	ChainSpec::from_genesis(
		"Harmonie Testnet Live",
		"harmonie_live",
		ChainType::Live,
		move || {
			testnet_genesis(
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				vec![],
				// Alith is Sudo
				AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
				Some(
					// Endowed: Alith, Baltathar, Charleth and Dorothy
					vec![
						AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
						AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
						AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")),
						AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")),
					],
				),
				42,
			)
		},
		vec![],
		None,
		Some("aft"),
		None,
		Some(
			serde_json::json!({
				"isEthereum": true,
				"ss58Format": 42,
				"tokenDecimals": 18,
				"tokenSymbol": "HMY",
			})
			.as_object()
			.expect("Map given; qed")
			.clone(),
		),
		Default::default(),
	)
}

pub fn testnet_genesis(
	initial_authorities: Vec<(
		// Stash
		AccountId,
		// Controller
		AccountId,
		// Session Keys
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	chain_id: u64,
) -> RuntimeGenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")), // Alith
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")), // Baltathar
			AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")), // Charleth
			AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")), // Dorothy
			AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")), // Ethan
			AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")), // Faith
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

	const ENDOWMENT: Balance = 1_000_000 * AFT;
	const STASH: Balance = ENDOWMENT / 10;

	RuntimeGenesisConfig {
		system: SystemConfig { code: wasm_binary_unwrap().to_vec(), ..Default::default() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
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
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		sudo: SudoConfig { key: Some(root_key) },
		babe: BabeConfig {
			epoch_config: Some(harmonie_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: Default::default(),
		grandpa: Default::default(),
		transaction_payment: Default::default(),
		nomination_pools: Default::default(),
		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id, ..Default::default() },
		evm: Default::default(),
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
	}
}
