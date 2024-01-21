use std::path::PathBuf;

use wabbarust::WabbaRust;
use wabbarust::download::nexus::NexusDownload;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let install_folder: PathBuf = [std::env::current_dir()?, PathBuf::from("install")].iter().collect();
	let wr = WabbaRust::new(String::from("Aeby8A0QzwcCnVrkMXjqf3Id38TRmVrnB9YDI9o9kAx4--Izy5qNKLuv2xBfl3--ZbD0azECEsornaIWSYxZgA=="), install_folder, None);
	wr.init()?;

	let ussep = NexusDownload::new(
		String::from("Unofficial Skyrim Special Edition Patch-266-4-3-0a-1702019266.7z"),
		String::from("4.3.0a"),
		String::from("266"),
		String::from("skyrimspecialedition"),
		String::from("c75d8dd9478fb7fc685507c01313f36db909965c409082f23a14c2031d36ba6a")
	);

	ussep.download(wr.get_download_dir(), wr.get_api_key()).await?;

	return Ok(());
}
