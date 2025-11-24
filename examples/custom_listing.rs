//! Example of using SAP with custom configuration

use sap::{Config, Core, Display, Flags, IconOption, IconSeparator, IconTheme, Layout, Recursion, Sorting};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get directory to list from command line args
    let paths: Vec<PathBuf> = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect();

    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths
    };

    // Load configuration from file (or use defaults if no file exists)
    let _config = Config::default();

    // Create custom flags
    let mut flags = Flags::default();

    // Configure display settings
    flags.display = Display::All; // Show all files including hidden
    flags.layout = Layout::Tree; // Use tree layout

    // Configure icons using flags::Icons
    flags.icons = sap::flags::Icons {
        when: IconOption::Always,
        theme: IconTheme::Fancy,
        separator: IconSeparator(" ".into()),
    };

    // Configure recursion
    flags.recursion = Recursion {
        enabled: true,
        depth: 3, // Maximum depth of 3 levels
    };

    // Configure sorting
    flags.sorting = Sorting::default();

    // Create the core SAP processor with custom flags
    let core = Core::new(flags);

    // Run the listing
    let exit_code = core.run(paths).await;

    std::process::exit(exit_code as i32);
}
