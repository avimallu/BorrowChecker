use crate::core::receipt::SplittingError;

pub fn split_by_comma(
    input_str: &str,
    error_message: &str,
) -> Result<(String, String), SplittingError> {
    input_str
        .split_once(",")
        .map(|(value, other)| (value.to_string(), other.to_string()))
        .ok_or_else(|| SplittingError::DecimalParsingError(error_message.to_string()))
}
