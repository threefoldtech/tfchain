use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{ed25519, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::convert::TryInto;
use tfchain_runtime::opaque::SessionKeys;
use tfchain_runtime::{
    AccountId, AuraConfig, BalancesConfig, CouncilConfig, CouncilMembershipConfig, GenesisConfig,
    GrandpaConfig, SessionConfig, Signature, SmartContractModuleConfig, SudoConfig, SystemConfig,
    TFTBridgeModuleConfig, TFTPriceModuleConfig, TfgridModuleConfig, ValidatorSetConfig,
    WASM_BINARY,
};

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
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_account_id_from_seed_string<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed_string::<TPublic>(seed)).into_account()
}

fn get_from_seed_string<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

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
        move || {
            testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
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
				get_account_id_from_seed_string::<sr25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// bridge validator dev key 2
				get_account_id_from_seed_string::<sr25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// bridge validator dev key 3
				get_account_id_from_seed_string::<sr25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			true,
			vec![
				// bridge validator dev key 1
				get_account_id_from_seed_string::<sr25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// // bridge validator dev key 2
				// get_account_id_from_seed_string::<sr25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// // bridge validator dev key 3
				// get_account_id_from_seed_string::<sr25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			// Bridge fee account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			// TFT price pallet allow account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
            // TFT price pallet min price
            10,
            // TFT price pallet max price
            1000,
            // billing frequency
            10
		)
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        None,
        // Properties
        properties,
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary =
        WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

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
        "TF Chain Tesnet",
        // ID
        "tfchain_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob"),
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
				get_account_id_from_seed_string::<sr25519::Public>("quarter between satisfy three sphere six soda boss cute decade old trend"),
				// bridge validator dev key 2
				get_account_id_from_seed_string::<sr25519::Public>("employ split promote annual couple elder remain cricket company fitness senior fiscal"),
				// bridge validator dev key 3
				get_account_id_from_seed_string::<sr25519::Public>("remind bird banner word spread volume card keep want faith insect mind"),
			],
			// Bridge fee account
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			// TFT price pallet allow account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
            // TFT price pallet min price
            10,
            // TFT price pallet max price
            1000,
            // billing frequency
            5
		)
        },
        // Bootnodes
        vec![],
        None,
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
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    root_key: AccountId,
    foundation_account: AccountId,
    sales_account: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    bridge_validator_accounts: Vec<AccountId>,
    bridge_fee_account: AccountId,
    tft_price_allowed_account: AccountId,
    min_tft_price: u32,
    max_tft_price: u32,
    billing_frequency: u64,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            // changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        validator_set: ValidatorSetConfig {
            initial_validators: initial_authorities
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<_>>(),
        },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.1.clone(), x.2.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        aura: AuraConfig {
            authorities: vec![],
        },
        grandpa: GrandpaConfig {
            authorities: vec![],
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        tfgrid_module: TfgridModuleConfig {
            su_price_value: 100000,
            su_price_unit: 4,
            nu_price_value: 30000,
            nu_price_unit: 4,
            cu_price_value: 200000,
            cu_price_unit: 4,
            ipu_price_value: 50000,
            ipu_price_unit: 4,
            unique_name_price_value: 10000,
            domain_name_price_value: 20000,
            foundation_account: Some(foundation_account),
            sales_account: Some(sales_account),
            farming_policy_diy_cu: 2400,
            farming_policy_diy_su: 1000,
            farming_policy_diy_nu: 30,
            farming_policy_diy_ipu: 5,
            farming_policy_diy_minimal_uptime: 95,
            farming_policy_certified_cu: 3000,
            farming_policy_certified_su: 1250,
            farming_policy_certified_nu: 38,
            farming_policy_certified_ipu: 6,
            farming_policy_certified_minimal_uptime: 95,
            discount_for_dedication_nodes: 50,
            connection_price: 80,
        },
        tft_bridge_module: TFTBridgeModuleConfig {
            validator_accounts: Some(bridge_validator_accounts),
            fee_account: Some(bridge_fee_account),
            deposit_fee: 10000000,
            withdraw_fee: 10000000,
        },
        council: CouncilConfig::default(),
        council_membership: CouncilMembershipConfig {
            members: vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
                get_account_id_from_seed::<sr25519::Public>("Eve"),
            ]
            .try_into()
            .unwrap(),
            phantom: Default::default(),
        },
        // just some default for development
        tft_price_module: TFTPriceModuleConfig {
            allowed_origin: Some(tft_price_allowed_account),
            min_tft_price,
            max_tft_price,
        },
        smart_contract_module: SmartContractModuleConfig {
            billing_frequency: billing_frequency,
        },
    }
}
