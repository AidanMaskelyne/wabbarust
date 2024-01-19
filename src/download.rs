use std::cmp::min;
use std::fs::File;
use std::io::Write;

use anyhow::anyhow;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use futures_util::StreamExt;
use terminal_size::{Width, Height, terminal_size};

pub async fn download_file(client: &Client, path: &str) -> anyhow::Result<()> {
	let download_url = get_download_url().await?;

	let res = client
		.get(&download_url)
		.header("apikey", "Aeby8A0QzwcCnVrkMXjqf3Id38TRmVrnB9YDI9o9kAx4--Izy5qNKLuv2xBfl3--ZbD0azECEsornaIWSYxZgA==")
		.send()
		.await
		.or(Err(anyhow!("Failed to GET from `{}`", &download_url)))?;

	let term_width = terminal_size().unwrap().0.0;
	let max_text_len = term_width - (3 + 2);

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
	let _res = reqwest::Client::new()
		.get("https://api.nexusmods.com/api/v1/games/skyrimspecialedition/mods/266/files/449719/download_link.json")
		.header("apikey", "Aeby8A0QzwcCnVrkMXjqf3Id38TRmVrnB9YDI9o9kAx4--Izy5qNKLuv2xBfl3--ZbD0azECEsornaIWSYxZgA==")
		.send()
		.await?;
		
		return Ok(format!("https://cf-files.nexusmods.com/cdn/1704/266/Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z?md5=p9GMH2_Zu8cmds9k7FkEhg&expires=1705671703&user_id=152419148"));
}
