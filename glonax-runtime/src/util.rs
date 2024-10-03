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
/// use your_crate::OnOffExt;
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
