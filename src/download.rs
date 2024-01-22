use std::cmp::min;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::PathBuf;

use anyhow::anyhow;
use colored::Colorize;
use log::debug;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressState, ProgressStyle, HumanBytes};
use futures_util::StreamExt;
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct NexusModsFiles {
	files: Vec<NexusModsFile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NexusModsFile {
	id: Vec<i32>,
	file_id: i32,
	name: String,
	version: String,
	file_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[allow(non_snake_case)] // The field in the JSON response is "URI" and serde is case-sensitive
pub struct NexusModsDownloadResponse {
	name: String,
	short_name: String,
	URI: String,
}

pub enum DownloadType {
	Direct,
	Nexus
}

#[derive(Deserialize, Clone)]
pub struct Download {
	dl_type: String,
	file_name: String,
	version: Option<String>,
	mod_id: Option<String>,
	game: Option<String>,
	url: Option<String>,
	hash: String,
}

impl Download {
	pub fn new(
		dl_type: String,
		file_name: String,
		version: Option<String>,
		mod_id: Option<String>,
		game: Option<String>,
		url: Option<String>,
		hash: String,
	) -> Download {
		return Download {
			dl_type,
			file_name,
			version,
			mod_id,
			game,
			url,
			hash,
		};
	}

	pub async fn download(&self, client: &Client, download_dir: PathBuf, api_key: String) -> anyhow::Result<()> {
		let file_name = self.get_file_name();
		let output_file: PathBuf = [download_dir.clone(), PathBuf::from(self.get_file_name())].iter().collect();
		let download_url = self.get_download_url(&client, &api_key).await?;

		let res = match self.get_dl_type().unwrap() {
			DownloadType::Nexus => {
				client
					.get(&download_url)
					.header("apikey", &api_key)
					.send().await
					.or(Err(anyhow!("Failed to GET from `{}`", &download_url)))?
			},
			DownloadType::Direct => {
				client
					.get(&download_url)
					.send().await
					.or(Err(anyhow!("Failed to GET from `{}`", &download_url)))?
			}
		};

		let total_size = res.content_length().ok_or(anyhow!("Failed to get content length from `{}`", &download_url))?;

		let progress_bar = ProgressBar::new(total_size);
		progress_bar.set_style(ProgressStyle::default_bar()
			.template("{msg_custom} {bytes_per_sec} [{bar:50}] {percent_custom}")?
			.with_key("msg_custom", move |state: &ProgressState, w: &mut dyn std::fmt::Write| { // Truncate the file name so the entire bar fits on 1 line of the terminal
				let term_width = terminal_size::terminal_size().unwrap().0.0;
				let per_sec_length = String::from(format!("{}", HumanBytes(state.per_sec().floor() as u64))).len() + 3;

				// per sec + space + [ + bar + ] + space + percent
				let available_width = match term_width.checked_sub(per_sec_length as u16 + 1 + 52 + 1 + 4) {
					Some(x) => x,
					None => 0,
				};

				if file_name.len()  > available_width.into() {
					let mut msg = file_name.clone();
					if available_width < 2 {
						msg.truncate(available_width.into());
					} else {
						msg.truncate((available_width-2).into());
						msg.push('~');
						msg.push('~');
					}
					write!(w, "{}", msg).unwrap();
				} else {
					let mut msg = file_name.clone();
					if msg.len() < available_width.into() {
						for _ in 0..(available_width - msg.len() as u16) {
							msg.push(' ');
						}
					}
					write!(w, "{}", msg).unwrap();
				}
			})
			.with_key("percent_custom", |state: &ProgressState, w: &mut dyn std::fmt::Write| { // Make the percentage always take up the same width
				let percent = (state.fraction() * 100.0).floor() as u32;
				if percent < 10 {
					write!(w, "  {}%", percent).unwrap();
				} else if percent >= 10 && percent < 100 {
					write!(w, " {}%", percent).unwrap();
				} else {
					write!(w, "{}", format!("{}%", percent).green()).unwrap();
				}
			})
			.progress_chars("#-"));
		progress_bar.set_message(self.get_file_name());

		let mut file = File::create(&output_file).or(Err(anyhow!("Failed to create file `{}`", &output_file.display())))?;
		let mut downloaded: u64 = 0;
		let mut stream = res.bytes_stream();

		while let Some(item) = stream.next().await {
			let chunk = item.or(Err(anyhow!("Error while downloading file `{}`", self.get_file_name())))?;
			file.write_all(&chunk).or(Err(anyhow!("Failed to write chunk to file `{}`", &output_file.display())))?;
			let new = min(downloaded+(chunk.len() as u64), total_size);
			downloaded = new;
			progress_bar.set_position(new);
		}
		progress_bar.finish_with_message(self.get_file_name());

		self.hash_download(&output_file)?;

		return Ok(());
	}

	fn hash_download(&self, file: &PathBuf) -> anyhow::Result<()> {
		debug!("Checking hash of `{}`", self.get_file_name());

		print!("    {} Hashing `{}`...", "↳".blue(), self.get_file_name());
		stdout().flush()?;

		let hash = match sha256::try_digest(file) {
			Ok(hash) => hash,
			Err(e) => {
				println!(" {}", "FAILED".red().bold());
				return Err(anyhow!(e));
			}
		};
	
		if hash != self.get_hash() {
			println!(" {}", "FAILED".red());
			return Err(anyhow!("Downloaded file's hash didn't match one on record"));
		}
		
		println!(" {}", "OK".green().bold());
		debug!("File `{}` hashed successfully", self.get_file_name());

		return Ok(());
	}

	pub fn get_dl_type(&self) ->  anyhow::Result<DownloadType> {
		return match self.dl_type.as_str() {
			"direct" => Ok(DownloadType::Direct),
			"nexus" => Ok(DownloadType::Nexus),
			_ => Err(anyhow!("Unknown download type in manifest")),
		};
	}

	pub fn get_file_name(&self) -> String {
		return self.file_name.clone();
	}

	pub fn get_version(&self) -> Option<String> {
		return self.version.clone();
	}

	pub fn get_mod_id(&self) -> Option<String> {
		return self.mod_id.clone();
	}

	pub fn get_game(&self) -> Option<String> {
		return self.game.clone();
	}

	pub async fn get_download_url(&self, client: &Client, api_key: &String) -> anyhow::Result<String> {
		return match self.dl_type.as_str() {
			"direct" => Ok(self.url.clone().unwrap()),
			"nexus" => Ok(self.get_nexus_download_url(client, api_key).await?),
			_ => Err(anyhow!("Can't get download URL of invalid download type"))
		};
	}

	pub async fn get_nexus_download_url(&self, client: &Client, api_key: &String) -> anyhow::Result<String> {
		let mod_files = client
			.get(format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files.json", self.get_game().unwrap(), self.get_mod_id().unwrap()))
			.header("apikey", api_key.clone())
			.send().await?
			.text().await?;

		// Check for any error codes in the response
		{
			let mod_files: Value = serde_json::from_str(&mod_files)?;

			match &mod_files["code"] {
				serde_json::Value::Number(n) => {
					return Err(anyhow!("Mod files reqeust returned {n}: {}", mod_files["message"]));
				}
				_ => {}
			}
		}

		let mod_files: NexusModsFiles = serde_json::from_str(&mod_files)?;
		let mut file_id = String::new();

		for file in &mod_files.files {
			if file.version == self.get_version().unwrap() && file.file_name == self.get_file_name() {
				file_id = file.file_id.clone().to_string();
			}
		}

		let download_urls = client
			.get(format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json", self.get_game().unwrap(), self.get_mod_id().unwrap(), file_id))
			.header("apikey", api_key)
			.send().await?
			.text().await?;

		// Check for any error codes in the response
		{
			let download_urls: Value = serde_json::from_str(&download_urls)?;

			match &download_urls["code"] {
				serde_json::Value::Number(n) => {
					return Err(anyhow!("Mod files reqeust returned {n}: {}", download_urls["message"]));
				}
				_ => {}
			}
		}

		let download_urls: Vec<NexusModsDownloadResponse> = serde_json::from_str(&download_urls)?;
		let download_url = &download_urls[0].URI; // The download location at index 0 is the user's preference

		return Ok(String::from(download_url));
	}

	pub fn get_hash(&self) -> String {
		return self.hash.clone();
	}
}
