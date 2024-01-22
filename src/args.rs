use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "WabbaRust")]
#[command(author = "Aidan M. <aidan@muttleyville.org>")]
#[command(version)]
#[command(about = "An automated modlist installer CLI")]
pub struct Args {
	#[command(subcommand)]
	pub command: Commands,

	#[arg(long)]
	pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
	/// Install a modlist
	Install {
		/// Either the name of an official modlist or path to a custom manifest file
		modlist: std::path::PathBuf,
	},

	/// Repair an installed modlist
	Repair {
		/// Folder where modlist is installed
		modlist: std::path::PathBuf
	},

	/// Modify WabbaRust settings
	Config {
		/// Config option
		option: String,

		/// New value
		value: String
	}
}
