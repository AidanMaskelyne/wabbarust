use std::io::{stdout, Write, stdin};
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use wabbarust::WabbaRust;
use wabbarust::args::Args;
use wabbarust::config::WabbaRustConfig;
use wabbarust::download::{
	direct::DirectDownload,
	nexus::NexusDownload,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	let mut wr_config = WabbaRustConfig::load()?;

	if wr_config.config.api_key == None {
		let mut input = String::new();
		print!("Please enter an API key: ");
		stdout().flush()?;
		stdin().read_line(&mut input).context("Failed to read stdin")?;
		let input = input.trim().to_string();
		wr_config.set_option(String::from("api_key"), input);
	}


	let install_folder: PathBuf = [std::env::current_dir()?, PathBuf::from("install")].iter().collect();
	let wr = WabbaRust::new(
		wr_config.config.api_key.clone().unwrap(),
		install_folder,
		None,
		args.debug
	);
	wr.init()?;

	let _ussep = NexusDownload::new(
		String::from("Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z"),
		String::from("4.3.0a"),
		String::from("266"),
		String::from("skyrimspecialedition"),
		String::from("c75d8dd9478fb7fc685507c01313f36db909965c409082f23a14c2031d36ba6a")
	);

	let mo2 = DirectDownload::new(
		String::from("Mod.Organizer-2.5.0.7z"),
		String::from("https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z"),
		String::from("9f20a7f2807f5b5a0f801e749d1f4f9160d32b684fe4c27a2d70b0f29fa0fc94"),
	);

	mo2.download(wr.get_download_dir()).await?;

	// ussep.download(wr.get_download_dir(), wr.get_api_key()).await?;

	return Ok(());
}
