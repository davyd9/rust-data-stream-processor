use crate::cli::Cli;
use crate::error::AppError;
use log::{debug, info};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Asynchronously opens the input file specified in the CLI arguments and processes it line by line.
///
/// This function is designed for memory efficiency, as it reads the file as a stream
/// and does not load the entire content into memory. It logs the number of lines processed.
///
/// # Arguments
///
/// * `cli` - A reference to the parsed command-line arguments containing the input file path.
///
/// # Returns
///
/// A `Result` which is `Ok(())` on successful processing of the entire file, or an `AppError`
/// if any I/O error occurs.
pub async fn process_stream(cli: &Cli) -> Result<(), AppError> {
    info!("Starting to process stream from file: '{}'", cli.input);

    let file = File::open(&cli.input).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut line_count: u64 = 0;
    while let Some(line) = lines.next_line().await? {
        // Placeholder for future data transformation and filtering logic
        debug!("Read line [{}]: {}", line_count + 1, line);
        line_count += 1;
    }

    info!(
        "Successfully processed {} lines from '{}'",
        line_count,
        &cli.input
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Format};
    use std::path::PathBuf;
    use uuid::Uuid;

    /// Creates a temporary file with specified content for testing purposes.
    async fn create_temp_test_file(content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let file_name = format!("test-{}.log", Uuid::new_v4());
        let file_path = temp_dir.join(file_name);
        tokio::fs::write(&file_path, content).await.unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_process_stream_success() {
        // Arrange
        let file_content = "line 1\nline 2\nline 3";
        let file_path = create_temp_test_file(file_content).await;
        let cli = Cli {
            input: file_path.to_str().unwrap().to_string(),
            output: None,
            format: Format::Json,
        };

        // Act
        let result = process_stream(&cli).await;

        // Assert
        assert!(result.is_ok());

        // Cleanup
        tokio::fs::remove_file(file_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_process_stream_empty_file() {
        // Arrange
        let file_path = create_temp_test_file("").await;
        let cli = Cli {
            input: file_path.to_str().unwrap().to_string(),
            output: None,
            format: Format::Json,
        };

        // Act
        let result = process_stream(&cli).await;

        // Assert
        assert!(result.is_ok());

        // Cleanup
        tokio::fs::remove_file(file_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_process_stream_file_not_found() {
        // Arrange
        let non_existent_file = "/tmp/non_existent_file_12345.log";
        let cli = Cli {
            input: non_existent_file.to_string(),
            output: None,
            format: Format::Json,
        };

        // Act
        let result = process_stream(&cli).await;

        // Assert
        assert!(result.is_err());
        if let Err(AppError::Io(e)) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        } else {
            panic!("Expected an I/O error for a non-existent file");
        }
    }
}
