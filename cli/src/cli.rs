use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
  #[clap(subcommand)]
  pub subcommand: Option<Subcommand>,

  #[clap(flatten)]
  pub run: RunCmd,
}

#[derive(Debug, Parser)]
pub enum Subcommand {
  /// Build a chain specification.
  BuildSpec(sc_cli::BuildSpecCmd),

  /// Validate blocks.
  CheckBlock(sc_cli::CheckBlockCmd),

  /// Export blocks.
  ExportBlocks(sc_cli::ExportBlocksCmd),

  /// Export the state of a given block into a chain spec.
  ExportState(sc_cli::ExportStateCmd),

  /// Import blocks.
  ImportBlocks(sc_cli::ImportBlocksCmd),

  /// Remove the whole chain.
  PurgeChain(sc_cli::PurgeChainCmd),

  /// Revert the chain to a previous state.
  Revert(sc_cli::RevertCmd),

  ExportBuiltinWasm(ExportBuiltinWasmCommand),

  /// The custom benchmark subcommmand benchmarking runtime pallets.
  Benchmark(frame_benchmarking_cli::BenchmarkCmd),

  /// Key management CLI utilities
  #[clap(subcommand)]
  Key(sc_cli::KeySubcommand),
}

#[derive(Debug, Parser)]
pub struct RunCmd {
  #[clap(flatten)]
  pub base: sc_cli::RunCmd,

  /// Force using Lagoon native runtime.
  #[clap(long = "force-lagoon")]
  pub force_lagoon: bool,

  /// Setup a GRANDPA scheduled voting pause.
  ///
  /// This parameter takes two values, namely a block number and a delay (in
  /// blocks). After the given block number is finalized the GRANDPA voter
  /// will temporarily stop voting for new blocks until the given delay has
  /// elapsed (i.e. until a block at height `pause_block + delay` is imported).
  #[clap(long = "grandpa-pause", number_of_values(2))]
  pub grandpa_pause: Vec<u32>,
}

#[derive(Debug, Parser)]
pub struct ExportBuiltinWasmCommand {
  pub folder: String,
}
