use std::cmp::min;
use std::fs::File;
use std::io::Write;

use anyhow::anyhow;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use futures_util::StreamExt;
use serde_json::Value;
use terminal_size::{Width, Height, terminal_size};

pub async fn download_file(client: &Client, path: &str) -> anyhow::Result<()> {
	let download_url = get_download_url().await?;

	println!("Download URL: {}", download_url);
	let res = client
		.get(&download_url)
		.header("apikey", "Aeby8A0QzwcCnVrkMXjqf3Id38TRmVrnB9YDI9o9kAx4--Izy5qNKLuv2xBfl3--ZbD0azECEsornaIWSYxZgA==")
		.send()
		.await
		.or(Err(anyhow!("Failed to GET from `{}`", &download_url)))?;

	let total_size = res.content_length().ok_or(anyhow!("Failed to get content length from `{}`.", download_url))?;
	let progress_bar_message = String::from("Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z");

	let progress_bar = ProgressBar::new(total_size);
	progress_bar.set_style(ProgressStyle::default_bar()
		//.template("{msg}\n[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
		.template("{msg:>5} [{bar:50}] {percent_custom}")?
		.with_key("percent_custom", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "hello").unwrap())
		.progress_chars("#-"));
	progress_bar.set_message(progress_bar_message);

	let mut file = File::create(path).or(Err(anyhow!("Failed to create file `{}`", &path)))?;
	let mut downloaded: u64 = 0;
	let mut stream = res.bytes_stream();

	while let Some(item) = stream.next().await {
		let chunk = item.or(Err(anyhow!("Error while downloading file")))?;
		file.write_all(&chunk).or(Err(anyhow!("Failed to write chunk to file")))?;
		let new = min(downloaded+(chunk.len() as u64), total_size);
		downloaded = new;
		progress_bar.set_position(new);
	}

	progress_bar.finish_with_message(format!("Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z"));

	return Ok(());
}

async fn get_download_url() -> anyhow::Result<String> {
	let client = reqwest::Client::new();
	let mod_files = client
		.get(format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files.json", "skyrimspecialedition", 266))
		.header("apikey", "Aeby8A0QzwcCnVrkMXjqf3Id38TRmVrnB9YDI9o9kAx4--Izy5qNKLuv2xBfl3--ZbD0azECEsornaIWSYxZgA==")
		.send().await?
		.text().await?;

	let mod_files: Value = serde_json::from_str(&mod_files)?;
	let mut file_id = String::new();

	if let Some(files) = mod_files["files"].as_array() {
		for file in files {
			if file["version"] == "4.3.0a" {
				file_id = file["file_id"].to_string();
			}
		}
	};
	
	let download_url = client
		.get(format!("https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json", "skyrimspecialedition", 266, file_id))
		.header("apikey", "Aeby8A0QzwcCnVrkMXjqf3Id38TRmVrnB9YDI9o9kAx4--Izy5qNKLuv2xBfl3--ZbD0azECEsornaIWSYxZgA==")
		.send().await?
		.text().await?;

	// TODO: Check other error codes return in JSON response

	let download_url: Value = serde_json::from_str(&download_url)?;
	let download_url = &download_url[0]["URI"];

	return Ok(download_url[0].to_string());
}
