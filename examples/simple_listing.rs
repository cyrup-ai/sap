//! Simple example of using SAP as a library to list directory contents

use sap::{Config, Core, Flags};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get directory to list from command line args, or use current directory
    let paths: Vec<PathBuf> = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect();

    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Load default configuration
    let _config = Config::default();

    // Create flags with default settings
    let flags = Flags::default();

    // Create the core SAP processor
    let core = Core::new(flags);

    // Run the listing
    let exit_code = core.run(paths).await;

    std::process::exit(exit_code as i32);
}
