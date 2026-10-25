use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Defines the supported data formats for input and output streams.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum DataFormat {
    /// Comma-Separated Values format.
    Csv,
    /// JavaScript Object Notation format.
    Json,
}

/// A high-performance data stream processor for filtering and transforming large datasets.
///
/// This tool reads data from an input file, applies a series of declarative transformations,
/// and writes the result to an output file or stdout.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the input data file (e.g., a large CSV or JSON file).
    #[arg(short, long, value_name = "FILE_PATH")]
    pub input_file: PathBuf,

    /// Path to the output file. If not provided, results will be written to stdout.
    #[arg(short, long, value_name = "FILE_PATH")]
    pub output_file: Option<PathBuf>,

    /// The format of the input data.
    #[arg(short, long, value_enum)]
    pub format: DataFormat,
}
