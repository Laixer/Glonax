/// A trait to extend functionality for types that can be represented as "on" or "off" strings.
///
/// This trait provides a method to convert a type into a static string slice representing its
/// "on" or "off" state.
pub trait OnOffExt {
    fn as_on_off_str(&self) -> &'static str;
}

/// Extension trait for the `bool` type to provide a method for converting
/// the boolean value to a string representation of "on" or "off".
///
/// # Examples
///
/// ```
/// use glonax::util::OnOffExt;;
///
/// assert_eq!(true.as_on_off_str(), "on");
/// assert_eq!(false.as_on_off_str(), "off");
/// ```
impl OnOffExt for bool {
    fn as_on_off_str(&self) -> &'static str {
        if *self {
            "on"
        } else {
            "off"
        }
    }
}

/// Converts a string into a boolean value.
///
/// # Arguments
///
/// * `value` - The string value to convert.
///
/// # Returns
///
/// Returns a `Result` containing the converted boolean value if the conversion is successful,
/// or an `Err` value if the conversion fails.
///
/// # Examples
///
/// ```
/// use glonax::util::string_try_into_bool;
///
/// let value = "on";
/// let result = string_try_into_bool(value);
/// assert_eq!(result, Ok(true));
/// ```
pub fn string_try_into_bool(value: &str) -> Result<bool, ()> {
    match value.to_lowercase().as_str() {
        "1" => Ok(true),
        "on" => Ok(true),
        "true" => Ok(true),
        "0" => Ok(false),
        "off" => Ok(false),
        "false" => Ok(false),
        _ => Err(()),
    }
}
