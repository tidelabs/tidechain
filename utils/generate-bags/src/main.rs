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
