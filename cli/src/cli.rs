// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of Tidechain.

// Tidechain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tidechain.  If not, see <http://www.gnu.org/licenses/>.
#![allow(clippy::all)]

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

  /// Try some command against runtime state.
  #[cfg(feature = "try-runtime")]
  TryRuntime(try_runtime_cli::TryRuntimeCmd),

  /// The custom benchmark subcommmand benchmarking runtime pallets.
  #[cfg(feature = "runtime-benchmarks")]
  #[clap(subcommand)]
  Benchmark(frame_benchmarking_cli::BenchmarkCmd),

  /// Key management CLI utilities
  #[clap(subcommand)]
  Key(sc_cli::KeySubcommand),
}

#[derive(Debug, Parser)]
#[group(skip)]
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

  /// Disable automatic hardware benchmarks.
  ///
  /// By default these benchmarks are automatically ran at startup and measure
  /// the CPU speed, the memory bandwidth and the disk speed.
  ///
  /// The results are then printed out in the logs, and also sent as part of
  /// telemetry, if telemetry is enabled.
  #[arg(long)]
  pub no_hardware_benchmarks: bool,
}

#[derive(Debug, Parser)]
pub struct ExportBuiltinWasmCommand {
  pub folder: String,
}
