use frame_support::PalletId;
use grandpa_primitives::AuthorityId as GrandpaId;
use hex_literal::hex;
use itertools::Itertools;
use log::info;
pub use node_tidefi_runtime::GenesisConfig;
use node_tidefi_runtime::{
  constants::currency::TIDE, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
  BalancesConfig, CouncilConfig, IndicesConfig, QuorumConfig, SessionConfig, SessionKeys,
  StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
  traits::{AccountIdConversion, IdentifyAccount, Verify},
  Perbill,
};
use tidefi_primitives::Block;
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
  )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
  ChainSpec::from_genesis(
    "Development",
    "tidefi_devnet",
    ChainType::Development,
    development_config_genesis,
    vec![],
    None,
    None,
    None,
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
) -> GenesisConfig {
  const ENDOWMENT: u128 = 20_000 * TIDE;
  const STASH: u128 = 2 * TIDE;
  // Total funds in treasury also includes 2_000_000 TIDE for parachain auctions
  let mut treasury_funds: u128 = 10_200_000 * TIDE;
  treasury_funds -=
    adjust_treasury_balance_for_initial_validators(initial_authorities.len(), ENDOWMENT);

  info!(
    "👷‍♂️ Tokens taken from treasury:  {:>22}",
    adjust_treasury_balance_for_initial_validators(initial_authorities.len(), ENDOWMENT)
  );
  info!("👷‍♂️ Token remaining in treasury: {:>22}", treasury_funds);
  // Treasury Account Id
  pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");
  let treasury_account: AccountId = TREASURY_PALLET_ID.into_account();

  let mut inital_validators_endowment = initial_authorities
    .iter()
    .map(|k| (k.0.clone(), ENDOWMENT))
    .collect_vec();
  let mut endowed_accounts = vec![
    //      Quorum account
    (quorum.clone(), ENDOWMENT),
    //     Treasury Funds
    (treasury_account, treasury_funds),
  ];
  // Get rest of the stake holders
  let mut claims = get_stakeholder_tokens();

  let mut total_claims: u128 = 0;
  for (_, balance) in &claims {
    total_claims += balance;
  }

  info!("👷‍♂️ Total Investor Tokens:       {:>22}", total_claims);
  // assert_eq!(total_claims, 6_627_105 * TIDE, "Total claims is configured correctly");

  endowed_accounts.append(claims.as_mut());
  // Endow to validators
  endowed_accounts.append(&mut inital_validators_endowment);

  let mut total_supply: u128 = 0;
  for (_, balance) in &endowed_accounts {
    total_supply += *balance
  }

  info!(
    "👷‍♂️  Assert Total supply is 20 million: {} == {} ",
    total_supply,
    20_000_000 * TIDE
  );

  let vesting = get_vesting_terms();

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
    // FIXME: Should the quorum stay the sudo?
    sudo: SudoConfig {
      key: quorum.clone(),
    },
    babe: BabeConfig {
      authorities: Default::default(),
      epoch_config: Some(node_tidefi_runtime::BABE_GENESIS_EPOCH_CONFIG),
    },
    im_online: Default::default(),
    authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
    grandpa: Default::default(),
    technical_membership: Default::default(),
    treasury: Default::default(),
    quorum: QuorumConfig {
      quorum_enabled: true,
      quorum_account: quorum,
    },
  }
}

pub fn get_vesting_terms() -> Vec<(AccountId, u32, u32, u32, Balance)> {
  // 3 months in terms of 12s blocks is 648,000 blocks, i.e. period = 648,000
  // TODO:
  // who, start, period, period_count, per_period
  // vec![ (hex!["148d5e55a937b6a6c80db86b28bc55f7336b17b13225e80468eef71d01c79341"].into(), 1, 30, 1, 3655828 * TIDE)]
  vec![]
}

pub fn get_stakeholder_tokens() -> Vec<(AccountId, Balance)> {
  let claims = vec![(
    hex!["e4cdc8abc0405db44c1a6886a2f2c59012fa3b98c07b61d63cc7f9e437ba243e"].into(),
    3 * 6_000 * TIDE,
  )];
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
