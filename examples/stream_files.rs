//! Example of using SAP's streaming API to process files as they're discovered

use futures::StreamExt;
use sap::{Display, FileStream, IgnoreGlobs};
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

    // Create ignore globs to filter out certain directories
    let ignore_globs = IgnoreGlobs::default();

    // Create a file stream with max depth of 2
    let max_depth = 2;
    let display = Display::VisibleOnly; // Only show visible files
    let mut stream = FileStream::new(paths, max_depth, &ignore_globs, display);

    // Process files as they arrive
    let mut count = 0;
    while let Some(result) = stream.next().await {
        match result {
            Ok(entry) => {
                count += 1;
                println!(
                    "{:>4}. {} ({})",
                    count,
                    entry.path.display(),
                    if entry.file_type.is_dirlike() {
                        "directory"
                    } else {
                        "file"
                    }
                );
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    println!("\nTotal entries processed: {}", count);

    Ok(())
}
