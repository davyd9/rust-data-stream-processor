use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_cli_csv_filter_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "user_id,location,status")?;
    writeln!(file, "1,New York,active")?;
    writeln!(file, "2,Berlin,inactive")?;
    writeln!(file, "3,New York,active")?;

    let mut cmd = Command::cargo_bin("rust-data-stream-processor")?;
    cmd.arg(file.path())
        .arg("--filter")
        .arg("status=active");

    let expected_output = serde_json::json!({ "user_id": "1", "location": "New York", "status": "active" }).to_string()
        + "\n"
        + &serde_json::json!({ "user_id": "3", "location": "New York", "status": "active" }).to_string()
        + "\n";

    cmd.assert()
        .success()
        .stdout(predicate::eq(expected_output.as_str()));

    Ok(())
}

#[test]
fn test_cli_jsonl_filter_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, r#"{{"user_id":1,"location":"New York","status":"active"}}"#)?;
    writeln!(file, r#"{{"user_id":2,"location":"Berlin","status":"inactive"}}"#)?;
    writeln!(file, r#"{{"user_id":3,"location":"New York","status":"active"}}"#)?;

    let mut cmd = Command::cargo_bin("rust-data-stream-processor")?;
    cmd.arg(file.path())
        .arg("--filter")
        .arg("location=Berlin");

    let expected_output =
        r#"{"user_id":2,"location":"Berlin","status":"inactive"}"#;

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(expected_output));

    Ok(())
}

#[test]
fn test_cli_file_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rust-data-stream-processor")?;
    cmd.arg("nonexistent/file/path.csv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));

    Ok(())
}

#[test]
fn test_cli_invalid_filter_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "a,b\n1,2")?;

    let mut cmd = Command::cargo_bin("rust-data-stream-processor")?;
    cmd.arg(file.path()).arg("--filter").arg("keyonly");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid filter format"));

    Ok(())
}

#[test]
fn test_cli_auto_detect_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new_with_prefix("data_")?.into_temp_path().with_extension("csv");
    let mut csv_file = std::fs::File::create(&file)?;
    writeln!(csv_file, "id,val\n1,ok")?;

    let mut cmd = Command::cargo_bin("rust-data-stream-processor")?;
    cmd.arg(&file);

    let expected_output = serde_json::json!({ "id": "1", "val": "ok" }).to_string() + "\n";

    cmd.assert()
        .success()
        .stdout(predicate::eq(expected_output.as_str()));

    std::fs::remove_file(&file)?;

    Ok(())
}