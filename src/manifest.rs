use serde::{Serialize, Deserialize};

use crate::download::nexus::NexusDownload;

#[derive(Serialize, Deserialize)]
struct Manifest {
	nexus_downloads: Vec<NexusDownload>
}
