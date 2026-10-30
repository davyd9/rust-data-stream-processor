use rust_data_stream_processor::{
    cli::{self, FileFormat},
    error::AppError,
    filter::FilterCondition,
    processing,
};
use std::io::Write;
use tempfile::NamedTempFile;

async fn run_process_stream(
    content: &str,
    format: FileFormat,
    filter: Option<FilterCondition>,
) -> Result<String, AppError> {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    let mut output_buffer = Vec::new();
    let cli_args = cli::Cli {
        file_path: temp_file.path().to_path_buf(),
        file_format: Some(format),
        filter,
    };

    processing::process_stream(cli_args, &mut output_buffer).await?;

    Ok(String::from_utf8(output_buffer).expect("Output is not valid UTF-8"))
}

#[tokio::test]
async fn test_csv_processing_with_filter() {
    let csv_content = "id,name,city\n1,John,London\n2,Jane,Berlin\n3,Peter,London";
    let filter = FilterCondition::new("city=Berlin").unwrap();
    let result = run_process_stream(csv_content, FileFormat::Csv, Some(filter)).await;

    let expected_json = serde_json::json!({ "id": "2", "name": "Jane", "city": "Berlin" }).to_string() + "\n";
    assert_eq!(result.unwrap(), expected_json);
}

#[tokio::test]
async fn test_csv_processing_no_filter() {
    let csv_content = "id,city\n1,London\n2,Berlin";
    let result = run_process_stream(csv_content, FileFormat::Csv, None).await;

    let expected_json1 = serde_json::json!({ "id": "1", "city": "London" }).to_string();
    let expected_json2 = serde_json::json!({ "id": "2", "city": "Berlin" }).to_string();
    let expected_output = format!("{}\n{}\n", expected_json1, expected_json2);

    assert_eq!(result.unwrap(), expected_output);
}

#[tokio::test]
async fn test_jsonl_processing_with_filter() {
    let jsonl_content = r#"{"id":1,"city":"London"}
{"id":2,"city":"Berlin"}
{"id":3,"city":"London"}"#;
    let filter = FilterCondition::new("city=Berlin").unwrap();
    let result = run_process_stream(jsonl_content, FileFormat::Jsonl, Some(filter)).await;

    let expected_json = serde_json::json!({ "id": 2, "city": "Berlin" }).to_string() + "\n";
    assert_eq!(result.unwrap(), expected_json);
}

#[tokio::test]
async fn test_csv_processing_error_on_mismatched_columns() {
    let csv_content = "id,name\n1,John\n2"; // Second data row has fewer columns than header
    let result = run_process_stream(csv_content, FileFormat::Csv, None).await;

    assert!(matches!(result, Err(AppError::RecordProcessingError(_))));
}

#[tokio::test]
async fn test_no_matching_records_produces_empty_output() {
    let csv_content = "id,name,city\n1,John,London\n2,Jane,Paris";
    let filter = FilterCondition::new("city=Berlin").unwrap();
    let result = run_process_stream(csv_content, FileFormat::Csv, Some(filter)).await;

    assert_eq!(result.unwrap(), "");
}