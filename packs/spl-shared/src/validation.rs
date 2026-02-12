use std::sync::OnceLock;
use regex::Regex;
use validator::ValidationError;

static RE_ALPHANUMERIC: OnceLock<Regex> = OnceLock::new();

pub fn validate_alphanumeric(name: &str) -> Result<(), ValidationError> {
    let re = RE_ALPHANUMERIC.get_or_init(|| Regex::new(r"^\w+$").unwrap());
    if re.is_match(name) {
        Ok(())
    } else {
        Err(ValidationError::new("alphanumeric"))
    }
}

pub fn validate_range_min_max<T: PartialOrd>(min: T, max: T) -> Result<(), ValidationError> {
    if min <= max {
        Ok(())
    } else {
        Err(ValidationError::new("range_min_max"))
    }
}
