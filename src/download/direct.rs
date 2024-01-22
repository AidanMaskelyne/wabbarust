use std::{cmp::min, path::PathBuf};
use std::fs::File;
use std::io::Write;

use anyhow::anyhow;
use colored::Colorize;
use log::debug;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use futures_util::StreamExt;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DirectDownload {
	file_name: String,
	url: String,
	hash: String,
}

impl DirectDownload {
	pub fn new(file_name: String, url: String, hash: String) -> DirectDownload {
		return DirectDownload {
			file_name,
			url,
			hash,
		};
	}

	pub fn get_file_name(&self) -> String {
		return self.file_name.clone();
	}

	pub fn get_url(&self) -> String {
		return self.url.clone();
	}

	pub fn get_hash(&self) -> String {
		return self.hash.clone();
	}

	pub async fn download(&self, download_dir: PathBuf) -> anyhow::Result<()> {
		let client = Client::new();
		let file_name = self.get_file_name();
		let file = PathBuf::from([download_dir, PathBuf::from(self.get_file_name())].iter().collect::<PathBuf>());
		let download_url = self.get_url();

		debug!("Starting download of {} from {}", self.get_file_name(), download_url);
		let res = client
			.get(&download_url)
			.send()
			.await
			.or(Err(anyhow!("Failed to GET from `{}`", &download_url)))?;

		let total_size = res.content_length().ok_or(anyhow!("Failed to get content length from `{}`.", download_url))?;
		let progress_bar_message = self.get_file_name();

		let progress_bar = ProgressBar::new(total_size);
		progress_bar.set_style(ProgressStyle::default_bar()
			.template("{msg_custom} [{bar:50}] {percent_custom}")?
			.with_key("msg_custom", move |_state: &ProgressState, w: &mut dyn std::fmt::Write| { // Truncate the file name so the entire bar fits on 1 line of the terminal
				let term_width = terminal_size::terminal_size().unwrap().0.0;
				let available_width = match term_width.checked_sub(1 + 1 + 50 + 1 + 1 + 4) {
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

		progress_bar.finish_with_message(format!("{}", self.get_file_name()));

		debug!("Completed download of `{}`", self.get_file_name());
		self.check_hash(&file)?;

		return Ok(());
	}

	fn check_hash(&self, file: &PathBuf) -> anyhow::Result<()> {
		debug!("Checking hash of `{}`", self.get_file_name());
		print!("  {} Hashing `{}`...", "↳".blue(), self.get_file_name());
		std::io::stdout().flush()?;
		let hash = sha256::try_digest(file)?;

		if hash != self.get_hash() {
			println!(" {}", "FAILED".red());
			return Err(anyhow!("downloaded file's hash doesn't match one on record"));
		}

		println!(" {}", "Done".green());
		debug!("File `{}` hashed successfully", self.get_file_name());

		return Ok(());
	}
}
