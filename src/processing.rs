use crate::cli::DataFormat;
use crate::error::AppError;
use crate::filter::FilterCondition;
use futures::stream::StreamExt;
use serde_json::Value;
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufWriter};

/// Processes the input stream based on the specified data format, applies an optional filter,
/// and writes the results to the output stream as JSONL.
pub async fn process_stream<R, W>(
    reader: R,
    writer: W,
    format: DataFormat,
    filter_expr: Option<String>,
) -> Result<(), AppError>
where
    R: AsyncBufRead + Unpin + Send,
    W: AsyncWrite + Unpin + Send,
{
    let filter = filter_expr
        .map(|expr| FilterCondition::new(&expr))
        .transpose()?;

    let mut buf_writer = BufWriter::new(writer);

    match format {
        DataFormat::Json => process_json_stream(reader, &mut buf_writer, filter.as_ref()).await?,
        DataFormat::Csv => process_csv_stream(reader, &mut buf_writer, filter.as_ref()).await?,
    }

    buf_writer.flush().await?;
    Ok(())
}

/// Processes a JSONL (JSON Lines) stream.
async fn process_json_stream<R, W>(
    reader: R,
    writer: &mut W,
    filter: Option<&FilterCondition>,
) -> Result<(), AppError>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = reader.lines();
    while let Some(line_result) = lines.next_line().await? {
        if line_result.trim().is_empty() {
            continue;
        }

        let record: Value = serde_json::from_str(&line_result)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        if filter.map_or(true, |f| f.apply(&record)) {
            let mut output_line = serde_json::to_string(&record)?;
            output_line.push('\n');
            writer.write_all(output_line.as_bytes()).await?;
        }
    }
    Ok(())
}

/// Processes a CSV stream.
async fn process_csv_stream<R, W>(
    reader: R,
    writer: &mut W,
    filter: Option<&FilterCondition>,
) -> Result<(), AppError>
where
    R: AsyncBufRead + Unpin + Send,
    W: AsyncWrite + Unpin,
{
    let mut csv_reader = csv_async::AsyncReaderBuilder::new()
        .has_headers(true)
        .create_reader(reader);

    let headers = csv_reader
        .headers()
        .await?
        .iter()
        .map(String::from)
        .collect::<Vec<String>>();

    let mut records = csv_reader.records();
    while let Some(record_result) = records.next().await {
        let record = record_result?;
        let mut json_map = serde_json::Map::new();

        for (i, value) in record.iter().enumerate() {
            if let Some(header) = headers.get(i) {
                json_map.insert(header.clone(), Value::String(value.to_string()));
            }
        }

        let json_value = Value::Object(json_map);

        if filter.map_or(true, |f| f.apply(&json_value)) {
            let mut output_line = serde_json::to_string(&json_value)?;
            output_line.push('\n');
            writer.write_all(output_line.as_bytes()).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::DataFormat;
    use std::io::Cursor;

    async fn run_processing_test(
        input: &str,
        format: DataFormat,
        filter: Option<String>,
        expected: &str,
    ) {
        let reader = io::BufReader::new(Cursor::new(input.as_bytes()));
        let mut writer = Vec::new();

        process_stream(reader, &mut writer, format, filter)
            .await
            .unwrap();

        let output = String::from_utf8(writer).unwrap();
        assert_eq!(output, expected);
    }

    #[tokio::test]
    async fn test_process_json_stream_no_filter() {
        let input = r#"{"id":1,"name":"A"}
{"id":2,"name":"B"}"#;
        let expected = "{\"id\":1,\"name\":\"A\"}\n{\"id\":2,\"name\":\"B\"}\n";
        run_processing_test(input, DataFormat::Json, None, expected).await;
    }

    #[tokio::test]
    async fn test_process_json_stream_with_filter() {
        let input = "{\"id\":1,\"status\":\"active\"}\n{\"id\":2,\"status\":\"inactive\"}\n{\"id\":3,\"status\":\"active\"}";
        let expected = "{\"id\":1,\"status\":\"active\"}\n{\"id\":3,\"status\":\"active\"}\n";
        run_processing_test(
            input,
            DataFormat::Json,
            Some("status=active".to_string()),
            expected,
        )
        .await;
    }

    #[tokio::test]
    async fn test_process_csv_stream_no_filter() {
        let input = "id,name,value\n1,A,100\n2,B,200";
        let expected = "{\"id\":\"1\",\"name\":\"A\",\"value\":\"100\"}\n{\"id\":\"2\",\"name\":\"B\",\"value\":\"200\"}\n";
        run_processing_test(input, DataFormat::Csv, None, expected).await;
    }

    #[tokio::test]
    async fn test_process_csv_stream_with_filter() {
        let input = "id,city,status\n1,NY,processed\n2,LA,pending\n3,SF,processed";
        let expected = "{\"city\":\"NY\",\"id\":\"1\",\"status\":\"processed\"}\n{\"city\":\"SF\",\"id\":\"3\",\"status\":\"processed\"}\n";
        run_processing_test(
            input,
            DataFormat::Csv,
            Some("status=processed".to_string()),
            expected,
        )
        .await;
    }
}
