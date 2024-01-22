use std::path::PathBuf;

use clap::Parser;
use wabbarust::WabbaRust;
use wabbarust::args::Args;
use wabbarust::config::WabbaRustConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	let wr_config = WabbaRustConfig::load()?;

	let install_folder: PathBuf = [std::env::current_dir()?, PathBuf::from("install")].iter().collect();
	let wr = WabbaRust::new(
		wr_config.config.api_key.clone().unwrap(),
		install_folder,
		None,
		args.debug
	);
	wr.init()?;

	wr.start_downloading().await?;

	return Ok(());
}
