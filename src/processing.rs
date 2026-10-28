//! Core data processing logic for streaming and transforming records.
//!
//! This module orchestrates the reading, parsing, filtering, and writing of
//! data from different sources (JSONL, CSV). It leverages an abstraction layer
//! (`model::Record`) to apply filters with minimal data copying, optimizing
//! performance by delaying expensive transformations (like CSV-to-JSON conversion)
//! until after a record has been approved by the filter.

use crate::cli::Format;
use crate::error::AppError;
use crate::filter::FilterCondition;
use crate::model::{CsvRecord, JsonRecord};
use anyhow::Result;
use csv_async::{AsyncReaderBuilder, StringRecord};
use futures::{StreamExt, TryStreamExt};
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncBufRead, AsyncWrite, AsyncWriteExt};

/// Processes a stream of data based on the specified format.
///
/// It dispatches to format-specific processing functions.
pub async fn process_stream<R, W>(
    reader: R,
    writer: W,
    format: Format,
    filter: Option<FilterCondition>,
) -> Result<()>
where
    R: AsyncBufRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
{
    match format {
        Format::Jsonl => process_jsonl(reader, writer, filter).await?,
        Format::Csv => process_csv(reader, writer, filter).await?,
    }
    Ok(())
}

/// Processes a stream of JSONL data.
///
/// Each line is parsed into a `serde_json::Value`. A `JsonRecord` wrapper is
/// used to apply the filter efficiently. Records that pass the filter are
/// serialized back to JSON and written to the output.
async fn process_jsonl<R, W>(
    reader: R,
    mut writer: W,
    filter: Option<FilterCondition>,
) -> Result<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    use tokio::io::AsyncBufReadExt;
    let mut lines = reader.lines();

    while let Some(line_result) = lines.next_line().await? {
        let line = line_result;
        if line.trim().is_empty() {
            continue;
        }

        let json_value: Value = serde_json::from_str(&line).map_err(AppError::from)?;

        let should_write = match &filter {
            Some(f) => {
                let record_wrapper = JsonRecord::new(&json_value);
                f.apply(&record_wrapper)
            }
            None => true,
        };

        if should_write {
            let mut output_line = serde_json::to_string(&json_value)?;
            output_line.push('\n');
            writer.write_all(output_line.as_bytes()).await?;
        }
    }
    writer.flush().await?;
    Ok(())
}

/// Processes a stream of CSV data.
///
/// This function reads the header row first to build a name-to-index map.
/// For each subsequent record, it creates a zero-copy `CsvRecord` wrapper to
/// apply the filter. The full CSV-to-JSON conversion only occurs for records
/// that pass the filter, significantly improving performance.
async fn process_csv<R, W>(
    reader: R,
    mut writer: W,
    filter: Option<FilterCondition>,
) -> Result<()>
where
    R: AsyncBufRead + Unpin + Send,
    W: AsyncWrite + Unpin,
{
    let mut csv_reader = AsyncReaderBuilder::new().create_reader(reader);

    let headers = csv_reader.headers().await?.clone();
    let header_map: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.to_string(), i))
        .collect();

    let mut records = csv_reader.records();

    while let Some(record_result) = records.next().await {
        let record = record_result?;
        let should_write = match &filter {
            Some(f) => {
                let record_wrapper = CsvRecord::new(&header_map, &record);
                f.apply(&record_wrapper)
            }
            None => true,
        };

        if should_write {
            let json_value = record_to_json_value(&headers, &record)?;
            let mut output_line = serde_json::to_string(&json_value)?;
            output_line.push('\n');
            writer.write_all(output_line.as_bytes()).await?;
        }
    }
    writer.flush().await?;
    Ok(())
}

/// Converts a CSV `StringRecord` to a `serde_json::Value` using the headers.
fn record_to_json_value(
    headers: &StringRecord,
    record: &StringRecord,
) -> Result<Value, AppError> {
    if headers.len() != record.len() {
        return Err(AppError::RecordProcessingError(format!(
            "CSV record has {} fields, but header has {}",
            record.len(),
            headers.len()
        )));
    }

    let map: serde_json::Map<String, Value> = headers
        .iter()
        .zip(record.iter())
        .map(|(header, field)| (header.to_string(), Value::String(field.to_string())))
        .collect();

    Ok(Value::Object(map))
}
