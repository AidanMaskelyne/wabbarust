pub mod log;
pub mod download;

pub struct WabbaRust {
	api_key: String,
}

impl WabbaRust {
	pub fn init() -> anyhow::Result<()> {
		log::init(true);
		return Ok(());
	}
}
