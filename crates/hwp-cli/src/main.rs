// crates/hwp-cli/src/main.rs

//! HWP CLI - Command-line interface for HWP file operations

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use hwp_core::{
    HwpTextExtractor, converter::to_semantic_markdown, export::parse_structured_document,
    parser::SectionLimits,
};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::fmt;

use serde_json::json;

fn is_hwpx_path(p: &Path) -> bool {
    p.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("hwpx"))
        .unwrap_or(false)
}

fn exit_unsupported_hwpx(p: &Path) -> ! {
    let msg =
        "UNSUPPORTED_FORMAT: HWPX (.hwpx) is not supported yet. Supported: HWP v5 (.hwp, OLE/CFB).";
    eprintln!(
        "{}",
        json!({
            "error": {
                "code": "UNSUPPORTED_FORMAT",
                "message": msg,
                "details": {
                    "format": "hwpx",
                    "supported": ["hwp"],
                    "input": p.display().to_string(),
                    "hint": "Convert .hwpx to .hwp or wait for HWPX support (planned)."
                }
            }
        })
    );
    std::process::exit(2);
}

fn ensure_supported_input_or_exit(p: &Path) {
    if is_hwpx_path(p) {
        exit_unsupported_hwpx(p);
    }
}

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

        /// Max decompressed bytes per section (safety)
        #[arg(long, default_value_t = 67108864)]
        max_decompressed_bytes_per_section: usize,

        /// Max records per section (safety)
        #[arg(long, default_value_t = 200000)]
        max_records_per_section: usize,
    },

    /// Show information about HWP file
    Info {
        /// Input HWP file path
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Max decompressed bytes per section (safety)
        #[arg(long, default_value_t = 67108864)]
        max_decompressed_bytes_per_section: usize,

        /// Max records per section (safety)
        #[arg(long, default_value_t = 200000)]
        max_records_per_section: usize,
    },

    /// Export a deterministic StructuredDocument JSON.
    Json {
        /// Input HWP file path
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Pretty-print JSON
        #[arg(long, default_value_t = false)]
        pretty: bool,

        /// Max decompressed bytes per section (safety)
        #[arg(long, default_value_t = 67108864)]
        max_decompressed_bytes_per_section: usize,

        /// Max records per section (safety)
        #[arg(long, default_value_t = 200000)]
        max_records_per_section: usize,
    },

    /// Export semantic markdown derived from StructuredDocument.
    Markdown {
        /// Input HWP file path
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Max decompressed bytes per section (safety)
        #[arg(long, default_value_t = 67108864)]
        max_decompressed_bytes_per_section: usize,

        /// Max records per section (safety)
        #[arg(long, default_value_t = 200000)]
        max_records_per_section: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    fmt().with_env_filter(log_level).with_target(false).init();

    match cli.command {
        Commands::Extract {
            input,
            output,
            max_decompressed_bytes_per_section,
            max_records_per_section,
        } => {
            ensure_supported_input_or_exit(&input);
            extract_text(
                &input,
                output.as_deref(),
                max_decompressed_bytes_per_section,
                max_records_per_section,
            )?;
        }
        Commands::Info {
            input,
            max_decompressed_bytes_per_section,
            max_records_per_section,
        } => {
            ensure_supported_input_or_exit(&input);
            show_info(
                &input,
                max_decompressed_bytes_per_section,
                max_records_per_section,
            )?;
        }
        Commands::Json {
            input,
            output,
            pretty,
            max_decompressed_bytes_per_section,
            max_records_per_section,
        } => {
            ensure_supported_input_or_exit(&input);
            export_json(
                &input,
                output.as_deref(),
                pretty,
                max_decompressed_bytes_per_section,
                max_records_per_section,
            )?;
        }
        Commands::Markdown {
            input,
            output,
            max_decompressed_bytes_per_section,
            max_records_per_section,
        } => {
            ensure_supported_input_or_exit(&input);
            export_markdown(
                &input,
                output.as_deref(),
                max_decompressed_bytes_per_section,
                max_records_per_section,
            )?;
        }
    }

    Ok(())
}

/// Extract text from HWP file
fn extract_text(
    input: &Path,
    output: Option<&std::path::Path>,
    max_decompressed_bytes_per_section: usize,
    max_records_per_section: usize,
) -> Result<()> {
    info!("Extracting text from: {}", input.display());

    let file =
        File::open(input).with_context(|| format!("Failed to open file: {}", input.display()))?;

    let reader = BufReader::new(file);
    let limits = SectionLimits {
        max_decompressed_bytes: max_decompressed_bytes_per_section,
        max_records: max_records_per_section,
    };

    let mut extractor = HwpTextExtractor::open(reader)
        .with_context(|| format!("Failed to parse HWP file: {}", input.display()))?
        .with_limits(limits);

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
fn show_info(
    input: &Path,
    _max_decompressed_bytes_per_section: usize,
    _max_records_per_section: usize,
) -> Result<()> {
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

fn derive_title_from_path(input: &Path) -> String {
    input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

fn export_json(
    input: &Path,
    output: Option<&std::path::Path>,
    pretty: bool,
    max_decompressed_bytes_per_section: usize,
    max_records_per_section: usize,
) -> Result<()> {
    info!("Exporting structured JSON from: {}", input.display());

    let file =
        File::open(input).with_context(|| format!("Failed to open file: {}", input.display()))?;
    let reader = BufReader::new(file);

    let limits = SectionLimits {
        max_decompressed_bytes: max_decompressed_bytes_per_section,
        max_records: max_records_per_section,
    };

    let title = Some(derive_title_from_path(input));
    let doc = parse_structured_document(reader, title, limits)
        .with_context(|| format!("Failed to parse HWP file: {}", input.display()))?;

    let json = if pretty {
        serde_json::to_string_pretty(&doc)?
    } else {
        serde_json::to_string(&doc)?
    };

    match output {
        Some(path) => {
            let mut file = File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?;
            file.write_all(json.as_bytes())?;
            file.write_all(b"\n")?;
        }
        None => {
            print!("{}", json);
            println!();
        }
    }

    Ok(())
}

fn export_markdown(
    input: &Path,
    output: Option<&std::path::Path>,
    max_decompressed_bytes_per_section: usize,
    max_records_per_section: usize,
) -> Result<()> {
    info!("Exporting semantic markdown from: {}", input.display());

    let file =
        File::open(input).with_context(|| format!("Failed to open file: {}", input.display()))?;
    let reader = BufReader::new(file);

    let limits = SectionLimits {
        max_decompressed_bytes: max_decompressed_bytes_per_section,
        max_records: max_records_per_section,
    };

    let title = Some(derive_title_from_path(input));
    let doc = parse_structured_document(reader, title, limits)
        .with_context(|| format!("Failed to parse HWP file: {}", input.display()))?;

    let md = to_semantic_markdown(&doc);

    match output {
        Some(path) => {
            let mut file = File::create(path)
                .with_context(|| format!("Failed to create output file: {}", path.display()))?;
            file.write_all(md.as_bytes())?;
        }
        None => {
            print!("{}", md);
        }
    }

    Ok(())
}