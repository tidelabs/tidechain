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

use generate_bags::generate_thresholds;
use lagoon_runtime::Runtime as LagoonRuntime;
use std::path::{Path, PathBuf};
use structopt::{clap::arg_enum, StructOpt};
use tidechain_runtime::Runtime as TidechainRuntime;

arg_enum! {
  #[derive(Debug)]
  enum Runtime {
    Tidechain,
    Lagoon,
  }
}

impl Runtime {
  fn generate_thresholds_fn(
    &self,
  ) -> Box<dyn FnOnce(usize, &Path, u128, u128) -> Result<(), std::io::Error>> {
    match self {
      Runtime::Tidechain => Box::new(generate_thresholds::<TidechainRuntime>),
      Runtime::Lagoon => Box::new(generate_thresholds::<LagoonRuntime>),
    }
  }
}

#[derive(Debug, StructOpt)]
struct Opt {
  /// How many bags to generate.
  #[structopt(long, default_value = "200")]
  n_bags: usize,

  /// Which runtime to generate.
  #[structopt(
		long,
		case_insensitive = true,
		default_value = "Tidechain",
		possible_values = &Runtime::variants(),
	)]
  runtime: Runtime,

  /// Where to write the output.
  output: PathBuf,

  /// The total issuance of the native currency.
  #[structopt(short, long)]
  total_issuance: u128,

  /// The minimum account balance (i.e. existential deposit) for the native currency.
  #[structopt(short, long)]
  minimum_balance: u128,
}

fn main() -> Result<(), std::io::Error> {
  let Opt {
    n_bags,
    output,
    runtime,
    total_issuance,
    minimum_balance,
  } = Opt::from_args();

  runtime.generate_thresholds_fn()(n_bags, &output, total_issuance, minimum_balance)
}
