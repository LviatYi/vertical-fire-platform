pub(crate) enum SensitiveMode {
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
pub(crate) fn get_hidden_sensitive_string(str: &str, sensitive_mode: SensitiveMode) -> String {
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

/// # bring element to first
///
/// This function brings the element at the specified index to the first position in the vector.
///
/// ### Arguments
///
/// * `vec`: The mutable vector containing the elements.
/// * `idx`: The index of the element to bring to the first position.  
///   if `idx` is 0 or out of bounds, the vector remains unchanged.
pub(crate) fn bring_element_to_first<E>(vec: &mut [E], idx: usize) {
    if idx > 0 && idx < vec.len() {
        let sub_slice = &mut vec[0..=idx];
        sub_slice.rotate_right(1);
    }
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

    #[test]
    fn test_bring_element_to_first() {
        let mut vec = vec![1, 2, 3, 4, 5];
        super::bring_element_to_first(&mut vec, 2);
        assert_eq!(vec, vec![3, 1, 2, 4, 5]);

        let mut vec_last = vec![1, 2, 3];
        super::bring_element_to_first(&mut vec_last, 2);
        assert_eq!(vec_last, vec![3, 1, 2]);

        let mut vec_no_change = vec![1, 2, 3];
        super::bring_element_to_first(&mut vec_no_change, 0);
        assert_eq!(vec_no_change, vec![1, 2, 3]);

        let mut vec_out_of_range = vec![1, 2];
        super::bring_element_to_first(&mut vec_out_of_range, 5);
        assert_eq!(vec_out_of_range, vec![1, 2]);
    }
}
