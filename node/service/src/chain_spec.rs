use codec::{Decode, Encode};
use hex_literal::hex;
use itertools::Itertools;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{blake2_256, crypto::UncheckedInto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
  traits::{AccountIdConversion, IdentifyAccount, Verify},
  Perbill,
};
use strum::IntoEnumIterator;
// Tidechain primitives
use tidefi_primitives::{assets, AccountId, AssetId, Balance, Block, CurrencyId, Signature};

#[cfg(feature = "tidechain-native")]
const TIDECHAIN_STAGING_TELEMETRY_URL: &str = "wss://telemetry.tidefi.io/submit/";

#[cfg(feature = "hertel-native")]
const HERTEL_STAGING_TELEMETRY_URL: &str = "wss://telemetry.tidefi.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, sc_chain_spec::ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
  /// Block numbers with known hashes.
  pub fork_blocks: sc_client_api::ForkBlocks<Block>,
  /// Known bad block hashes.
  pub bad_blocks: sc_client_api::BadBlocks<Block>,
  /// Required for Tidechain and Hertel Runtime, for future light-client implementation.
  pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

#[cfg(feature = "tidechain-native")]
pub type TidechainChainSpec =
  sc_service::GenericChainSpec<tidechain_runtime::GenesisConfig, Extensions>;

#[cfg(feature = "hertel-native")]
pub type HertelChainSpec = sc_service::GenericChainSpec<hertel_runtime::GenesisConfig, Extensions>;

#[cfg(feature = "tidechain-native")]
pub fn tidechain_config() -> Result<TidechainChainSpec, String> {
  TidechainChainSpec::from_json_bytes(&include_bytes!("../res/tidechain.json")[..])
}

#[cfg(feature = "hertel-native")]
pub fn hertel_config() -> Result<HertelChainSpec, String> {
  HertelChainSpec::from_json_bytes(&include_bytes!("../res/hertel.json")[..])
}

#[cfg(feature = "hertel-native")]
fn hertel_session_keys(
  grandpa: GrandpaId,
  babe: BabeId,
  im_online: ImOnlineId,
  authority_discovery: AuthorityDiscoveryId,
) -> hertel_runtime::SessionKeys {
  hertel_runtime::SessionKeys {
    grandpa,
    babe,
    im_online,
    authority_discovery,
  }
}

#[cfg(feature = "tidechain-native")]
fn tidechain_session_keys(
  grandpa: GrandpaId,
  babe: BabeId,
  im_online: ImOnlineId,
  authority_discovery: AuthorityDiscoveryId,
) -> tidechain_runtime::SessionKeys {
  tidechain_runtime::SessionKeys {
    grandpa,
    babe,
    im_online,
    authority_discovery,
  }
}

#[cfg(feature = "hertel-native")]
fn hertel_testnet_genesis(
  wasm_binary: &[u8],
  initial_authorities: Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
  )>,
  stakeholders: Vec<(CurrencyId, AccountId, Balance)>,
  quorums: Vec<AccountId>,
  oracle: AccountId,
  root: AccountId,
  assets: Vec<(AssetId, Vec<u8>, Vec<u8>, u8)>,
) -> hertel_runtime::GenesisConfig {
  // 1000 TIDEs / validators
  const ENDOWMENT: u128 = 1000 * 1_000_000_000_000;
  const TOTAL_SUPPLY: u128 = 1_000_000_000 * 1_000_000_000_000;
  const STASH: u128 = 2 * 1_000_000_000_000;
  // Get Quorum Account ID (if multisig)
  let quorum = helpers::get_quorum_address(quorums.clone());
  // Treasury Account Id
  let treasury_account: AccountId = hertel_runtime::TreasuryPalletId::get().into_account();
  // Fees Account Id
  let fees_account: AccountId = hertel_runtime::FeesPalletId::get().into_account();
  // Get all TIDE from our stakeholders
  let mut claims = helpers::get_tide_from_stakeholders(stakeholders.clone());

  let mut total_claims: u128 = 0;
  for (_, balance) in &claims {
    total_claims += balance;
  }

  // Total funds in treasury
  let mut treasury_funds: u128 = TOTAL_SUPPLY;
  treasury_funds -=
  // remove the fund allocated to the validators and quorums
  helpers::adjust_treasury_balance_for_initial_validators_and_quorums(initial_authorities.len(), quorums.len(), ENDOWMENT)
  // all tokens claimed by the stake holders
  + total_claims
  // 10 tide endowed to the fee pallet 
  + 10_000_000_000_000
  // 10 tide endowed to root
  + 10_000_000_000_000;

  // Each initial validator get an endowment of `ENDOWMENT` TIDE.
  let mut inital_validators_endowment = initial_authorities
    .iter()
    .map(|k| (k.0.clone(), ENDOWMENT))
    .collect_vec();

  // Each quorums get an endowment of `ENDOWMENT` TIDE.
  let mut inital_quorums_endowment = quorums.iter().map(|k| (k.clone(), ENDOWMENT)).collect_vec();

  let mut endowed_accounts = vec![
    // Treasury funds
    (treasury_account, treasury_funds),
    // 10 tide to make sure the fees pallet can receive funds
    (fees_account, 10_000_000_000_000),
    // 10 tide to root so he can pay fees
    (root.clone(), 10_000_000_000_000),
  ];

  // Add all stake holders account
  endowed_accounts.append(&mut claims);

  // Endow to validators
  endowed_accounts.append(&mut inital_validators_endowment);

  // Endow to quorums
  endowed_accounts.append(&mut inital_quorums_endowment);

  let mut total_supply: u128 = 0;
  for (_, balance) in &endowed_accounts {
    total_supply += *balance
  }

  assert_eq!(
    total_supply, TOTAL_SUPPLY,
    "Total Supply (endowed_accounts) is not equal to 1 billion"
  );

  hertel_runtime::GenesisConfig {
    system: hertel_runtime::SystemConfig {
      code: wasm_binary.to_vec(),
    },
    balances: hertel_runtime::BalancesConfig {
      balances: endowed_accounts.clone(),
    },

    indices: hertel_runtime::IndicesConfig { indices: vec![] },
    session: hertel_runtime::SessionConfig {
      keys: initial_authorities
        .iter()
        .map(|x| {
          (
            x.0.clone(),
            x.0.clone(),
            hertel_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
          )
        })
        .collect::<Vec<_>>(),
    },
    staking: hertel_runtime::StakingConfig {
      minimum_validator_count: 1,
      validator_count: initial_authorities.len() as u32,
      invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
      stakers: initial_authorities
        .iter()
        .map(|x| {
          (
            x.0.clone(),
            x.1.clone(),
            STASH,
            hertel_runtime::StakerStatus::Validator,
          )
        })
        .collect(),
      slash_reward_fraction: Perbill::from_percent(10),
      ..Default::default()
    },
    elections: Default::default(),
    council: hertel_runtime::CouncilConfig {
      members: vec![],
      phantom: Default::default(),
    },
    technical_committee: hertel_runtime::TechnicalCommitteeConfig {
      members: vec![],
      phantom: Default::default(),
    },

    // FIXME: Remove sudo once the staging is completed
    sudo: hertel_runtime::SudoConfig {
      key: Some(root.clone()),
    },

    babe: hertel_runtime::BabeConfig {
      authorities: Default::default(),
      epoch_config: Some(hertel_runtime::BABE_GENESIS_EPOCH_CONFIG),
    },
    im_online: Default::default(),
    authority_discovery: hertel_runtime::AuthorityDiscoveryConfig { keys: vec![] },
    grandpa: Default::default(),
    technical_membership: Default::default(),
    treasury: Default::default(),
    // tidefi custom genesis
    wrapr_quorum: hertel_runtime::WraprQuorumConfig {
      enabled: true,
      account: quorum,
    },
    wrapr_oracle: hertel_runtime::WraprOracleConfig {
      enabled: true,
      account: oracle,
    },
    wrapr_asset_registry: hertel_runtime::WraprAssetRegistryConfig {
      // these assets are created on first initialization
      assets: helpers::get_assets_with_stakeholders(stakeholders, assets),
      // FIXME: Is the asset_registry owner should be the same account as root?
      // this is the owner of the wrapped asset on chain and have full authority on them
      // this account can also create new wrapped asset on chain
      account: root,
    },
    wrapr_security: Default::default(),
    wrapr_fees: Default::default(),
  }
}

#[cfg(feature = "tidechain-native")]
fn tidechain_testnet_genesis(
  wasm_binary: &[u8],
  initial_authorities: Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
  )>,
  stakeholders: Vec<(CurrencyId, AccountId, Balance)>,
  quorums: Vec<AccountId>,
  oracle: AccountId,
  assets: Vec<(AssetId, Vec<u8>, Vec<u8>, u8)>,
) -> tidechain_runtime::GenesisConfig {
  // 1000 TIDEs / validators
  const ENDOWMENT: u128 = 1000 * 1_000_000_000_000;
  const TOTAL_SUPPLY: u128 = 1_000_000_000 * 1_000_000_000_000;
  const STASH: u128 = 2 * 1_000_000_000_000;
  // Get Quorum Account ID (if multisig)
  let quorum = helpers::get_quorum_address(quorums.clone());
  // Treasury Account Id
  let treasury_account: AccountId = tidechain_runtime::TreasuryPalletId::get().into_account();
  // Fees Account Id
  let fees_account: AccountId = tidechain_runtime::FeesPalletId::get().into_account();
  // Asset registry Account Id
  let asset_registry: AccountId = tidechain_runtime::AssetRegistryPalletId::get().into_account();
  // Get all TIDE from our stakeholders
  let mut claims = helpers::get_tide_from_stakeholders(stakeholders.clone());

  let mut total_claims: u128 = 0;
  for (_, balance) in &claims {
    total_claims += balance;
  }

  // Total funds in treasury
  let mut treasury_funds: u128 = TOTAL_SUPPLY;
  treasury_funds -=
  // remove the fund allocated to the validators and quorums
  helpers::adjust_treasury_balance_for_initial_validators_and_quorums(initial_authorities.len(), quorums.len(), ENDOWMENT)
  // all tokens claimed by the stake holders
  + total_claims
  // 1 tide endowed to the fee pallet 
  + 1_000_000_000_000;

  // Each initial validator get an endowment of `ENDOWMENT` TIDE.
  let mut inital_validators_endowment = initial_authorities
    .iter()
    .map(|k| (k.0.clone(), ENDOWMENT))
    .collect_vec();

  // Each quorums get an endowment of `ENDOWMENT` TIDE.
  let mut inital_quorums_endowment = quorums.iter().map(|k| (k.clone(), ENDOWMENT)).collect_vec();

  let mut endowed_accounts = vec![
    // Treasury funds
    (treasury_account, treasury_funds),
    // 1 tide to make sure the fees pallet can receive funds
    (fees_account, 1_000_000_000_000),
  ];

  // Add all stake holders account
  endowed_accounts.append(&mut claims);

  // Endow to validators
  endowed_accounts.append(&mut inital_validators_endowment);

  // Endow to quorums
  endowed_accounts.append(&mut inital_quorums_endowment);

  let mut total_supply: u128 = 0;
  for (_, balance) in &endowed_accounts {
    total_supply += *balance
  }

  assert_eq!(
    total_supply, TOTAL_SUPPLY,
    "Total Supply (endowed_accounts) is not equal to 1 billion"
  );

  tidechain_runtime::GenesisConfig {
    system: tidechain_runtime::SystemConfig {
      code: wasm_binary.to_vec(),
    },
    balances: tidechain_runtime::BalancesConfig {
      balances: endowed_accounts.clone(),
    },

    indices: tidechain_runtime::IndicesConfig { indices: vec![] },
    session: tidechain_runtime::SessionConfig {
      keys: initial_authorities
        .iter()
        .map(|x| {
          (
            x.0.clone(),
            x.0.clone(),
            tidechain_session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
          )
        })
        .collect::<Vec<_>>(),
    },
    staking: tidechain_runtime::StakingConfig {
      minimum_validator_count: 1,
      validator_count: initial_authorities.len() as u32,
      invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
      stakers: initial_authorities
        .iter()
        .map(|x| {
          (
            x.0.clone(),
            x.1.clone(),
            STASH,
            tidechain_runtime::StakerStatus::Validator,
          )
        })
        .collect(),
      slash_reward_fraction: Perbill::from_percent(10),
      ..Default::default()
    },
    elections: Default::default(),
    council: tidechain_runtime::CouncilConfig {
      members: vec![],
      phantom: Default::default(),
    },
    technical_committee: tidechain_runtime::TechnicalCommitteeConfig {
      members: vec![],
      phantom: Default::default(),
    },

    babe: tidechain_runtime::BabeConfig {
      authorities: Default::default(),
      epoch_config: Some(tidechain_runtime::BABE_GENESIS_EPOCH_CONFIG),
    },
    im_online: Default::default(),
    authority_discovery: tidechain_runtime::AuthorityDiscoveryConfig { keys: vec![] },
    grandpa: Default::default(),
    technical_membership: Default::default(),
    treasury: Default::default(),
    // tidefi custom genesis
    wrapr_quorum: tidechain_runtime::WraprQuorumConfig {
      enabled: true,
      account: quorum,
    },
    wrapr_oracle: tidechain_runtime::WraprOracleConfig {
      enabled: true,
      account: oracle,
    },
    wrapr_asset_registry: tidechain_runtime::WraprAssetRegistryConfig {
      // these assets are created on first initialization
      assets: helpers::get_assets_with_stakeholders(stakeholders, assets),
      // FIXME: Not sure if the owner should be the asset registry pallet itself?
      account: asset_registry,
    },
    wrapr_security: Default::default(),
    wrapr_fees: Default::default(),
  }
}

/// Development config (single validator Alice)
#[cfg(feature = "hertel-native")]
pub fn hertel_development_config() -> Result<HertelChainSpec, String> {
  let mut properties = Map::new();

  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());

  let wasm_binary = hertel_runtime::WASM_BINARY.ok_or("Hertel development wasm not available")?;

  Ok(HertelChainSpec::from_genesis(
    "Development",
    "hertel_dev",
    ChainType::Development,
    move || hertel_development_config_genesis(wasm_binary),
    vec![],
    None,
    None,
    Some(properties),
    Default::default(),
  ))
}

/// Hertel local testnet config.
#[cfg(feature = "hertel-native")]
pub fn hertel_local_testnet_config() -> Result<HertelChainSpec, String> {
  let mut properties = Map::new();

  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());
  let wasm_binary = hertel_runtime::WASM_BINARY.ok_or("Hertel development wasm not available")?;

  let boot_nodes = vec![];

  Ok(HertelChainSpec::from_genesis(
    "Development",
    "hertel_dev",
    ChainType::Local,
    move || hertel_local_testnet_config_genesis(wasm_binary),
    boot_nodes,
    None,
    None,
    Some(properties),
    Default::default(),
  ))
}

/// Hertel staging testnet config.
#[cfg(feature = "hertel-native")]
pub fn hertel_staging_testnet_config() -> Result<HertelChainSpec, String> {
  let mut properties = Map::new();

  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());
  let wasm_binary = hertel_runtime::WASM_BINARY.ok_or("Hertel development wasm not available")?;

  let boot_nodes = vec![];

  Ok(HertelChainSpec::from_genesis(
    "Development",
    "hertel_dev",
    ChainType::Development,
    move || hertel_staging_testnet_config_genesis(wasm_binary),
    boot_nodes,
    Some(
      TelemetryEndpoints::new(vec![(HERTEL_STAGING_TELEMETRY_URL.to_string(), 0)])
        .expect("Discovery Staging telemetry url is valid; qed"),
    ),
    None,
    Some(properties),
    Default::default(),
  ))
}

/// Development config (single validator Alice)
#[cfg(feature = "tidechain-native")]
pub fn tidechain_development_config() -> Result<TidechainChainSpec, String> {
  let mut properties = Map::new();

  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());

  let wasm_binary =
    tidechain_runtime::WASM_BINARY.ok_or("Tidechain development wasm not available")?;

  Ok(TidechainChainSpec::from_genesis(
    "Development",
    "tidechain_dev",
    ChainType::Development,
    move || tidechain_development_config_genesis(wasm_binary),
    vec![],
    None,
    None,
    Some(properties),
    Default::default(),
  ))
}

/// Tidechain local testnet config.
#[cfg(feature = "tidechain-native")]
pub fn tidechain_staging_testnet_config() -> Result<TidechainChainSpec, String> {
  let mut properties = Map::new();

  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());

  let wasm_binary =
    tidechain_runtime::WASM_BINARY.ok_or("Tidechain development wasm not available")?;

  let boot_nodes = vec![];

  Ok(TidechainChainSpec::from_genesis(
    "Development",
    "tidechain_local",
    ChainType::Local,
    move || tidechain_staging_testnet_config_genesis(wasm_binary),
    boot_nodes,
    Some(
      TelemetryEndpoints::new(vec![(TIDECHAIN_STAGING_TELEMETRY_URL.to_string(), 0)])
        .expect("Tidechain Staging telemetry url is valid; qed"),
    ),
    None,
    Some(properties),
    Default::default(),
  ))
}

/// Tidechain staging testnet config.
#[cfg(feature = "tidechain-native")]
pub fn tidechain_local_testnet_config() -> Result<TidechainChainSpec, String> {
  let mut properties = Map::new();

  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());

  let wasm_binary =
    tidechain_runtime::WASM_BINARY.ok_or("Tidechain development wasm not available")?;

  let boot_nodes = vec![];

  Ok(TidechainChainSpec::from_genesis(
    "Development",
    "tidechain_local_testnet",
    ChainType::Development,
    move || tidechain_local_testnet_config_genesis(wasm_binary),
    boot_nodes,
    None,
    None,
    Some(properties),
    Default::default(),
  ))
}

#[cfg(feature = "hertel-native")]
fn hertel_development_config_genesis(wasm_binary: &[u8]) -> hertel_runtime::GenesisConfig {
  hertel_testnet_genesis(
    wasm_binary,
    vec![helpers::authority_keys_from_seed("Alice")],
    helpers::get_stakeholder_tokens_hertel(),
    vec![helpers::get_account_id_from_seed::<sr25519::Public>(
      "Charlie",
    )],
    helpers::get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    helpers::get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    helpers::get_all_assets(),
  )
}

#[cfg(feature = "hertel-native")]
fn hertel_local_testnet_config_genesis(wasm_binary: &[u8]) -> hertel_runtime::GenesisConfig {
  hertel_testnet_genesis(
    wasm_binary,
    vec![
      helpers::authority_keys_from_seed("Alice"),
      helpers::authority_keys_from_seed("Bob"),
    ],
    helpers::get_stakeholder_tokens_hertel(),
    vec![
      helpers::get_account_id_from_seed::<sr25519::Public>("Charlie"),
      helpers::get_account_id_from_seed::<sr25519::Public>("Dave"),
      helpers::get_account_id_from_seed::<sr25519::Public>("Eve"),
    ],
    helpers::get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    helpers::get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    helpers::get_all_assets(),
  )
}

#[cfg(feature = "hertel-native")]
fn hertel_staging_testnet_config_genesis(wasm_binary: &[u8]) -> hertel_runtime::GenesisConfig {
  let initial_authorities: Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
  )> = vec![
    (
      //5EPH4xUAqCbTPw3kfjqewELdrCEQiNp84vuhiVg8hU1A5wX7
      hex!["66a2af6ffba20fa9b410b0cb7e2eb1d99fb3748066210e9e806ba0e824b8de70"].into(),
      //5DF43vX2vamASYjah4u9gNDF95oq3Yb2NPPseua3cfFfTtta
      hex!["3420aaae4b1e1d2ad0abc953083681b9552aec63231d5f8da73d92954a086d2e"].into(),
      //5FN2w1sbZtiJSm4eK6sKfyDSz42uo4g7vN61PhxWBVnCN3KA
      hex!["91ec1590b039fff51d37525cdf599dad78e73349365085f8219bcac459f371da"].unchecked_into(),
      //5H1PrCT66bGdQDzG6CB73ryrxmuCidWBExpeUXGvZ4jzMZ5R
      hex!["daa6cf7287fe8e3bc7feab159409fef2aff2297f64086b0e8fe5a122642b1311"].unchecked_into(),
      //5HTEc4VSMRDcatHFBgrVRZURAJZykSXJ149C7GGBCRHVWjxo
      hex!["ee5c12d9accd02f7b9c41076205b5536e145d87fc4d8e9bb3e59ac5ebf923727"].unchecked_into(),
      //5DJF1CUe6zM3Ux6UvJm895K43ksaomwarhjuX3mVeLm319ci
      hex!["368f462a8a00121449647ad3f7224e61dd4a7a93d678ab739c91910c2f8e2d56"].unchecked_into(),
    ),
    (
      //5Gn8KDvn6ZfcqG4s5WBLbH4bARS77nFrA738tHaNrGkESUb9
      hex!["d08853ecd641605a685c827ead7521b178725a06d80c1ba6131a6fc87b6b9b13"].into(),
      //5Dcq84dRFew2R11X6vXK4Zn6VPbtxzkTtvbpXMMjTmZpyh4k
      hex!["44bc8b430298b88630d178b3d2237879a3202ebaaa65966b3f63a57d5c9f9b24"].into(),
      //5DrLpQS5hd7yLBAcn1pmMHUAQY4m1GAPQh7qs1rrnupac3WQ
      hex!["4f0ab2c75b9b2a2ccdccb1ba566149f06afb7e19bd58af68959f8162452d7385"].unchecked_into(),
      //5FWgrDY4HbGFgnERStbfPLxi12vAmTXbo8aLYjLpnwx2mnSN
      hex!["9885b037ad8e0df9076f69d2a3d501f0c2d0e4a2ed31b65c6a7d437679ec3658"].unchecked_into(),
      //5FHfAgjpQ6pjDG56JCC2sJ58JqGpo23d8CPUVR1dECpyZFwf
      hex!["8e95d661acebbc44fa4fa7824b98ba5cd34b9f4b9f421d1e4e7f248b5e87cc10"].unchecked_into(),
      //5GYitpB3bB4FTTdxN2mBZwxaRa4Na3zAAUqND1nD3x1PxLES
      hex!["c64f4bc3852a7122902c5cacd7777fe1b04b6e16131026788e6ea23f18912c2a"].unchecked_into(),
    ),
    (
      //5Ge8JkHNACxSVR9vNpirDmLNHBVCPqhybEe61BSKsgimEEgr
      hex!["ca6e56414a58383dbc87c281f7e320f3ebdf59233b9544c898294701e9b13f77"].into(),
      //5GxQPxE4D5YrLteyLmVqhtYwpHR5gCx8XHhrrqC7qUB7U53r
      hex!["d85eeacb2f51ac029a9ebcd14e218f869d19ba83f9dac23e8064d809d49f2e20"].into(),
      //5HiusCdwsa28RgCs15ouuJvjPseTyk8RyKyJyhDjhQy8ekNa
      hex!["fa50eb4d60acc5a1a0537b21ae93117063a7148f172265d70f0137caf129aef0"].unchecked_into(),
      //5DvyAENVNcHm5wjqmPVpCPSkdKfGTT8zk8DnxU88KTt3fbsk
      hex!["5292047acf6707b86e8f2c80ec1efa8c40e4a11c01c43cebaf29ab8cf6b6d271"].unchecked_into(),
      //5DPJ9G65fDeoaTFZqwpE8r2X27zHvQ6mNqASkvhSsRDAmywc
      hex!["3a6a116479e5f6c1b6aa4b6e25eff90d30d6d528a9c29bbd72e353930604ae7e"].unchecked_into(),
      //5F4dorWaEpsESMAR1yZijhvQkV5iD1FtBUMcBEBZESnc7mPK
      hex!["84a7126293f444a53e68567d8f259aae4895e1d8991a929900c4866eeb8f8966"].unchecked_into(),
    ),
  ];

  // quorums
  let quorums: Vec<AccountId> = vec![
    //5EFKNPG2kPsyeVK8E5e7i5uiRfYdbQkq8qfhVxeVV42tZfPe
    hex!["60907755938c5ee6561ee929a766cb42cfbce19b19619c3b89adc30cf9cd970b"].into(),
    //5HVb1QTxnzHXpTPLCVT61Ag3Mb4fmyMYAy3kxbXYXMS9KjM6
    hex!["f0273ecee5c89e91c9baee61755498a40885133d0f5ee7ee4b4f035aa1551e53"].into(),
    //5EA2mLbbbdq6cyqDwZuHEGvKPPBVWDNuCS3DwtaetAum9aSe
    hex!["5c88582258ab5c02f342cd3ff37601252953cad2fb04de192cab2e2656788a6e"].into(),
  ];

  hertel_testnet_genesis(
    wasm_binary,
    initial_authorities,
    helpers::get_stakeholder_tokens_hertel(),
    quorums,
    //5HKDZMoz5NnX37Np8dMKMAANbNu9N1XuQec15b3tZ8NaBTAR
    hex!["e83e965a0e2c599751184bcea1507d9fe37510d9d75eb37cba3ad8c1a5a1fe12"].into(),
    //5Hp9T9DoHRmLXsZ6j85R7xxqmUxCZ7MS4pfi4C6W6og484G6
    hex!["fe4ee0c4bae7d8a058b478c48bbaeab5e9b9d6adccacc49a45796dfb02bd9908"].into(),
    helpers::get_all_assets(),
  )
}

#[cfg(feature = "tidechain-native")]
fn tidechain_development_config_genesis(wasm_binary: &[u8]) -> tidechain_runtime::GenesisConfig {
  tidechain_testnet_genesis(
    wasm_binary,
    vec![helpers::authority_keys_from_seed("Alice")],
    helpers::get_stakeholder_tokens_tidechain(),
    vec![helpers::get_account_id_from_seed::<sr25519::Public>("Bob")],
    helpers::get_account_id_from_seed::<sr25519::Public>("Charlie"),
    helpers::get_all_assets(),
  )
}

#[cfg(feature = "tidechain-native")]
fn tidechain_staging_testnet_config_genesis(
  wasm_binary: &[u8],
) -> tidechain_runtime::GenesisConfig {
  let initial_authorities: Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
  )> = vec![
    (
      //5EPH4xUAqCbTPw3kfjqewELdrCEQiNp84vuhiVg8hU1A5wX7
      hex!["66a2af6ffba20fa9b410b0cb7e2eb1d99fb3748066210e9e806ba0e824b8de70"].into(),
      //5DF43vX2vamASYjah4u9gNDF95oq3Yb2NPPseua3cfFfTtta
      hex!["3420aaae4b1e1d2ad0abc953083681b9552aec63231d5f8da73d92954a086d2e"].into(),
      //5FN2w1sbZtiJSm4eK6sKfyDSz42uo4g7vN61PhxWBVnCN3KA
      hex!["91ec1590b039fff51d37525cdf599dad78e73349365085f8219bcac459f371da"].unchecked_into(),
      //5H1PrCT66bGdQDzG6CB73ryrxmuCidWBExpeUXGvZ4jzMZ5R
      hex!["daa6cf7287fe8e3bc7feab159409fef2aff2297f64086b0e8fe5a122642b1311"].unchecked_into(),
      //5HTEc4VSMRDcatHFBgrVRZURAJZykSXJ149C7GGBCRHVWjxo
      hex!["ee5c12d9accd02f7b9c41076205b5536e145d87fc4d8e9bb3e59ac5ebf923727"].unchecked_into(),
      //5DJF1CUe6zM3Ux6UvJm895K43ksaomwarhjuX3mVeLm319ci
      hex!["368f462a8a00121449647ad3f7224e61dd4a7a93d678ab739c91910c2f8e2d56"].unchecked_into(),
    ),
    (
      //5Gn8KDvn6ZfcqG4s5WBLbH4bARS77nFrA738tHaNrGkESUb9
      hex!["d08853ecd641605a685c827ead7521b178725a06d80c1ba6131a6fc87b6b9b13"].into(),
      //5Dcq84dRFew2R11X6vXK4Zn6VPbtxzkTtvbpXMMjTmZpyh4k
      hex!["44bc8b430298b88630d178b3d2237879a3202ebaaa65966b3f63a57d5c9f9b24"].into(),
      //5DrLpQS5hd7yLBAcn1pmMHUAQY4m1GAPQh7qs1rrnupac3WQ
      hex!["4f0ab2c75b9b2a2ccdccb1ba566149f06afb7e19bd58af68959f8162452d7385"].unchecked_into(),
      //5FWgrDY4HbGFgnERStbfPLxi12vAmTXbo8aLYjLpnwx2mnSN
      hex!["9885b037ad8e0df9076f69d2a3d501f0c2d0e4a2ed31b65c6a7d437679ec3658"].unchecked_into(),
      //5FHfAgjpQ6pjDG56JCC2sJ58JqGpo23d8CPUVR1dECpyZFwf
      hex!["8e95d661acebbc44fa4fa7824b98ba5cd34b9f4b9f421d1e4e7f248b5e87cc10"].unchecked_into(),
      //5GYitpB3bB4FTTdxN2mBZwxaRa4Na3zAAUqND1nD3x1PxLES
      hex!["c64f4bc3852a7122902c5cacd7777fe1b04b6e16131026788e6ea23f18912c2a"].unchecked_into(),
    ),
    (
      //5Ge8JkHNACxSVR9vNpirDmLNHBVCPqhybEe61BSKsgimEEgr
      hex!["ca6e56414a58383dbc87c281f7e320f3ebdf59233b9544c898294701e9b13f77"].into(),
      //5GxQPxE4D5YrLteyLmVqhtYwpHR5gCx8XHhrrqC7qUB7U53r
      hex!["d85eeacb2f51ac029a9ebcd14e218f869d19ba83f9dac23e8064d809d49f2e20"].into(),
      //5HiusCdwsa28RgCs15ouuJvjPseTyk8RyKyJyhDjhQy8ekNa
      hex!["fa50eb4d60acc5a1a0537b21ae93117063a7148f172265d70f0137caf129aef0"].unchecked_into(),
      //5DvyAENVNcHm5wjqmPVpCPSkdKfGTT8zk8DnxU88KTt3fbsk
      hex!["5292047acf6707b86e8f2c80ec1efa8c40e4a11c01c43cebaf29ab8cf6b6d271"].unchecked_into(),
      //5DPJ9G65fDeoaTFZqwpE8r2X27zHvQ6mNqASkvhSsRDAmywc
      hex!["3a6a116479e5f6c1b6aa4b6e25eff90d30d6d528a9c29bbd72e353930604ae7e"].unchecked_into(),
      //5F4dorWaEpsESMAR1yZijhvQkV5iD1FtBUMcBEBZESnc7mPK
      hex!["84a7126293f444a53e68567d8f259aae4895e1d8991a929900c4866eeb8f8966"].unchecked_into(),
    ),
  ];

  // quorums
  let quorums: Vec<AccountId> = vec![
    //5EFKNPG2kPsyeVK8E5e7i5uiRfYdbQkq8qfhVxeVV42tZfPe
    hex!["60907755938c5ee6561ee929a766cb42cfbce19b19619c3b89adc30cf9cd970b"].into(),
    //5HVb1QTxnzHXpTPLCVT61Ag3Mb4fmyMYAy3kxbXYXMS9KjM6
    hex!["f0273ecee5c89e91c9baee61755498a40885133d0f5ee7ee4b4f035aa1551e53"].into(),
    //5EA2mLbbbdq6cyqDwZuHEGvKPPBVWDNuCS3DwtaetAum9aSe
    hex!["5c88582258ab5c02f342cd3ff37601252953cad2fb04de192cab2e2656788a6e"].into(),
  ];

  tidechain_testnet_genesis(
    wasm_binary,
    initial_authorities,
    helpers::get_stakeholder_tokens_tidechain(),
    quorums,
    //5HKDZMoz5NnX37Np8dMKMAANbNu9N1XuQec15b3tZ8NaBTAR
    hex!["e83e965a0e2c599751184bcea1507d9fe37510d9d75eb37cba3ad8c1a5a1fe12"].into(),
    helpers::get_all_assets(),
  )
}

#[cfg(feature = "tidechain-native")]
fn tidechain_local_testnet_config_genesis(wasm_binary: &[u8]) -> tidechain_runtime::GenesisConfig {
  let initial_authorities: Vec<(
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
  )> = vec![
    (
      //5EPH4xUAqCbTPw3kfjqewELdrCEQiNp84vuhiVg8hU1A5wX7
      hex!["66a2af6ffba20fa9b410b0cb7e2eb1d99fb3748066210e9e806ba0e824b8de70"].into(),
      //5DF43vX2vamASYjah4u9gNDF95oq3Yb2NPPseua3cfFfTtta
      hex!["3420aaae4b1e1d2ad0abc953083681b9552aec63231d5f8da73d92954a086d2e"].into(),
      //5FN2w1sbZtiJSm4eK6sKfyDSz42uo4g7vN61PhxWBVnCN3KA
      hex!["91ec1590b039fff51d37525cdf599dad78e73349365085f8219bcac459f371da"].unchecked_into(),
      //5H1PrCT66bGdQDzG6CB73ryrxmuCidWBExpeUXGvZ4jzMZ5R
      hex!["daa6cf7287fe8e3bc7feab159409fef2aff2297f64086b0e8fe5a122642b1311"].unchecked_into(),
      //5HTEc4VSMRDcatHFBgrVRZURAJZykSXJ149C7GGBCRHVWjxo
      hex!["ee5c12d9accd02f7b9c41076205b5536e145d87fc4d8e9bb3e59ac5ebf923727"].unchecked_into(),
      //5DJF1CUe6zM3Ux6UvJm895K43ksaomwarhjuX3mVeLm319ci
      hex!["368f462a8a00121449647ad3f7224e61dd4a7a93d678ab739c91910c2f8e2d56"].unchecked_into(),
    ),
    (
      //5Gn8KDvn6ZfcqG4s5WBLbH4bARS77nFrA738tHaNrGkESUb9
      hex!["d08853ecd641605a685c827ead7521b178725a06d80c1ba6131a6fc87b6b9b13"].into(),
      //5Dcq84dRFew2R11X6vXK4Zn6VPbtxzkTtvbpXMMjTmZpyh4k
      hex!["44bc8b430298b88630d178b3d2237879a3202ebaaa65966b3f63a57d5c9f9b24"].into(),
      //5DrLpQS5hd7yLBAcn1pmMHUAQY4m1GAPQh7qs1rrnupac3WQ
      hex!["4f0ab2c75b9b2a2ccdccb1ba566149f06afb7e19bd58af68959f8162452d7385"].unchecked_into(),
      //5FWgrDY4HbGFgnERStbfPLxi12vAmTXbo8aLYjLpnwx2mnSN
      hex!["9885b037ad8e0df9076f69d2a3d501f0c2d0e4a2ed31b65c6a7d437679ec3658"].unchecked_into(),
      //5FHfAgjpQ6pjDG56JCC2sJ58JqGpo23d8CPUVR1dECpyZFwf
      hex!["8e95d661acebbc44fa4fa7824b98ba5cd34b9f4b9f421d1e4e7f248b5e87cc10"].unchecked_into(),
      //5GYitpB3bB4FTTdxN2mBZwxaRa4Na3zAAUqND1nD3x1PxLES
      hex!["c64f4bc3852a7122902c5cacd7777fe1b04b6e16131026788e6ea23f18912c2a"].unchecked_into(),
    ),
    (
      //5Ge8JkHNACxSVR9vNpirDmLNHBVCPqhybEe61BSKsgimEEgr
      hex!["ca6e56414a58383dbc87c281f7e320f3ebdf59233b9544c898294701e9b13f77"].into(),
      //5GxQPxE4D5YrLteyLmVqhtYwpHR5gCx8XHhrrqC7qUB7U53r
      hex!["d85eeacb2f51ac029a9ebcd14e218f869d19ba83f9dac23e8064d809d49f2e20"].into(),
      //5HiusCdwsa28RgCs15ouuJvjPseTyk8RyKyJyhDjhQy8ekNa
      hex!["fa50eb4d60acc5a1a0537b21ae93117063a7148f172265d70f0137caf129aef0"].unchecked_into(),
      //5DvyAENVNcHm5wjqmPVpCPSkdKfGTT8zk8DnxU88KTt3fbsk
      hex!["5292047acf6707b86e8f2c80ec1efa8c40e4a11c01c43cebaf29ab8cf6b6d271"].unchecked_into(),
      //5DPJ9G65fDeoaTFZqwpE8r2X27zHvQ6mNqASkvhSsRDAmywc
      hex!["3a6a116479e5f6c1b6aa4b6e25eff90d30d6d528a9c29bbd72e353930604ae7e"].unchecked_into(),
      //5F4dorWaEpsESMAR1yZijhvQkV5iD1FtBUMcBEBZESnc7mPK
      hex!["84a7126293f444a53e68567d8f259aae4895e1d8991a929900c4866eeb8f8966"].unchecked_into(),
    ),
  ];

  // quorums
  let quorums: Vec<AccountId> = vec![
    //5EFKNPG2kPsyeVK8E5e7i5uiRfYdbQkq8qfhVxeVV42tZfPe
    hex!["60907755938c5ee6561ee929a766cb42cfbce19b19619c3b89adc30cf9cd970b"].into(),
    //5HVb1QTxnzHXpTPLCVT61Ag3Mb4fmyMYAy3kxbXYXMS9KjM6
    hex!["f0273ecee5c89e91c9baee61755498a40885133d0f5ee7ee4b4f035aa1551e53"].into(),
    //5EA2mLbbbdq6cyqDwZuHEGvKPPBVWDNuCS3DwtaetAum9aSe
    hex!["5c88582258ab5c02f342cd3ff37601252953cad2fb04de192cab2e2656788a6e"].into(),
  ];

  tidechain_testnet_genesis(
    wasm_binary,
    initial_authorities,
    helpers::get_stakeholder_tokens_tidechain(),
    quorums,
    //5HKDZMoz5NnX37Np8dMKMAANbNu9N1XuQec15b3tZ8NaBTAR
    hex!["e83e965a0e2c599751184bcea1507d9fe37510d9d75eb37cba3ad8c1a5a1fe12"].into(),
    helpers::get_all_assets(),
  )
}

// helpers for our genesis configuration
mod helpers {
  use super::*;
  type AccountPublic = <Signature as Verify>::Signer;

  /// Helper function to generate a crypto pair from seed
  pub(crate) fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
      .expect("static values are valid; qed")
      .public()
  }

  /// Helper function to generate an account ID from seed
  pub(crate) fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
  where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
  {
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
  }

  /// Helper function to generate stash, controller and session key from seed
  pub(crate) fn authority_keys_from_seed(
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

  pub(crate) fn get_tide_from_stakeholders(
    stakeholders: Vec<(CurrencyId, AccountId, Balance)>,
  ) -> Vec<(AccountId, Balance)> {
    stakeholders
      .into_iter()
      .filter(|(currency_id, _, _)| *currency_id == CurrencyId::Tide)
      .map(|(_, account_id, balance)| (account_id, balance))
      .collect()
  }

  pub(crate) fn get_assets_with_stakeholders(
    stakeholders: Vec<(CurrencyId, AccountId, Balance)>,
    assets: Vec<(AssetId, Vec<u8>, Vec<u8>, u8)>,
  ) -> Vec<(CurrencyId, Vec<u8>, Vec<u8>, u8, Vec<(AccountId, Balance)>)> {
    assets
      .iter()
      .map(|(asset_id, asset_name, asset_symbol, decimals)| {
        let all_endowed_accounts = stakeholders
          .clone()
          .into_iter()
          .filter(|(currency_id, _, _)| *currency_id == CurrencyId::Wrapped(*asset_id))
          .map(|(_, account_id, balance)| (account_id, balance))
          .collect();
        (
          CurrencyId::Wrapped(*asset_id),
          asset_name.clone(),
          asset_symbol.clone(),
          *decimals,
          all_endowed_accounts,
        )
      })
      .collect()
  }

  pub(crate) fn get_quorum_address(quorums: Vec<AccountId>) -> AccountId {
    if quorums.len() > 1 {
      // threshold (60%)
      let threshold = (quorums.len() as f64 * 0.6).ceil() as u16;
      // create multisig from the quorum accounts provided
      let mut signatories = quorums;
      signatories.sort();
      let entropy = (b"modlpy/utilisuba", &signatories, threshold).using_encoded(blake2_256);
      AccountId::decode(&mut &entropy[..]).unwrap_or_default()
    } else {
      quorums.first().unwrap().clone()
    }
  }

  pub fn get_all_assets() -> Vec<(AssetId, Vec<u8>, Vec<u8>, u8)> {
    assets::Asset::iter()
      .filter(|asset| asset.id() != assets::TIDE)
      .map(|asset| {
        (
          asset.id(),
          asset.name().into(),
          asset.symbol().into(),
          asset.exponent(),
        )
      })
      .collect()
  }

  pub(crate) fn adjust_treasury_balance_for_initial_validators_and_quorums(
    initial_validators: usize,
    initial_quorums: usize,
    endowment: u128,
  ) -> u128 {
    // Validators + quorums
    (initial_validators + initial_quorums) as u128 * endowment
  }

  // SECRET="key" ./scripts/prepare-dev-hertel.sh
  #[cfg(feature = "hertel-native")]
  pub fn get_stakeholder_tokens_hertel() -> Vec<(CurrencyId, AccountId, Balance)> {
    vec![
      // faucet
      (
        CurrencyId::Tide,
        //5HQD3Nj89oWfh1ZgooiMDYFvS1bWHNmvtfok7krVQZyv1Hst
        hex!["ec0d12f5a230e4c757dfa1cc786ca0b86bdce58dc7c67436e67a606c3bd93735"].into(),
        // 10_000 TIDE
        assets::Asset::Tide.saturating_mul(10_000),
      ),
      // investors
      (
        CurrencyId::Tide,
        //5DJ3bpy5LB5jsCFCCig3ayHHLFmY9KByYUmFUTQiu1t4KLEV
        hex!["3668e36adabd3b469202d415106c82729d74f967cd412a29dc09dcf45b62fd02"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5DFPWrUPp2mTWTdX6BXARVwML4BvkR33TFoXu4W8MKRJXZqv
        hex!["3462307f866e7ed4bd83a3a0d3f533bc1258fce78a50d6fdb02254d0fac86033"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5DqsUjGivLn2jiPF4cmwnE7vuNZeW5b3w5FvqBeKYkZzccKU
        hex!["4eaeaa9358b864fa1d8e4ace39107324c3f76e74fb80ff0ce7db91bdbbc3a218"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5Exi5fGsVGK8NvCULcS7muFiRmTQKG7wHsFPwhbXXaT3Adb7
        hex!["8021fb6321b727d4157c3ac86b4557dfefa35490bfcfe583f1a55148a65f9118"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5EfQRF5MKSc4Ae4A8N1rVTToxPixXzcBtXiaGGbNPXUKU7z6
        hex!["72ef5a70cf4007e94e92d8ff916a611665354b9919af98ea3a938962a955613a"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      // devs
      (
        assets::Asset::Tide.currency_id(),
        //5CDBQosadsb88FyTgLaK9JkrPney4KCXWKKQpXbRr6vMGfzM
        hex!["0676b7ff90098c566721f404c12f7160319b9032261660ef2fcec61710c9e177"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5CDBQosadsb88FyTgLaK9JkrPney4KCXWKKQpXbRr6vMGfzM
        hex!["0676b7ff90098c566721f404c12f7160319b9032261660ef2fcec61710c9e177"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5CDBQosadsb88FyTgLaK9JkrPney4KCXWKKQpXbRr6vMGfzM
        hex!["0676b7ff90098c566721f404c12f7160319b9032261660ef2fcec61710c9e177"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5CDBQosadsb88FyTgLaK9JkrPney4KCXWKKQpXbRr6vMGfzM
        hex!["0676b7ff90098c566721f404c12f7160319b9032261660ef2fcec61710c9e177"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5CDBQosadsb88FyTgLaK9JkrPney4KCXWKKQpXbRr6vMGfzM
        hex!["0676b7ff90098c566721f404c12f7160319b9032261660ef2fcec61710c9e177"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5DaX6GxeTKAU6uu949hvtJwpfhZkmYSiTmuwbmF7KvqLDUG4
        hex!["42f95b258e1ba467fb73babae8d23fc6c9985f207109a60a071d3db1e1e97214"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5DaX6GxeTKAU6uu949hvtJwpfhZkmYSiTmuwbmF7KvqLDUG4
        hex!["42f95b258e1ba467fb73babae8d23fc6c9985f207109a60a071d3db1e1e97214"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5DaX6GxeTKAU6uu949hvtJwpfhZkmYSiTmuwbmF7KvqLDUG4
        hex!["42f95b258e1ba467fb73babae8d23fc6c9985f207109a60a071d3db1e1e97214"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5DaX6GxeTKAU6uu949hvtJwpfhZkmYSiTmuwbmF7KvqLDUG4
        hex!["42f95b258e1ba467fb73babae8d23fc6c9985f207109a60a071d3db1e1e97214"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5DaX6GxeTKAU6uu949hvtJwpfhZkmYSiTmuwbmF7KvqLDUG4
        hex!["42f95b258e1ba467fb73babae8d23fc6c9985f207109a60a071d3db1e1e97214"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5CooWAL2vwGMbmSXJRmbaNGB7H5sHk2Vk4jfLZgomNpVjTrk
        hex!["20de8f59b70e0eb1ae4344d6f649b0e9668089a3831cf4827dd7fe21fc85067b"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5CooWAL2vwGMbmSXJRmbaNGB7H5sHk2Vk4jfLZgomNpVjTrk
        hex!["20de8f59b70e0eb1ae4344d6f649b0e9668089a3831cf4827dd7fe21fc85067b"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5CooWAL2vwGMbmSXJRmbaNGB7H5sHk2Vk4jfLZgomNpVjTrk
        hex!["20de8f59b70e0eb1ae4344d6f649b0e9668089a3831cf4827dd7fe21fc85067b"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5CooWAL2vwGMbmSXJRmbaNGB7H5sHk2Vk4jfLZgomNpVjTrk
        hex!["20de8f59b70e0eb1ae4344d6f649b0e9668089a3831cf4827dd7fe21fc85067b"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5CooWAL2vwGMbmSXJRmbaNGB7H5sHk2Vk4jfLZgomNpVjTrk
        hex!["20de8f59b70e0eb1ae4344d6f649b0e9668089a3831cf4827dd7fe21fc85067b"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5HWQJXxKpDK4m4bBSchFSRcrfsjvxzUbM5mZFUkxDcmUxJVP
        hex!["f0c674a34591219d583388dbc2d1233eb700387866fb95f49f50e901dbfea21b"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5HWQJXxKpDK4m4bBSchFSRcrfsjvxzUbM5mZFUkxDcmUxJVP
        hex!["f0c674a34591219d583388dbc2d1233eb700387866fb95f49f50e901dbfea21b"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5HWQJXxKpDK4m4bBSchFSRcrfsjvxzUbM5mZFUkxDcmUxJVP
        hex!["f0c674a34591219d583388dbc2d1233eb700387866fb95f49f50e901dbfea21b"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5HWQJXxKpDK4m4bBSchFSRcrfsjvxzUbM5mZFUkxDcmUxJVP
        hex!["f0c674a34591219d583388dbc2d1233eb700387866fb95f49f50e901dbfea21b"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5HWQJXxKpDK4m4bBSchFSRcrfsjvxzUbM5mZFUkxDcmUxJVP
        hex!["f0c674a34591219d583388dbc2d1233eb700387866fb95f49f50e901dbfea21b"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5D58VZMkYurpZe2cg6k9MsMvG3Zx47HSWXprHn3SGaoZT2BZ
        hex!["2c8f2a6da01abdd82c05b2336b9df2ff3bfb4f04a519b2c830259659784b7b14"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5D58VZMkYurpZe2cg6k9MsMvG3Zx47HSWXprHn3SGaoZT2BZ
        hex!["2c8f2a6da01abdd82c05b2336b9df2ff3bfb4f04a519b2c830259659784b7b14"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5D58VZMkYurpZe2cg6k9MsMvG3Zx47HSWXprHn3SGaoZT2BZ
        hex!["2c8f2a6da01abdd82c05b2336b9df2ff3bfb4f04a519b2c830259659784b7b14"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5D58VZMkYurpZe2cg6k9MsMvG3Zx47HSWXprHn3SGaoZT2BZ
        hex!["2c8f2a6da01abdd82c05b2336b9df2ff3bfb4f04a519b2c830259659784b7b14"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5D58VZMkYurpZe2cg6k9MsMvG3Zx47HSWXprHn3SGaoZT2BZ
        hex!["2c8f2a6da01abdd82c05b2336b9df2ff3bfb4f04a519b2c830259659784b7b14"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5DURQrTvagqJmv4kZgxTyQCwXUMhpxASFhre6cyhcQ2MSzXx
        hex!["3e52c399382bbb8cb1378d278fd7f9f894fe0e93876b2f95304bca36919a4c32"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5DURQrTvagqJmv4kZgxTyQCwXUMhpxASFhre6cyhcQ2MSzXx
        hex!["3e52c399382bbb8cb1378d278fd7f9f894fe0e93876b2f95304bca36919a4c32"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5DURQrTvagqJmv4kZgxTyQCwXUMhpxASFhre6cyhcQ2MSzXx
        hex!["3e52c399382bbb8cb1378d278fd7f9f894fe0e93876b2f95304bca36919a4c32"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5DURQrTvagqJmv4kZgxTyQCwXUMhpxASFhre6cyhcQ2MSzXx
        hex!["3e52c399382bbb8cb1378d278fd7f9f894fe0e93876b2f95304bca36919a4c32"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5DURQrTvagqJmv4kZgxTyQCwXUMhpxASFhre6cyhcQ2MSzXx
        hex!["3e52c399382bbb8cb1378d278fd7f9f894fe0e93876b2f95304bca36919a4c32"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5DcQ7QKtTt5FByXcMj39UmYY4Yd42jxwFLk6CVPP6E2BnGc2
        hex!["4468592112f141475ee850b31d5ba7f7b3e08a425657c19bc5d7b8591c3b6503"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5DcQ7QKtTt5FByXcMj39UmYY4Yd42jxwFLk6CVPP6E2BnGc2
        hex!["4468592112f141475ee850b31d5ba7f7b3e08a425657c19bc5d7b8591c3b6503"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5DcQ7QKtTt5FByXcMj39UmYY4Yd42jxwFLk6CVPP6E2BnGc2
        hex!["4468592112f141475ee850b31d5ba7f7b3e08a425657c19bc5d7b8591c3b6503"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5DcQ7QKtTt5FByXcMj39UmYY4Yd42jxwFLk6CVPP6E2BnGc2
        hex!["4468592112f141475ee850b31d5ba7f7b3e08a425657c19bc5d7b8591c3b6503"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5DcQ7QKtTt5FByXcMj39UmYY4Yd42jxwFLk6CVPP6E2BnGc2
        hex!["4468592112f141475ee850b31d5ba7f7b3e08a425657c19bc5d7b8591c3b6503"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5DRy1PxmxQEbpLtZ7Fhtwkp9FhjyZqSEq6wJ7QvJjDEv3Tpj
        hex!["3c736354d5c79a8ca7bfb27ab8ca708d7ed223f47b6af10688a053400f605c11"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5DRy1PxmxQEbpLtZ7Fhtwkp9FhjyZqSEq6wJ7QvJjDEv3Tpj
        hex!["3c736354d5c79a8ca7bfb27ab8ca708d7ed223f47b6af10688a053400f605c11"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5DRy1PxmxQEbpLtZ7Fhtwkp9FhjyZqSEq6wJ7QvJjDEv3Tpj
        hex!["3c736354d5c79a8ca7bfb27ab8ca708d7ed223f47b6af10688a053400f605c11"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5DRy1PxmxQEbpLtZ7Fhtwkp9FhjyZqSEq6wJ7QvJjDEv3Tpj
        hex!["3c736354d5c79a8ca7bfb27ab8ca708d7ed223f47b6af10688a053400f605c11"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5DRy1PxmxQEbpLtZ7Fhtwkp9FhjyZqSEq6wJ7QvJjDEv3Tpj
        hex!["3c736354d5c79a8ca7bfb27ab8ca708d7ed223f47b6af10688a053400f605c11"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5F3rR7FQs7HHJahUPBAB5LnmiavhExs2Hv4NwYishrosYPub
        hex!["840e44a5783d856de632cb7730f844ed1286b40e7cd32f9ffad6a783367bee0e"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5F3rR7FQs7HHJahUPBAB5LnmiavhExs2Hv4NwYishrosYPub
        hex!["840e44a5783d856de632cb7730f844ed1286b40e7cd32f9ffad6a783367bee0e"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5F3rR7FQs7HHJahUPBAB5LnmiavhExs2Hv4NwYishrosYPub
        hex!["840e44a5783d856de632cb7730f844ed1286b40e7cd32f9ffad6a783367bee0e"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5F3rR7FQs7HHJahUPBAB5LnmiavhExs2Hv4NwYishrosYPub
        hex!["840e44a5783d856de632cb7730f844ed1286b40e7cd32f9ffad6a783367bee0e"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5F3rR7FQs7HHJahUPBAB5LnmiavhExs2Hv4NwYishrosYPub
        hex!["840e44a5783d856de632cb7730f844ed1286b40e7cd32f9ffad6a783367bee0e"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tide.currency_id(),
        //5EeP7WRUrXPkmNobee22ZYSFcgssASW6tZpBE6jSsJoXobHg
        hex!["7227b654418bbd07626d09091d4d8accf0c3a3aac5a1d9fbfdc42e7f55670e62"].into(),
        // 1_000_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Tether.currency_id(),
        //5EeP7WRUrXPkmNobee22ZYSFcgssASW6tZpBE6jSsJoXobHg
        hex!["7227b654418bbd07626d09091d4d8accf0c3a3aac5a1d9fbfdc42e7f55670e62"].into(),
        // 1_000_000 USDT
        assets::Asset::Tether.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::USDCoin.currency_id(),
        //5EeP7WRUrXPkmNobee22ZYSFcgssASW6tZpBE6jSsJoXobHg
        hex!["7227b654418bbd07626d09091d4d8accf0c3a3aac5a1d9fbfdc42e7f55670e62"].into(),
        // 1_000_000 USDC
        assets::Asset::USDCoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Bitcoin.currency_id(),
        //5EeP7WRUrXPkmNobee22ZYSFcgssASW6tZpBE6jSsJoXobHg
        hex!["7227b654418bbd07626d09091d4d8accf0c3a3aac5a1d9fbfdc42e7f55670e62"].into(),
        // 1_000_000 BTC
        assets::Asset::Bitcoin.saturating_mul(1_000_000),
      ),
      (
        assets::Asset::Ethereum.currency_id(),
        //5EeP7WRUrXPkmNobee22ZYSFcgssASW6tZpBE6jSsJoXobHg
        hex!["7227b654418bbd07626d09091d4d8accf0c3a3aac5a1d9fbfdc42e7f55670e62"].into(),
        // 1_000_000 ETH
        assets::Asset::Ethereum.saturating_mul(1_000_000),
      ),
    ]
  }

  // SECRET="key" ./scripts/prepare-dev-tidechain.sh
  #[cfg(feature = "tidechain-native")]
  pub fn get_stakeholder_tokens_tidechain() -> Vec<(CurrencyId, AccountId, Balance)> {
    vec![
      // faucet
      (
        CurrencyId::Tide,
        //5HQD3Nj89oWfh1ZgooiMDYFvS1bWHNmvtfok7krVQZyv1Hst
        hex!["ec0d12f5a230e4c757dfa1cc786ca0b86bdce58dc7c67436e67a606c3bd93735"].into(),
        // 10_000 TIDE
        assets::Asset::Tide.saturating_mul(10_000),
      ),
      // investors
      (
        CurrencyId::Tide,
        //5DJ3bpy5LB5jsCFCCig3ayHHLFmY9KByYUmFUTQiu1t4KLEV
        hex!["3668e36adabd3b469202d415106c82729d74f967cd412a29dc09dcf45b62fd02"].into(),
        // 1_000 TIDES
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5DFPWrUPp2mTWTdX6BXARVwML4BvkR33TFoXu4W8MKRJXZqv
        hex!["3462307f866e7ed4bd83a3a0d3f533bc1258fce78a50d6fdb02254d0fac86033"].into(),
        // 1_000 TIDES
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5DqsUjGivLn2jiPF4cmwnE7vuNZeW5b3w5FvqBeKYkZzccKU
        hex!["4eaeaa9358b864fa1d8e4ace39107324c3f76e74fb80ff0ce7db91bdbbc3a218"].into(),
        // 1_000 TIDES
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5Exi5fGsVGK8NvCULcS7muFiRmTQKG7wHsFPwhbXXaT3Adb7
        hex!["8021fb6321b727d4157c3ac86b4557dfefa35490bfcfe583f1a55148a65f9118"].into(),
        // 1_000 TIDES
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5EfQRF5MKSc4Ae4A8N1rVTToxPixXzcBtXiaGGbNPXUKU7z6
        hex!["72ef5a70cf4007e94e92d8ff916a611665354b9919af98ea3a938962a955613a"].into(),
        // 1_000 TIDES
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      // devs
      (
        CurrencyId::Tide,
        //5CDBQosadsb88FyTgLaK9JkrPney4KCXWKKQpXbRr6vMGfzM
        hex!["0676b7ff90098c566721f404c12f7160319b9032261660ef2fcec61710c9e177"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5DaX6GxeTKAU6uu949hvtJwpfhZkmYSiTmuwbmF7KvqLDUG4
        hex!["42f95b258e1ba467fb73babae8d23fc6c9985f207109a60a071d3db1e1e97214"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5CooWAL2vwGMbmSXJRmbaNGB7H5sHk2Vk4jfLZgomNpVjTrk
        hex!["20de8f59b70e0eb1ae4344d6f649b0e9668089a3831cf4827dd7fe21fc85067b"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5HWQJXxKpDK4m4bBSchFSRcrfsjvxzUbM5mZFUkxDcmUxJVP
        hex!["f0c674a34591219d583388dbc2d1233eb700387866fb95f49f50e901dbfea21b"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
      (
        CurrencyId::Tide,
        //5D58VZMkYurpZe2cg6k9MsMvG3Zx47HSWXprHn3SGaoZT2BZ
        hex!["2c8f2a6da01abdd82c05b2336b9df2ff3bfb4f04a519b2c830259659784b7b14"].into(),
        // 1_000 TIDE
        assets::Asset::Tide.saturating_mul(1_000),
      ),
    ]
  }
}
