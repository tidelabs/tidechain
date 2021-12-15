use grandpa_primitives::AuthorityId as GrandpaId;
//use hex_literal::hex;
use codec::{Decode, Encode};
use hex_literal::hex;
use itertools::Itertools;
pub use node_tidefi_runtime::GenesisConfig;
use node_tidefi_runtime::{
  constants::currency::TIDE, wasm_binary_unwrap, AuthorityDiscoveryConfig, BabeConfig,
  BalancesConfig, CouncilConfig, FeesPalletId, IndicesConfig, SessionConfig, SessionKeys,
  StakerStatus, StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
  TreasuryPalletId, WraprAssetRegistryConfig, WraprOracleConfig, WraprQuorumConfig,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use serde_json::map::Map;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{blake2_256, crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{
  traits::{AccountIdConversion, IdentifyAccount, Verify},
  Perbill,
};

use tidefi_primitives::{assets, AssetId, Block, CurrencyId};
pub use tidefi_primitives::{AccountId, Balance, Signature};

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str =
  "ws://dedevtidesubstrate-telem.semantic-network.tech:8001/submit/";

const TESTNET_TELEMETRY_URL: &str = "wss://telemetry.tidefi.io/submit/";

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
    get_stakeholder_tokens_devnet(),
    vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
    get_account_id_from_seed::<sr25519::Public>("Alice"),
    get_account_id_from_seed::<sr25519::Public>("Alice"),
    get_all_assets(),
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
    Some(
      TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
        .expect("Staging telemetry url is valid; qed"),
    ),
    None,
    Some(properties),
    Default::default(),
  )
}

fn testnet_config_genesis() -> GenesisConfig {
  // SECRET='...' ./scripts/prepare-test-net.sh
  // The secret should be an account seed

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

  testnet_genesis(
    initial_authorities,
    get_stakeholder_tokens_testnet(),
    quorums,
    //5HKDZMoz5NnX37Np8dMKMAANbNu9N1XuQec15b3tZ8NaBTAR
    hex!["e83e965a0e2c599751184bcea1507d9fe37510d9d75eb37cba3ad8c1a5a1fe12"].into(),
    //5Hp9T9DoHRmLXsZ6j85R7xxqmUxCZ7MS4pfi4C6W6og484G6
    hex!["fe4ee0c4bae7d8a058b478c48bbaeab5e9b9d6adccacc49a45796dfb02bd9908"].into(),
    get_all_assets(),
  )
}

/// Testnet config (multisig validator)
pub fn testnet_config() -> ChainSpec {
  let mut properties = Map::new();
  properties.insert("tokenSymbol".into(), "TIDE".into());
  properties.insert("tokenDecimals".into(), 12.into());

  ChainSpec::from_genesis(
    "Testnet",
    "tidefi_testnet",
    ChainType::Live,
    testnet_config_genesis,
    vec![
      // FIXME: Add more bootnodes
      "/ip4/10.100.2.18/tcp/30333/p2p/12D3KooWEQmRfrvLbDcmxm8dGpFqkpeyUm5rLTY2SjKH9VJxE7rj"
        .parse()
        .expect("Testnet bootnode uri is valid; qed"),
    ],
    Some(
      TelemetryEndpoints::new(vec![(TESTNET_TELEMETRY_URL.to_string(), 0)])
        .expect("Testnet telemetry url is valid; qed"),
    ),
    None,
    Some(properties),
    Default::default(),
  )
}

fn adjust_treasury_balance_for_initial_validators_and_quorums(
  initial_validators: usize,
  initial_quorums: usize,
  endowment: u128,
) -> u128 {
  // Validators + quorums
  (initial_validators + initial_quorums) as u128 * endowment
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
  stakeholders: Vec<(CurrencyId, AccountId, Balance)>,
  quorums: Vec<AccountId>,
  oracle: AccountId,
  root: AccountId,
  assets: Vec<(AssetId, Vec<u8>, Vec<u8>, u8)>,
) -> GenesisConfig {
  // 1000 TIDEs / validators
  const ENDOWMENT: u128 = 1000 * TIDE;
  const TOTAL_SUPPLY: u128 = 1_000_000_000 * TIDE;
  const STASH: u128 = 2 * TIDE;

  // Stake holders
  let mut claims: Vec<(AccountId, Balance)> = stakeholders
    .clone()
    .into_iter()
    .filter(|(currency_id, _, _)| *currency_id == CurrencyId::Tide)
    .map(|(_, account_id, balance)| (account_id, balance))
    .collect();

  let mut total_claims: u128 = 0;

  for (_, balance) in &claims {
    total_claims += balance;
  }

  // get quorum
  let quorum = if quorums.len() > 1 {
    // threshold (60%)
    let threshold = (quorums.len() as f64 * 0.6).ceil() as u16;
    // create multisig from the quorum accounts provided
    let mut signatories = quorums.clone();
    signatories.sort();
    let entropy = (b"modlpy/utilisuba", &signatories, threshold).using_encoded(blake2_256);
    AccountId::decode(&mut &entropy[..]).unwrap_or_default()
  } else {
    quorums.first().unwrap().clone()
  };

  // Total funds in treasury
  let mut treasury_funds: u128 = TOTAL_SUPPLY;
  treasury_funds -=
    // remove the fund allocated to the validators and quorums
    adjust_treasury_balance_for_initial_validators_and_quorums(initial_authorities.len(), quorums.len(), ENDOWMENT)
    // all tokens claimed by the stake holders
    + total_claims
    // 1 tide endowed to the fee pallet 
    + 1_000_000_000_000;

  // Treasury Account Id
  let treasury_account: AccountId = TreasuryPalletId::get().into_account();
  let fees_account: AccountId = FeesPalletId::get().into_account();

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

  // build assets
  let final_assets = assets
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
    .collect();

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
      assets: final_assets,
      // FIXME: Is the asset_registry owner should be the same account as root?
      // this is the owner of the wrapped asset on chain and have full authority on them
      // this account can also create new wrapped asset on chain
      account: root,
    },
    wrapr_security: Default::default(),
    wrapr_fees: Default::default(),
  }
}

pub fn _get_vesting_terms() -> Vec<(AccountId, u32, u32, u32, Balance)> {
  // 43800 = minutes in a month
  // 20 blocks / minutes
  // 876_000 blocks / months
  // 2_628_000 / 3 months

  // TODO:
  // who, start, period, period_count, per_period
  // vec![ (hex!["148d5e55a937b6a6c80db86b28bc55f7336b17b13225e80468eef71d01c79341"].into(), 1, 30, 1, 2628000 * TIDE)]
  vec![]
}

pub fn get_all_assets() -> Vec<(AssetId, Vec<u8>, Vec<u8>, u8)> {
  vec![
    (assets::BTC, "Bitcoin".into(), "BTC".into(), 8),
    (assets::ETH, "Ethereum".into(), "ETH".into(), 18),
    (assets::USDC, "USD Coin".into(), "USDC".into(), 2),
    (assets::USDT, "Tether".into(), "USDT".into(), 2),
  ]
}

pub fn get_stakeholder_tokens_devnet() -> Vec<(CurrencyId, AccountId, Balance)> {
  let alice_addr = get_account_id_from_seed::<sr25519::Public>("Alice");
  let bob_addr = get_account_id_from_seed::<sr25519::Public>("Bob");
  let eve_addr = get_account_id_from_seed::<sr25519::Public>("Eve");
  let ferdie_addr = get_account_id_from_seed::<sr25519::Public>("Ferdie");

  let claims = vec![
    // alice already have quorum endowment for tides
    // 1000 TIDE
    (CurrencyId::Tide, bob_addr.clone(), 1000 * TIDE),
    (CurrencyId::Tide, eve_addr.clone(), 1000 * TIDE),
    (CurrencyId::Tide, ferdie_addr.clone(), 1000 * TIDE),
    // 100k USDT
    (
      CurrencyId::Wrapped(assets::USDT),
      alice_addr.clone(),
      10_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::USDT),
      bob_addr.clone(),
      10_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::USDT),
      eve_addr.clone(),
      10_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::USDT),
      ferdie_addr.clone(),
      10_000_000,
    ),
    // 100k USDC
    (
      CurrencyId::Wrapped(assets::USDC),
      alice_addr.clone(),
      10_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::USDC),
      bob_addr.clone(),
      10_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::USDC),
      eve_addr.clone(),
      10_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::USDC),
      ferdie_addr.clone(),
      10_000_000,
    ),
    // 10k BTC
    (
      CurrencyId::Wrapped(assets::BTC),
      alice_addr.clone(),
      1_000_000_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::BTC),
      bob_addr.clone(),
      1_000_000_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::BTC),
      eve_addr.clone(),
      1_000_000_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::BTC),
      ferdie_addr.clone(),
      1_000_000_000_000,
    ),
    // 10k ETH
    (
      CurrencyId::Wrapped(assets::ETH),
      alice_addr,
      10_000_000_000_000_000_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::ETH),
      bob_addr,
      10_000_000_000_000_000_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::ETH),
      eve_addr,
      10_000_000_000_000_000_000_000,
    ),
    (
      CurrencyId::Wrapped(assets::ETH),
      ferdie_addr,
      10_000_000_000_000_000_000_000,
    ),
  ];
  claims
}

// SECRET="key" ./scripts/prepare-test-net.sh
pub fn get_stakeholder_tokens_testnet() -> Vec<(CurrencyId, AccountId, Balance)> {
  vec![
    (
      CurrencyId::Tide,
      //5DUTRtdo3T6CtLx5rxJQxAVhT9RmZUWGw4FJWZSPWbLFhNf2
      hex!["3e598e8ee9577c609c70823e394ab1a2e0301f73f074a773a3a1b20bfba9050e"].into(),
      // 1000 TIDES
      1000 * TIDE,
    ),
    (
      CurrencyId::Tide,
      //5GHab6U9Ke5XjbHHEB5WSUreyp293BryKjJrGWgQ1nCvEDzM
      hex!["bac2a7f4be9d7e0f8eee75e0af5e33240698e8ac0b02904627bd9c4d37b3dd5e"].into(),
      // 1000 TIDES
      1000 * TIDE,
    ),
    (
      CurrencyId::Tide,
      //5CLmiDfMLGbuuvuuc87ZF1fr9itkyVzTE5hjWb725JemcGka
      hex!["0c40e6b8b6686685828658080a17af04562fa69818c848146795c8c586691a68"].into(),
      // 1000 TIDES
      1000 * TIDE,
    ),
    (
      CurrencyId::Tide,
      //5CJMQZA3LgdZ7EXN1eTXjxqQvmxgZEuXy9iWA1Yvd67zK9Da
      hex!["0a689812fb1b2763c3ff90ad8f12c652848904d7f4cb3ea5d5328a30c4d3c978"].into(),
      // 1000 TIDES
      1000 * TIDE,
    ),
    (
      CurrencyId::Tide,
      //5DWorbmbirDwHNNrLFu15aRjD63fiEAbi5K9Eo96mxwirVdM
      hex!["4024cecb82ca165b7960b22a19ac3fafa5240582691eaf22ffee7a6f06cb1526"].into(),
      // 1000 TIDES
      1000 * TIDE,
    ),
  ]
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use sp_runtime::BuildStorage;

  fn local_testnet_genesis_instant_single() -> GenesisConfig {
    testnet_genesis(
      vec![authority_keys_from_seed("Alice")],
      vec![],
      vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
      get_account_id_from_seed::<sr25519::Public>("Alice"),
      get_account_id_from_seed::<sr25519::Public>("Alice"),
      vec![],
    )
  }

  fn local_testnet_genesis_instant_multiple() -> GenesisConfig {
    testnet_genesis(
      vec![
        authority_keys_from_seed("Alice"),
        authority_keys_from_seed("Bob"),
      ],
      vec![],
      vec![get_account_id_from_seed::<sr25519::Public>("Alice")],
      get_account_id_from_seed::<sr25519::Public>("Alice"),
      get_account_id_from_seed::<sr25519::Public>("Alice"),
      vec![],
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
      local_testnet_genesis_instant_multiple,
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
  fn test_create_testnet_chain_spec() {
    assert!(!testnet_config().build_storage().is_err());
  }
}
