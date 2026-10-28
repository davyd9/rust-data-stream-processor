//! Defines the data record abstraction for zero-copy processing.
//!
//! This module introduces the `Record` trait, which provides a generic interface
//! for accessing fields from different data formats (like JSON or CSV) without
//! needing to parse them into a single, owned intermediate representation.

use csv_async::StringRecord;
use serde_json::Value;
use std::collections::HashMap;

/// A trait for accessing fields from a data record in a format-agnostic way.
///
/// This enables zero-copy or minimal-copy filtering by allowing direct access
/// to field values as string slices (`&str`).
pub trait Record<'a> {
    /// Retrieves the value of a field as a string slice, if it exists and can be
    /// represented as a string.
    ///
    /// # Arguments
    /// * `key` - The name of the field to retrieve.
    fn get_field_as_str(&self, key: &str) -> Option<&'a str>;
}

/// A wrapper for `serde_json::Value` that implements the `Record` trait.
pub struct JsonRecord<'a> {
    value: &'a Value,
}

impl<'a> JsonRecord<'a> {
    /// Creates a new `JsonRecord` from a `serde_json::Value` reference.
    pub fn new(value: &'a Value) -> Self {
        Self { value }
    }
}

impl<'a> Record<'a> for JsonRecord<'a> {
    /// Accesses a field from the underlying JSON object.
    ///
    /// Returns the value as a `&str` only if the field exists and is a JSON string.
    /// Other JSON types (Number, Bool, etc.) will not match and will return `None`.
    fn get_field_as_str(&self, key: &str) -> Option<&'a str> {
        self.value
            .as_object()
            .and_then(|obj| obj.get(key))
            .and_then(|val| val.as_str())
    }
}

/// A wrapper for a CSV `StringRecord` that implements the `Record` trait.
///
/// It holds references to the CSV headers and the current record to enable
/// field lookups by name. This is a zero-copy implementation for filtering.
pub struct CsvRecord<'a> {
    header_map: &'a HashMap<String, usize>,
    record: &'a StringRecord,
}

impl<'a> CsvRecord<'a> {
    /// Creates a new `CsvRecord`.
    ///
    /// # Arguments
    /// * `header_map` - A map from header names to their column index.
    /// * `record` - A reference to the current `csv_async::StringRecord`.
    pub fn new(header_map: &'a HashMap<String, usize>, record: &'a StringRecord) -> Self {
        Self { header_map, record }
    }
}

impl<'a> Record<'a> for CsvRecord<'a> {
    /// Accesses a field from the underlying CSV record by its header name.
    ///
    /// It looks up the column index from the header map and returns the corresponding
    /// field from the record as a `&str`.
    fn get_field_as_str(&self, key: &str) -> Option<&'a str> {
        self.header_map
            .get(key)
            .and_then(|&index| self.record.get(index))
    }
}
