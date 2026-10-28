//! Provides data filtering capabilities based on key-value conditions.
//!
//! This module defines the `FilterCondition` struct, which represents a simple
//! `key=value` filter. The `apply` method is implemented generically over any
//! type that satisfies the `Record` trait, allowing for efficient, zero-copy
//! filtering across different data formats.

use crate::error::AppError;
use crate::model::Record;

/// Represents a `key=value` filter condition.
#[derive(Debug, Clone)]
pub struct FilterCondition {
    key: String,
    value: String,
}

impl FilterCondition {
    /// Parses a filter string in `key=value` format into a `FilterCondition`.
    ///
    /// # Arguments
    /// * `expr` - The filter expression string.
    ///
    /// # Errors
    /// Returns `AppError::InvalidFilter` if the expression format is invalid.
    pub fn new(expr: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = expr.splitn(2, '=').collect();
        if let [key, value] = parts.as_slice() {
            Ok(FilterCondition {
                key: key.to_string(),
                value: value.to_string(),
            })
        } else {
            Err(AppError::InvalidFilter(expr.to_string()))
        }
    }

    /// Applies the filter condition to a given record.
    ///
    /// This method is generic over any type `R` that implements the `Record` trait.
    /// It checks if the record contains a field matching the filter's key and if
    /// its string representation matches the filter's value. This comparison is
    /// case-sensitive and strictly string-based for performance.
    ///
    /// # Arguments
    /// * `record` - A reference to an object implementing the `Record` trait.
    ///
    /// # Returns
    /// `true` if the record matches the condition, `false` otherwise.
    pub fn apply<'a, R: Record<'a>>(&self, record: &'a R) -> bool {
        record
            .get_field_as_str(&self.key)
            .map_or(false, |v| v == self.value)
    }
}
