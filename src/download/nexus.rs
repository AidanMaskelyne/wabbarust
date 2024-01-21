use std::{cmp::min, path::PathBuf};
use std::fs::File;
use std::io::Write;

use anyhow::anyhow;
use log::debug;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use futures_util::StreamExt;
use serde_json::Value;

pub struct NexusDownload {
	file_name: String,
	version: String,
	mod_id: String,
	hash: String,
}

impl NexusDownload {
	pub fn new(file_name: String, version: String, mod_id: String, hash: String) -> NexusDownload {
		return NexusDownload {
			file_name,
			version,
			mod_id,
			hash,
		};
	}

	pub fn get_file_name(&self) -> String {
		return self.file_name.clone();
	}

	pub fn get_mod_id(&self) -> String {
		return self.mod_id.clone();
	}

	pub fn get_version(&self) -> String {
		return self.version.clone();
	}

	pub fn get_hash(&self) -> String {
		return self.hash.clone();
	}

	pub async fn download(&self, download_dir: PathBuf, api_key: String) -> anyhow::Result<()> {
		let client = Client::new();
		let file_name = self.get_file_name();
		let (download_url, file) = self.get_download_info(download_dir.clone(), api_key.clone()).await?;

		debug!("Starting download of {file_name} from {}", download_url);
		let res = client
			.get(&download_url)
			.header("apikey", api_key)
			.send()
			.await
			.or(Err(anyhow!("Failed to GET from `{}`", &download_url)))?;

		let total_size = res.content_length().ok_or(anyhow!("Failed to get content length from `{}`.", download_url))?;
		let progress_bar_message = String::from("Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z");

		let progress_bar = ProgressBar::new(total_size);
		progress_bar.set_style(ProgressStyle::default_bar()
			.template("{msg} [{bar:50}] {percent_custom}")?
			.with_key("percent_custom", |state: &ProgressState, w: &mut dyn std::fmt::Write| { // Make the percentage always take up the same width
				let percent = (state.fraction() * 100.0).floor() as u32;
				if percent < 10 {
					write!(w, "  {}%", percent).unwrap();
				} else if percent >= 10 && percent < 100 {
					write!(w, " {}%", percent).unwrap();
				} else {
					write!(w, "{}%", percent).unwrap();
				}
			})
			.progress_chars("#-"));
		progress_bar.set_message(progress_bar_message);

		let mut f = File::create(file.clone()).or(Err(anyhow!("Failed to create file `{}`", file.display())))?;
		let mut downloaded: u64 = 0;
		let mut stream = res.bytes_stream();

		while let Some(item) = stream.next().await {
			let chunk = item.or(Err(anyhow!("Error while downloading file")))?;
			f.write_all(&chunk).or(Err(anyhow!("Failed to write chunk to file")))?;
			let new = min(downloaded+(chunk.len() as u64), total_size);
			downloaded = new;
			progress_bar.set_position(new);
		}

		progress_bar.finish_with_message(format!("{}", file_name));

		debug!("Completed download of `{}`", file_name);
		debug!("Checking hash of `{}`", file_name);
		println!("Hashing `{file_name}`...");
		self.check_hash(&file)?;
		debug!("File `{}` hashed successfully", file_name);

		return Ok(());
	}

	async fn get_download_info(&self, download_dir: PathBuf, api_key: String) -> anyhow::Result<(String, PathBuf)> {
		let client = reqwest::Client::new();
		let mod_files = client
			.get(format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files.json", "skyrimspecialedition", self.get_mod_id()))
			.header("apikey", api_key.clone())
			.send().await?
			.text().await?;
	
		let mod_files: Value = serde_json::from_str(&mod_files)?;
		let mut file_id = String::new();
		let mut file_name = String::new();
	
		if let Some(files) = mod_files["files"].as_array() {
			for file in files {
				if file["version"] == self.get_version() {
					file_id = file["file_id"].to_string();
					file_name = file["file_name"].as_str().unwrap().to_string(); 	// Serde string `Value`s contain quotes by default; converting to str removes them.
				}
			}
		};
		
		let download_url = client
			.get(format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json", "skyrimspecialedition", self.get_mod_id(), file_id))
			.header("apikey", api_key)
			.send().await?
			.text().await?;
	
		// TODO: Check other error codes returned in JSON response
	
		let download_url: Value = serde_json::from_str(&download_url)?;
		let download_url = &download_url[0]["URI"];
	
		return Ok((
			String::from(download_url.as_str().unwrap()),
			PathBuf::from([download_dir, PathBuf::from(file_name)].iter().collect::<PathBuf>())
		));
	}

	fn check_hash(&self, file: &PathBuf) -> anyhow::Result<()> {
		let hash = sha256::try_digest(file)?;

		if hash != self.get_hash() {
			return Err(anyhow!("downloaded file's hash doesn't match one on record"));
		}

		return Ok(());
	}
}