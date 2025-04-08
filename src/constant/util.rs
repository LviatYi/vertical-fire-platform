/// # get hidden sensitive string
///
/// convert string from "something" to "so*****ng"
///
/// ### Arguments
///
/// * `str`: original string
pub fn get_hidden_sensitive_string(str: &str) -> String {
    let len = str.len();
    let mut result = String::new();

    for (i, c) in str.chars().enumerate() {
        if len < 4 || (i > 1 && i < len - 2) {
            result.push('*');
        } else {
            result.push(c);
        }
    }

    result
}
