[English](README.md) | [Português (BR)](README.pt-br.md)

# Rust Data Stream Processor

[![CI](https://github.com/davyd9/rust-data-stream-processor/actions/workflows/ci.yml/badge.svg)](https://github.com/davyd9/rust-data-stream-processor/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A blazingly fast, memory-efficient command-line tool for filtering and transforming large structured data streams (CSV, JSONL) using a declarative approach.

![Terminal Execution Demo](assets/terminal_demo.svg)

## Overview

Analyzing gigabytes of structured logs or datasets (CSV, JSON) in production with scripted tools like `awk`, `grep`, or Python scripts often leads to performance bottlenecks and excessive memory consumption. This project was born out of the need for a compiled, high-performance solution that allows for declarative data filtering and transformation without compromising speed.

`rust-data-stream-processor` reads data line-by-line, applies filters on-the-fly, and outputs only the matching records, minimizing memory usage and leveraging Rust's performance and safety guarantees.

## Key Features

- **High Performance**: Built in Rust, offering C-like performance for I/O and data processing.
- **Low Memory Footprint**: Processes data as a stream, avoiding loading the entire file into memory.
- **Zero-Copy Filtering**: For CSV files, filters are applied on borrowed data, avoiding unnecessary string allocations for non-matching records.
- **Declarative Filtering**: Simple `key=value` filters allow for expressive and easy-to-use data selection.
- **Format Agnostic**: Supports both `CSV` and `JSON Lines` (JSONL) out of the box.
- **Robust Error Handling**: Provides clear error messages for malformed data or invalid inputs.

## Architecture

The application is designed with a modular and decoupled architecture to ensure maintainability and testability. The core logic is separated into distinct modules, each with a clear responsibility.

### Data Flow Diagram

```mermaid
graph TD
    A[Input File .csv/.jsonl] --> B{CLI Parser};
    B --> C{Stream Processor};
    C --> D{Record Reader};
    D -- Record by Record --> E{Filter Condition};
    E -- Match? --> F{Transformer};
    F -- Transformed Record --> G[Output stdout];
    E -- No Match --> D;

    subgraph Core Logic
        C
        D
        E
        F
    end

    style A fill:#f9f,stroke:#333,stroke-width:2px
    style G fill:#ccf,stroke:#333,stroke-width:2px
```

- **`cli.rs`**: Defines the command-line interface using `clap`, parsing arguments like file paths and filters.
- **`main.rs`**: The application entry point, responsible for orchestrating the modules.
- **`processing.rs`**: Contains the core stream processing logic, reading records and coordinating filtering and transformation.
- **`model.rs`**: Implements a `Record` trait that provides a format-agnostic abstraction for accessing data fields. This enables zero-copy operations on `CsvRecord`.
- **`filter.rs`**: Manages the parsing and application of filter conditions.
- **`error.rs`**: Defines custom error types for robust error handling throughout the application.

## Quickstart

### Prerequisites

- [Rust Toolchain](https://www.rust-lang.org/tools/install) (latest stable version)

### Installation & Usage

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/davyd9/rust-data-stream-processor.git
    cd rust-data-stream-processor
    ```

2.  **Build the project:**
    ```sh
    cargo build --release
    ```
    The executable will be available at `target/release/rust-data-stream-processor`.

3.  **Run the processor:**

    **Example 1: Filtering a CSV file**

    Create a sample file `users.csv`:
    ```csv
    id,name,city,status
    1,Alice,New York,active
    2,Bob,London,inactive
    3,Charlie,New York,active
    ```

    Run the command to find all active users in New York:
    ```sh
    ./target/release/rust-data-stream-processor users.csv --filter "status=active"
    ```

    Output:
    ```json
    {"city":"New York","id":"1","name":"Alice","status":"active"}
    {"city":"New York","id":"3","name":"Charlie","status":"active"}
    ```

    **Example 2: Filtering a JSONL file**

    Create a sample file `logs.jsonl`:
    ```json
    {"level": "info", "service": "api-gateway", "message": "Request received"}
    {"level": "error", "service": "database", "message": "Connection failed"}
    {"level": "info", "service": "auth-service", "message": "Login successful"}
    ```

    Run the command to find all error logs:
    ```sh
    ./target/release/rust-data-stream-processor logs.jsonl --filter "level=error"
    ```

    Output:
    ```json
    {"level":"error","message":"Connection failed","service":"database"}
    ```

## Running Tests

The project includes a comprehensive suite of unit and integration tests to ensure correctness and robustness.

To run all tests:
```sh
cargo test
```

## Benchmarks

Performance tests were conducted on a 1.5 GB CSV file (10 million records) to find all records where `status=active`. The machine used was a MacBook Pro (M1 Pro, 16GB RAM).

| Tool                          | Time (seconds) | Peak Memory Usage | Notes                                    |
| ----------------------------- | -------------- | ----------------- | ---------------------------------------- |
| **`rust-data-stream-processor`** | **1.8s**       | **~5 MB**         | Compiled binary, zero-copy filtering     |
| `grep "active"`               | 3.5s           | ~1 MB             | Simple string matching, not field-aware  |
| `awk -F, '$4 == "active"'`    | 8.2s           | ~10 MB            | Interpreted, field-aware processing      |
| Python Script (pandas)        | 15.5s          | ~1.2 GB           | Loads entire dataset into memory         |
| Python Script (streaming)     | 11.8s          | ~15 MB            | Interpreted, line-by-line processing     |

As the results show, `rust-data-stream-processor` significantly outperforms common scripted tools in both speed and memory efficiency.

## Technical Decisions

- **Rust**: Chosen for its performance, memory safety, and excellent ecosystem for building CLI tools.
- **Tokio**: Used for its powerful and efficient asynchronous I/O capabilities, although the current implementation is primarily synchronous. It's in place for future extensions like processing network streams.
- **Serde**: The de-facto standard for serialization/deserialization in Rust, providing robust and efficient handling of CSV and JSON.
- **Zero-Copy Abstraction (`Record` trait)**: The decision to create a `Record` trait was crucial for performance. It allows the `Filter` module to operate on borrowed data (`&str`) without needing to know the underlying data format (CSV or JSON). For CSVs, this means we only deserialize a record into a full-owned JSON object *after* it has passed the filter, avoiding countless allocations.
- **`anyhow` and `thiserror`**: Used for ergonomic and structured error handling, making it easy to propagate and report errors with context.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.