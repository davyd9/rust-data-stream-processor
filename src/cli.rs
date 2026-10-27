use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Defines the command-line interface for the data stream processor.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input file path. If not provided, reads from stdin.
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// Output file path. If not provided, writes to stdout.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// The format of the input data.
    #[arg(short, long, value_enum)]
    pub format: DataFormat,

    /// A simple filter expression in "key=value" format.
    #[arg(long)]
    pub filter: Option<String>,
}

/// Enum representing the supported input data formats.
#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum DataFormat {
    Json,
    Csv,
}
