// crates/hwp-cli/src/main.rs

//! HWP CLI - Command-line interface for HWP file operations

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hwp_core::HwpTextExtractor;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(name = "hwp")]
#[command(author, version, about = "HWP file processing CLI", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract text from HWP file
    Extract {
        /// Input HWP file path
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show information about HWP file
    Info {
        /// Input HWP file path
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    fmt().with_env_filter(log_level).with_target(false).init();

    match cli.command {
        Commands::Extract { input, output } => {
            extract_text(&input, output.as_deref())?;
        }
        Commands::Info { input } => {
            show_info(&input)?;
        }
    }

    Ok(())
}

/// Extract text from HWP file
fn extract_text(input: &PathBuf, output: Option<&std::path::Path>) -> Result<()> {
    info!("Extracting text from: {}", input.display());

    let file =
        File::open(input).with_context(|| format!("Failed to open file: {}", input.display()))?;

    let reader = BufReader::new(file);
    let mut extractor = HwpTextExtractor::open(reader)
        .with_context(|| format!("Failed to parse HWP file: {}", input.display()))?;

    let text = extractor
        .extract_all_text()
        .with_context(|| "Failed to extract text from HWP file")?;

    match output {
        Some(path) => {
            let mut file = File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?;
            file.write_all(text.as_bytes())?;
            info!("Text extracted to: {}", path.display());
        }
        None => {
            println!("{}", text);
        }
    }

    Ok(())
}

/// Show information about HWP file
fn show_info(input: &PathBuf) -> Result<()> {
    info!("Reading file info: {}", input.display());

    let file =
        File::open(input).with_context(|| format!("Failed to open file: {}", input.display()))?;

    let reader = BufReader::new(file);
    let ole = hwp_core::HwpOleFile::open(reader)
        .with_context(|| format!("Failed to parse HWP file: {}", input.display()))?;

    let header = ole.header();

    println!("HWP File Information");
    println!("====================");
    println!("Version: {}", header.version);
    println!();
    println!("Properties:");
    println!("  Compressed: {}", header.properties.is_compressed());
    println!("  Encrypted: {}", header.properties.is_encrypted());
    println!("  Distribution: {}", header.properties.is_distribution());
    println!("  Has Script: {}", header.properties.has_script());
    println!("  Has DRM: {}", header.properties.has_drm());
    println!("  Has History: {}", header.properties.has_history());
    println!("  Has Signature: {}", header.properties.has_signature());

    Ok(())
}
