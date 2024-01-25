use std::path::PathBuf;

use reqwest::Client;
use anyhow::anyhow;
use log::debug;

use args::Commands;
use config::WabbaRustConfig;
use download::Download;

pub mod args;
pub mod config;
pub mod download;

#[derive(Clone)]
pub struct WabbaRust {
	config: WabbaRustConfig,
	install_dir: PathBuf,
	download_dir: PathBuf,
	debug: bool,

	downloads: Vec<Download>
}

impl WabbaRust {
	pub fn new(config: WabbaRustConfig, install_dir: PathBuf, download_dir: Option<PathBuf>, debug: bool) -> WabbaRust {
		let download_dir: PathBuf = download_dir.unwrap_or([install_dir.clone(), PathBuf::from("downloads")].iter().collect());

		return WabbaRust {
			config,
			install_dir,
			download_dir,
			debug,

			downloads: vec![
				Download::new(
					String::from("direct"),
					String::from("Mod.Organizer-2.5.0.7z"),
					None,
					None,
					None,
					Some(String::from("https://github.com/ModOrganizer2/modorganizer/releases/download/v2.5.0/Mod.Organizer-2.5.0.7z")),
					String::from("9f20a7f2807f5b5a0f801e749d1f4f9160d32b684fe4c27a2d70b0f29fa0fc94"),
				),
				Download::new(
					String::from("nexus"),
					String::from("Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z"),
					Some(String::from("4.3.0a")),
					Some(String::from("266")),
					Some(String::from("skyrimspecialedition")),
					None,
					String::from("c75d8dd9478fb7fc685507c01313f36db909965c409082f23a14c2031d36ba6a"),
				)
			]
		};
	}

	pub fn init(&self) -> anyhow::Result<()> {
		self.init_logger()?;
		self.init_paths()?;

		return Ok(());
	}

	pub fn get_api_key(&self) -> String {
		return self.config.get_api_key();
	}

	pub fn get_install_dir(&self) -> PathBuf {
		return self.install_dir.clone();
	}

	pub fn get_download_dir(&self) -> PathBuf {
		return self.download_dir.clone();
	}

	fn init_logger(&self) -> anyhow::Result<()> {
		if self.debug {
			std::env::set_var("RUST_LOG", "debug");
		}

		env_logger::try_init()?;

		return Ok(());
	}

	fn init_paths(&self) -> anyhow::Result<()> {
		if !self.install_dir.exists() {
			debug!("Creating directory {}", self.install_dir.display());
			std::fs::create_dir(self.install_dir.clone())?;

			debug!("Creating directory {}", self.download_dir.display());
			std::fs::create_dir(self.download_dir.clone())?;

			return Ok(());
		}

		return Err(anyhow!("required directories already exist"));
	}

	pub async fn start_downloading(&self) -> anyhow::Result<()> {
		let client = Client::new();

		for download in &self.downloads {
			download.download(&client, self.get_download_dir(), self.get_api_key()).await?;
		}

		return Ok(());
	}

	pub async fn exec_command(&mut self, command: crate::args::Commands) -> anyhow::Result<()> {
		match command {
			Commands::Install { modlist } => self.install_modlist(modlist).await?,
			Commands::Config { option, value } => self.config.set_option(&option, &value),
			Commands::Repair { modlist } => {},
		}

		return Ok(());
	}

	async fn install_modlist(&self, modlist: String) -> anyhow::Result<()> {
		// TODO: Search ~/.config/wabbarust/manifests/ for `modlist`, or assume it's a path
		// TODO: and then load the manifest file
		self.start_downloading().await?;

		return Ok(());
	}
}
