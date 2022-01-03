use color_eyre::eyre;

fn main() -> eyre::Result<()> {
  color_eyre::install()?;
  tidechain_cli::run()?;
  Ok(())
}
