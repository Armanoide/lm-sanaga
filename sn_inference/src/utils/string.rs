pub fn find_json_object_end(str: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (i, c) in str.char_indices() {
        match c {
            '"' if !escaped => in_string = !in_string,
            '\\' if in_string => escaped = !escaped,
            _ if escaped => escaped = false,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1); // Return index **after** closing `}`
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let input = r#"{"key":"value"} extra"#;
        let end = find_json_object_end(input).unwrap();
        assert_eq!(&input[..end], r#"{"key":"value"}"#);
    }

    #[test]
    fn test_nested_object() {
        let input = r#"{"a":{"b":"c"}} trailing"#;
        let end = find_json_object_end(input).unwrap();
        assert_eq!(&input[..end], r#"{"a":{"b":"c"}}"#);
    }

    #[test]
    fn test_incomplete_object() {
        let input = r#"{"key":"value""#;
        assert_eq!(find_json_object_end(input), None);
    }

    #[test]
    fn test_escaped_quotes() {
        let input = r#"{"quote": "some \"nested\" quote"} and more"#;
        let end = find_json_object_end(input).unwrap();
        assert_eq!(&input[..end], r#"{"quote": "some \"nested\" quote"}"#);
    }

    #[test]
    fn test_empty_object() {
        let input = r#"{} junk"#;
        let end = find_json_object_end(input).unwrap();
        assert_eq!(&input[..end], r#"{}"#);
    }
}
