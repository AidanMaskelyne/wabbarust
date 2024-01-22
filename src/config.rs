use std::{
	path::PathBuf,
	fs::OpenOptions,
	io::Write
};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct WabbaRustConfig {
	#[serde(skip)]
	config_dir: PathBuf,

	#[serde(skip)]
	manifest_dir: PathBuf,

	#[serde(skip)]
	config_file: PathBuf,

	pub config: Config,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
	pub api_key: Option<String>,
}

impl WabbaRustConfig {
	pub fn load() -> anyhow::Result<WabbaRustConfig> {
		let config_dir: PathBuf = [dirs::config_dir().unwrap(), PathBuf::from("wabbarust")].iter().collect();
		let manifest_dir: PathBuf = [config_dir.clone(), PathBuf::from("wabbarust")].iter().collect();
		let config_file: PathBuf = [config_dir.clone(), PathBuf::from("wabbarust.toml")].iter().collect();

		if !config_dir.exists() {
			std::fs::create_dir_all(config_dir.clone())?;
		}

		if !config_file.exists() {
			std::fs::File::create_new(config_file.clone())?;
		}

		let config = std::fs::read_to_string(config_file.clone())?;
		let config: WabbaRustConfig = toml::from_str(&config)?;

		return Ok(WabbaRustConfig {
			config_dir,
			manifest_dir,
			config_file,
			..config
		});
	}

	pub fn get_config_dir(&self) -> PathBuf {
		return self.config_dir.clone();
	}

	pub fn get_manifest_dir(&self) -> PathBuf {
		return self.manifest_dir.clone();
	}

	pub fn get_config_file(&self) -> PathBuf {
		return self.config_file.clone();
	}

	pub fn set_option(&mut self, option: String, value: String) {
		match option.as_str() {
			"api_key" => {
				self.config.api_key = Some(value);
			}
			_ => {}
		};

		let toml = toml::to_string(&self).unwrap();

		let mut file = OpenOptions::new()
			.write(true)
			.create(false)
			.open(self.get_config_file()).unwrap();

		file.write_all(toml.as_bytes()).unwrap();
	}
}
