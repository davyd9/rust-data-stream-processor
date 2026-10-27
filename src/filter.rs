use crate::error::AppError;
use serde_json::Value;

/// Represents a simple key-value filter condition.
#[derive(Debug, Clone)]
pub struct FilterCondition {
    key: String,
    value: String,
}

impl FilterCondition {
    /// Parses a filter string of the format "key=value" into a `FilterCondition`.
    pub fn new(expression: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = expression.splitn(2, '=').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AppError::InvalidFilter(format!(
                "Invalid filter expression. Expected format: 'key=value', but got '{}'",
                expression
            )));
        }
        Ok(FilterCondition {
            key: parts[0].to_string(),
            value: parts[1].to_string(),
        })
    }

    /// Applies the filter condition to a given JSON value.
    /// Returns `true` if the record matches the condition, `false` otherwise.
    /// The check is case-sensitive and compares against string representations of values.
    pub fn apply(&self, record: &Value) -> bool {
        if let Some(val) = record.get(&self.key) {
            return match val {
                Value::String(s) => s == &self.value,
                Value::Number(n) => n.to_string() == self.value,
                Value::Bool(b) => b.to_string() == self.value,
                _ => false, // Null, Array, Object are not matched
            };
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_filter_condition_success() {
        let condition = FilterCondition::new("user.id=123").unwrap();
        assert_eq!(condition.key, "user.id");
        assert_eq!(condition.value, "123");
    }

    #[test]
    fn test_new_filter_condition_invalid_format() {
        assert!(FilterCondition::new("user.id:123").is_err());
        assert!(FilterCondition::new("user.id=").is_err());
        assert!(FilterCondition::new("=123").is_err());
        assert!(FilterCondition::new("user.id").is_err());
    }

    #[test]
    fn test_apply_filter_matches_string() {
        let condition = FilterCondition::new("status=active").unwrap();
        let record = json!({ "id": 1, "status": "active" });
        assert!(condition.apply(&record));
    }

    #[test]
    fn test_apply_filter_matches_number() {
        let condition = FilterCondition::new("id=42").unwrap();
        let record = json!({ "id": 42, "status": "active" });
        assert!(condition.apply(&record));
    }

    #[test]
    fn test_apply_filter_no_match_value() {
        let condition = FilterCondition::new("status=inactive").unwrap();
        let record = json!({ "id": 1, "status": "active" });
        assert!(!condition.apply(&record));
    }

    #[test]
    fn test_apply_filter_no_match_key() {
        let condition = FilterCondition::new("user_id=1").unwrap();
        let record = json!({ "id": 1, "status": "active" });
        assert!(!condition.apply(&record));
    }
}
