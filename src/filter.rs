use crate::error::AppError;
use crate::model::Record;

#[derive(Debug, Clone)]
pub struct FilterCondition {
    key: String,
    value: String,
}

impl FilterCondition {
    pub fn new(filter_str: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = filter_str.split('=').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(AppError::InvalidFilterFormat(filter_str.to_string()));
        }
        Ok(FilterCondition {
            key: parts[0].to_string(),
            value: parts[1].to_string(),
        })
    }

    pub fn apply<'a, R: Record<'a>>(&self, record: &'a R) -> bool {
        record.get(&self.key).map_or(false, |v| v == self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // A mock record implementation for testing purposes.
    struct MockRecord<'a> {
        fields: HashMap<&'a str, &'a str>,
    }

    impl<'a> Record<'a> for MockRecord<'a> {
        fn get(&self, key: &str) -> Option<&'a str> {
            self.fields.get(key).copied()
        }
    }

    #[test]
    fn test_filter_condition_new_success() {
        let filter = FilterCondition::new("status=active").unwrap();
        assert_eq!(filter.key, "status");
        assert_eq!(filter.value, "active");
    }

    #[test]
    fn test_filter_condition_new_invalid_format() {
        assert!(matches!(
            FilterCondition::new("statusactive"),
            Err(AppError::InvalidFilterFormat(_))
        ));
        assert!(matches!(
            FilterCondition::new("status="),
            Err(AppError::InvalidFilterFormat(_))
        ));
        assert!(matches!(
            FilterCondition::new("=active"),
            Err(AppError::InvalidFilterFormat(_))
        ));
        assert!(matches!(
            FilterCondition::new("status=active=true"),
            Err(AppError::InvalidFilterFormat(_))
        ));
    }

    #[test]
    fn test_apply_filter_match() {
        let filter = FilterCondition::new("city=Berlin").unwrap();
        let record = MockRecord { fields: HashMap::from([("city", "Berlin"), ("country", "Germany")]) };
        assert!(filter.apply(&record));
    }

    #[test]
    fn test_apply_filter_no_match() {
        let filter = FilterCondition::new("city=London").unwrap();
        let record = MockRecord { fields: HashMap::from([("city", "Berlin"), ("country", "Germany")]) };
        assert!(!filter.apply(&record));
    }

    #[test]
    fn test_apply_filter_key_not_found() {
        let filter = FilterCondition::new("zip=10115").unwrap();
        let record = MockRecord { fields: HashMap::from([("city", "Berlin"), ("country", "Germany")]) };
        assert!(!filter.apply(&record));
    }

    #[test]
    fn test_apply_filter_empty_value_in_record() {
        let filter = FilterCondition::new("country=Germany").unwrap();
        let record = MockRecord { fields: HashMap::from([("city", ""), ("country", "Germany")]) };
        assert!(filter.apply(&record));
    }

    #[test]
    fn test_apply_filter_on_empty_record() {
        let filter = FilterCondition::new("city=Berlin").unwrap();
        let record = MockRecord { fields: HashMap::new() };
        assert!(!filter.apply(&record));
    }
}