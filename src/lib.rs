use std::path::PathBuf;

use anyhow::anyhow;
use log::debug;

pub mod download;

pub struct WabbaRust {
	api_key: String,
	install_dir: PathBuf,
	download_dir: PathBuf,
}

impl WabbaRust {
	pub fn new(api_key: String, install_dir: PathBuf, download_dir: Option<PathBuf>) -> WabbaRust {
		let download_dir: PathBuf = download_dir.unwrap_or([install_dir.clone(), PathBuf::from("downloads")].iter().collect());

		return WabbaRust {
			api_key,
			install_dir,
			download_dir,
		};
	}

	pub fn init(&self) -> anyhow::Result<()> {
		self.init_logger()?;
		self.init_paths()?;

		return Ok(());
	}

	pub fn get_api_key(&self) -> String {
		return self.api_key.clone();
	}

	pub fn get_install_dir(&self) -> PathBuf {
		return self.install_dir.clone();
	}

	pub fn get_download_dir(&self) -> PathBuf {
		return self.download_dir.clone();
	}

	fn init_logger(&self) -> anyhow::Result<()> {
		std::env::set_var("RUST_LOG", "trace");
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
}
