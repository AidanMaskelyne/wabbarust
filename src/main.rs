#![allow(non_snake_case)]
use wabbarust::WabbaRust;
use wabbarust::download::download_file;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	WabbaRust::init()?;

	let client = reqwest::Client::new();

	download_file(&client, "/home/aidan/dev/wabbarust/downloads/Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z").await?;

	return Ok(());
}
