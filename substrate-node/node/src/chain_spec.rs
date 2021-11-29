use sp_core::{Pair, Public, sr25519, ed25519};
use tfchain_runtime::{
	AccountId, BabeConfig, BalancesConfig, 
	GenesisConfig, GrandpaConfig, ImOnlineId,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, 
	TfgridModuleConfig, TFTBridgeModuleConfig, SessionConfig,
	AuthorityDiscoveryConfig, AuthorityDiscoveryId, StakingConfig,
	StakerStatus,
};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;
use tfchain_runtime::opaque::SessionKeys;
use tfchain_runtime::constants;
// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_account_id_from_seed_string<TPublic: Public>(seed: &str) -> AccountId where
AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed_string::<TPublic>(seed)).into_account()
}

fn get_from_seed_string<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

fn get_session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
	) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, authority_discovery }
}

/// Generate an authority keys.
pub fn get_authority_keys_from_seed(seed: &str)
	-> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	let properties = Some(
		serde_json::json!({
			"tokenDecimals": 7,
			"tokenSymbol": "TFT",
		})
		.as_object()
		.expect("Map given; qed")
		.clone(),
	);

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial validators
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Foundation account
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			// Sales account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				// bridge validator dev key 1
				get_account_id_from_seed_string::<ed25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// bridge validator dev key 2
				get_account_id_from_seed_string::<ed25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// bridge validator dev key 3
				get_account_id_from_seed_string::<ed25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			true,
			vec![
				// bridge validator dev key 1
				get_account_id_from_seed_string::<ed25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// // bridge validator dev key 2
				// get_account_id_from_seed_string::<ed25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// // bridge validator dev key 3
				// get_account_id_from_seed_string::<ed25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			// Bridge fee account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			// Staking Pool Account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		properties,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	let properties = Some(
		serde_json::json!({
			"tokenDecimals": 7,
			"tokenSymbol": "TFT",
		})
		.as_object()
		.expect("Map given; qed")
		.clone(),
	);

	Ok(ChainSpec::from_genesis(
		// Name
		"TF Chain Local Testnet",
		// ID
		"tfchain_testnet",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial validators
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Foundation account
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			// Sales account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			// Pre-funded accounts
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
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				// bridge validator dev key 1
				get_account_id_from_seed_string::<ed25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// // bridge validator dev key 2
				// get_account_id_from_seed_string::<ed25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// // bridge validator dev key 3
				// get_account_id_from_seed_string::<ed25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			true,
			vec![
				// bridge validator dev key 1
				get_account_id_from_seed_string::<ed25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// bridge validator dev key 2
				get_account_id_from_seed_string::<ed25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// bridge validator dev key 3
				get_account_id_from_seed_string::<ed25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			// Bridge fee account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			// Staking Pool Account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		properties,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	root_key: AccountId,
	foundation_account: AccountId,
	sales_account: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
	bridge_validator_accounts: Vec<AccountId>,
	bridge_fee_account: AccountId,
	staking_pool_account: AccountId,
) -> GenesisConfig {
	const INITIAL_STAKING: u128 = 1_000_000 * constants::currency::CENTS;
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (
						x.0.clone(), // stash
						x.0.clone(), // stash
						get_session_keys(
							x.2.clone(), // grandpa
							x.3.clone(), // babe
							x.4.clone(), // im-online
							x.5.clone(), // authority-discovery
						)))
				.collect::<Vec<_>>(),
		}),
		pallet_staking: Some(StakingConfig {
			staking_pool_account,
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),
		pallet_im_online: Default::default(),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
		pallet_tfgrid: Some(TfgridModuleConfig {
			su_price_value: 300000,
			su_price_unit: 4,
			nu_price_value: 2000,
			nu_price_unit: 4,
			cu_price_value: 600000,
			cu_price_unit: 4,
			ipu_price_value: 100000,
			ipu_price_unit: 4,
			unique_name_price_value: 20000,
			domain_name_price_value: 40000,
			foundation_account,
			sales_account,
			farming_policy_diy_cu: 160000000,
			farming_policy_diy_su: 100000000,
			farming_policy_diy_nu: 2000000,
			farming_policy_diy_ipu: 800000,
			farming_policy_certified_cu: 200000000,
			farming_policy_certified_su: 120000000,
			farming_policy_certified_nu: 3000000,
			farming_policy_certified_ipu: 1000000
		}),
		pallet_tft_bridge: Some(TFTBridgeModuleConfig{
			validator_accounts: bridge_validator_accounts,
			fee_account: bridge_fee_account,
			deposit_fee: 10000000,
			burn_fee: 10000000
		})
	}
}