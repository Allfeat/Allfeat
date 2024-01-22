use super::{
	authority_keys_from_seed, generate_accounts, AuthorityDiscoveryId, BabeId, Extensions,
	GrandpaId, ImOnlineId,
};
use allfeat_primitives::{AccountId, Balance};
use harmonie_runtime::{
	constants::currency::AFT, opaque::SessionKeys, wasm_binary_unwrap, MaxNominations,
	RuntimeGenesisConfig, StakerStatus,
};
use hex_literal::hex;
use sc_chain_spec::ChainType;
use sp_runtime::Perbill;

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

	ChainSpec::builder(wasm_binary_unwrap(), Default::default())
		.with_name("Harmonie Testnet Development")
		.with_id("harmonie_live")
		.with_chain_type(ChainType::Development)
		.with_genesis_config_patch(testnet_genesis(
			vec![authority_keys_from_seed("Alice")],
			vec![],
			accounts[0],
			None,
			42,
		))
		.with_protocol_id("aft")
		.with_properties(
			serde_json::json!({
				"isEthereum": true,
				"ss58Format": 42,
				"tokenDecimals": 18,
				"tokenSymbol": "HMY",
			})
			.as_object()
			.expect("Map given; qed")
			.clone(),
		)
		.build()
}

/// Generate a default spec for the parachain service. Use this as a starting point when launching
/// a custom chain.
pub fn get_chain_spec() -> ChainSpec {
	ChainSpec::builder(wasm_binary_unwrap(), Default::default())
		.with_name("Harmonie Testnet Live")
		.with_id("harmonie_live")
		.with_chain_type(ChainType::Live)
		.with_genesis_config_patch(testnet_genesis(
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
		))
		.with_protocol_id("aft")
		.with_properties(
			serde_json::json!({
				"isEthereum": true,
				"ss58Format": 42,
				"tokenDecimals": 18,
				"tokenSymbol": "HMY",
			})
			.as_object()
			.expect("Map given; qed")
			.clone(),
		)
		.build()
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
) -> serde_json::Value {
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

	const ENDOWMENT: Balance = 10_000_000 * AFT;
	const STASH: Balance = ENDOWMENT / 1000;

	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect::<Vec<_>>(),
		},
		"session": {
			"keys": initial_authorities
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
		"staking": {
			"validatorCount": initial_authorities.len() as u32,
			"minimumValidatorCount": initial_authorities.len() as u32,
			"invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
			"slashRewardFraction": Perbill::from_percent(10),
			"stakers": stakers.clone(),
		},
		"sudo": { "key": Some(root_key.clone()) },
		"babe": {
			"epochConfig": Some(harmonie_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		// EVM compatibility
		"evmChainId": { "chainId": chain_id },
	})
}
