pub enum SensitiveMode {
    /// do not show any char
    Full,
    /// only the specified number of characters will be displayed at most.
    /// However, if the length is less than this number, none of them will be displayed.
    /// to ensure that there are definitely some characters that are not displayed.
    Normal(usize),
}

/// # get hidden sensitive string
///
/// convert string from "something" to "so*****ng"
///
/// ### Arguments
///
/// * `str`: original string
/// * `sensitive_mode`: mode to show sensitive string
pub fn get_hidden_sensitive_string(str: &str, sensitive_mode: SensitiveMode) -> String {
    let len = str.len();
    let mut result = String::new();
    let show_len = match sensitive_mode {
        SensitiveMode::Full => 0,
        SensitiveMode::Normal(shown_len) => shown_len,
    };

    let prefix_len = show_len / 2;
    let suffix_len = show_len - prefix_len;

    for (i, c) in str.chars().enumerate() {
        if len < show_len || (i >= prefix_len && i < len - suffix_len) {
            result.push('*');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_hidden_sensitive_string_full() {
        let str = "1234567890";
        let result = super::get_hidden_sensitive_string(str, super::SensitiveMode::Full);
        assert_eq!(result, "**********");
    }
    
    #[test]
    fn test_get_hidden_sensitive_string_normal() {
        let str = "1234567890";
        let result = super::get_hidden_sensitive_string(str, super::SensitiveMode::Normal(5));
        assert_eq!(result, "12*****890");
    }
}
