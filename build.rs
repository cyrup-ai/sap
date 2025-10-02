use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::*;
use std::fs;
use std::process::exit;

include!("src/app.rs");

fn main() {
    let outdir = std::env::var_os("SHELL_COMPLETIONS_DIR")
        .or_else(|| std::env::var_os("OUT_DIR"))
        .unwrap_or_else(|| exit(0));

    if let Err(err) = fs::create_dir_all(&outdir) {
        eprintln!("cargo:warning=Failed to create completion directory: {}", err);
        return;
    }

    let mut app = Cli::command();
    let bin_name = "lsd";
    
    if let Err(err) = generate_to(Bash, &mut app, bin_name, &outdir) {
        eprintln!("cargo:warning=Failed to generate Bash completions: {}", err);
    }
    if let Err(err) = generate_to(Fish, &mut app, bin_name, &outdir) {
        eprintln!("cargo:warning=Failed to generate Fish completions: {}", err);
    }
    if let Err(err) = generate_to(Zsh, &mut app, bin_name, &outdir) {
        eprintln!("cargo:warning=Failed to generate Zsh completions: {}", err);
    }
    if let Err(err) = generate_to(PowerShell, &mut app, bin_name, &outdir) {
        eprintln!("cargo:warning=Failed to generate PowerShell completions: {}", err);
    }

    // Disable git feature for these target where git2 is not well supported
    if !std::env::var("CARGO_FEATURE_GIT2")
        .map(|flag| flag == "1")
        .unwrap_or(false)
        || std::env::var("TARGET")
            .map(|target| target == "i686-pc-windows-gnu")
            .unwrap_or(false)
    {
        println!(r#"cargo:rustc-cfg=feature="no-git""#);
    }
}
