use grandpa_primitives::AuthorityId as GrandpaId;
//use hex_literal::hex;
use itertools::Itertools;
pub use node_tidefi_runtime::GenesisConfig;
use node_tidefi_runtime::{
  constants::currency::TIDE, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
  BalancesConfig, CouncilConfig, IndicesConfig, SessionConfig, SessionKeys, StakerStatus,
  StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, TreasuryPalletId,
  WraprAssetRegistryConfig, WraprOracleConfig, WraprQuorumConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
  traits::{AccountIdConversion, IdentifyAccount, Verify},
  Perbill,
};
use tidefi_primitives::{assets, Block, CurrencyId};
pub use tidefi_primitives::{AccountId, Balance, Signature};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str =
  "ws://dedevtidesubstrate-telem.semantic-network.tech:8001/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
  /// Block numbers with known hashes.
  pub fork_blocks: sc_client_api::ForkBlocks<Block>,
  /// Known bad block hashes.
  pub bad_blocks: sc_client_api::BadBlocks<Block>,
  pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub(crate) fn session_keys(
  grandpa: GrandpaId,
  babe: BabeId,
  im_online: ImOnlineId,
  authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
  SessionKeys {
    grandpa,
    babe,
    im_online,
    authority_discovery,
  }
}
/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
  TPublic::Pair::from_string(&format!("//{}", seed), None)
    .expect("static values are valid; qed")
    .public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
  AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
  AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
  seed: &str,
) -> (
  AccountId,
  AccountId,
  GrandpaId,
  BabeId,
  ImOnlineId,
  AuthorityDiscoveryId,
) {
  (
    get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
    get_account_id_from_seed::<sr25519::Public>(seed),
    get_from_seed::<GrandpaId>(seed),
    get_from_seed::<BabeId>(seed),
    get_from_seed::<ImOnlineId>(seed),
    get_from_seed::<AuthorityDiscoveryId>(seed),
  )
}

fn development_config_genesis() -> GenesisConfig {
  testnet_genesis(
    vec![authority_keys_from_seed("Alice")],
    vec![],
    get_account_id_from_seed::<sr25519::Public>("Alice"),
    get_account_id_from_seed::<sr25519::Public>("Alice"),
    get_account_id_from_seed::<sr25519::Public>("Alice"),
  )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
  let mut properties = Map::new();

  // FIXME: Should we set a token symbol? As the other assets are
  // marked with `1.0000 pTIDE` by example in the polkadot UI
  // maybe we can fork and customize a bit the polkadot UI
  //properties.insert("tokenSymbol".into(), "TIDE".into());

  properties.insert("tokenDecimals".into(), 12.into());

  ChainSpec::from_genesis(
    "Development",
    "tidefi_devnet",
    ChainType::Development,
    development_config_genesis,
    vec![],
    None,
    None,
    Some(properties),
    Default::default(),
  )
}

fn testnet_config_genesis() -> GenesisConfig {
  testnet_genesis(
    vec![
      authority_keys_from_seed("Alice"),
      authority_keys_from_seed("Bob"),
    ],
    vec![],
    get_account_id_from_seed::<sr25519::Public>("Alice"),
    get_account_id_from_seed::<sr25519::Public>("Alice"),
    get_account_id_from_seed::<sr25519::Public>("Alice"),
  )
}

/// Testnet config (single validator Alice)
pub fn testnet_config() -> ChainSpec {
  ChainSpec::from_genesis(
    "Local Testnet",
    "tidefi_testnet",
    ChainType::Local,
    testnet_config_genesis,
    vec![],
    Some(
      TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
        .expect("Staging telemetry url is valid; qed"),
    ),
    None,
    None,
    Default::default(),
  )
}

fn adjust_treasury_balance_for_initial_validators(
  initial_validators: usize,
  endowment: u128,
) -> u128 {
  // The extra one is for quorum
  (initial_validators + 1) as u128 * endowment
}

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
  _initial_nominators: Vec<AccountId>,
  quorum: AccountId,
  oracle: AccountId,
  root: AccountId,
) -> GenesisConfig {
  // 20k TIDEs / validators
  const ENDOWMENT: u128 = 20_000 * TIDE;
  const TOTAL_SUPPLY: u128 = 1_000_000_000 * TIDE;
  const STASH: u128 = 2 * TIDE;

  // Stake holders
  let mut claims = get_stakeholder_tokens();
  let mut total_claims: u128 = 0;

  for (_, balance) in &claims {
    total_claims += balance;
  }

  assert_eq!(
    total_claims,
    70_500_000 * TIDE,
    "Total claims is configured correctly"
  );

  let alice_addr = get_account_id_from_seed::<sr25519::Public>("Alice");
  let bob_addr = get_account_id_from_seed::<sr25519::Public>("Bob");
  let eve_addr = get_account_id_from_seed::<sr25519::Public>("Eve");
  let ferdie_addr = get_account_id_from_seed::<sr25519::Public>("Ferdie");

  // Total funds in treasury
  let mut treasury_funds: u128 = TOTAL_SUPPLY;
  treasury_funds -=
    // remove the fund allocated to the validators
    adjust_treasury_balance_for_initial_validators(initial_authorities.len(), ENDOWMENT)
    // all tokens claimed by the stake holders
    + total_claims;

  // Treasury Account Id
  let treasury_account: AccountId = TreasuryPalletId::get().into_account();

  // Each initial validsator get an endowment of `ENDOWMENT` TIDE.
  let mut inital_validators_endowment = initial_authorities
    .iter()
    .map(|k| (k.0.clone(), ENDOWMENT))
    .collect_vec();

  let mut endowed_accounts = vec![
    // Quorum initial endowment
    (quorum.clone(), ENDOWMENT),
    // Treasury funds
    (treasury_account, treasury_funds),
  ];

  // Add all stake holders account
  endowed_accounts.append(claims.as_mut());

  // Endow to validators
  endowed_accounts.append(&mut inital_validators_endowment);

  // FIXME: add vesting to the airdrop pallet
  let _vesting = get_vesting_terms();

  let mut total_supply: u128 = 0;
  for (_, balance) in &endowed_accounts {
    total_supply += *balance
  }

  assert_eq!(
    total_supply, TOTAL_SUPPLY,
    "Total Supply (endowed_accounts) is not equal to 1 billion"
  );

  GenesisConfig {
    system: SystemConfig {
      code: wasm_binary_unwrap().to_vec(),
      changes_trie_config: Default::default(),
    },
    balances: BalancesConfig {
      balances: endowed_accounts.clone(),
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
      minimum_validator_count: 1,
      validator_count: initial_authorities.len() as u32,
      invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
      stakers: initial_authorities
        .iter()
        .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
        .collect(),
      slash_reward_fraction: Perbill::from_percent(10),
      ..Default::default()
    },
    elections: Default::default(),
    council: CouncilConfig {
      members: vec![],
      phantom: Default::default(),
    },
    technical_committee: TechnicalCommitteeConfig {
      members: vec![],
      phantom: Default::default(),
    },

    // FIXME: Remove sudo once the staging is completed
    sudo: SudoConfig { key: root.clone() },

    babe: BabeConfig {
      authorities: Default::default(),
      epoch_config: Some(node_tidefi_runtime::BABE_GENESIS_EPOCH_CONFIG),
    },
    im_online: Default::default(),
    authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
    grandpa: Default::default(),
    technical_membership: Default::default(),
    treasury: Default::default(),
    // tidefi custom genesis
    wrapr_quorum: WraprQuorumConfig {
      enabled: true,
      account: quorum,
    },
    wrapr_oracle: WraprOracleConfig {
      enabled: true,
      account: oracle,
    },
    wrapr_asset_registry: WraprAssetRegistryConfig {
      // these assets are created on first initialization
      assets: vec![
        (
          CurrencyId::Wrapped(assets::BTC),
          "Bitcoin".into(),
          "BTC".into(),
          8,
          vec![
            (alice_addr.clone(), 25000000000000),
            (bob_addr.clone(), 15000000000000),
            (eve_addr.clone(), 10000000000000),
            (ferdie_addr.clone(), 20000000000000),
          ],
        ),
        (
          CurrencyId::Wrapped(assets::ETH),
          "Ethereum".into(),
          "ETH".into(),
          18,
          vec![
            (alice_addr.clone(), 2500000000000000000000000),
            (bob_addr.clone(), 1500000000000000000000000),
            (eve_addr.clone(), 100000000000000000000000),
            (ferdie_addr.clone(), 200000000000000000000000),
          ],
        ),
        (
          CurrencyId::Wrapped(assets::USDC),
          "USD Coin".into(),
          "USDC".into(),
          2,
          vec![
            (alice_addr.clone(), 1125600000),
            (bob_addr.clone(), 122600000),
            (eve_addr.clone(), 112600000),
            (ferdie_addr.clone(), 321600000),
          ],
        ),
        (
          CurrencyId::Wrapped(assets::USDT),
          "Tether".into(),
          "USDT".into(),
          2,
          vec![
            (alice_addr, 2125600000),
            (bob_addr, 522600000),
            (eve_addr, 312600000),
            (ferdie_addr, 121600000),
          ],
        ),
      ],
      // FIXME: Is the asset_registry owner should be the same account as root?
      // this is the owner of the wrapped asset on chain and have full authority on them
      // this account can also create new wrapped asset on chain
      account: root,
    },
    wrapr_security: Default::default(),
    wrapr_fees: Default::default(),
  }
}

pub fn get_vesting_terms() -> Vec<(AccountId, u32, u32, u32, Balance)> {
  // 43800 = minutes in a month
  // 20 blocks / minutes
  // 876_000 blocks / months
  // 2_628_000 / 3 months

  // TODO:
  // who, start, period, period_count, per_period
  // vec![ (hex!["148d5e55a937b6a6c80db86b28bc55f7336b17b13225e80468eef71d01c79341"].into(), 1, 30, 1, 2628000 * TIDE)]
  vec![]
}

// 70_500_000
pub fn get_stakeholder_tokens() -> Vec<(AccountId, Balance)> {
  let bob_addr = get_account_id_from_seed::<sr25519::Public>("Bob");
  let eve_addr = get_account_id_from_seed::<sr25519::Public>("Eve");
  let ferdie_addr = get_account_id_from_seed::<sr25519::Public>("Ferdie");

  let claims = vec![
    (bob_addr, 22_500_000 * TIDE),
    (eve_addr, 23_500_000 * TIDE),
    (ferdie_addr, 24_500_000 * TIDE),
  ];
  claims
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use sp_runtime::BuildStorage;

  fn local_testnet_genesis_instant_single() -> GenesisConfig {
    testnet_genesis(
      vec![authority_keys_from_seed("Alice")],
      vec![],
      get_account_id_from_seed::<sr25519::Public>("Alice"),
      get_account_id_from_seed::<sr25519::Public>("Alice"),
      get_account_id_from_seed::<sr25519::Public>("Alice"),
    )
  }

  /// Local testnet config (single validator - Alice)
  pub fn integration_test_config_with_single_authority() -> ChainSpec {
    ChainSpec::from_genesis(
      "Integration Test",
      "test",
      ChainType::Development,
      local_testnet_genesis_instant_single,
      vec![],
      None,
      None,
      None,
      Default::default(),
    )
  }

  /// Local testnet config (multivalidator Alice + Bob)
  pub fn integration_test_config_with_two_authorities() -> ChainSpec {
    ChainSpec::from_genesis(
      "Integration Test",
      "test",
      ChainType::Development,
      testnet_config_genesis,
      vec![],
      None,
      None,
      None,
      Default::default(),
    )
  }

  #[test]
  fn test_create_development_chain_spec() {
    assert!(!development_config().build_storage().is_err());
  }

  #[test]
  fn test_create_soba_testnet_chain_spec() {
    assert!(!testnet_config().build_storage().is_err());
  }
}
