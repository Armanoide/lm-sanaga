pub fn find_json_object_end_bytes(buffer: &[u8]) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (i, &byte) in buffer.iter().enumerate() {
        match byte {
            b'"' if !escaped => in_string = !in_string,
            b'\\' if in_string => escaped = !escaped,
            _ if escaped => escaped = false,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1); // index after the closing '}'
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
        let input = br#"{"key":"value"} extra"#;
        let end = find_json_object_end_bytes(input).unwrap();
        assert_eq!(&input[..end], br#"{"key":"value"}"#);
    }

    #[test]
    fn test_nested_object() {
        let input = br#"{"a":{"b":"c"}} trailing"#;
        let end = find_json_object_end_bytes(input).unwrap();
        assert_eq!(&input[..end], br#"{"a":{"b":"c"}}"#);
    }

    #[test]
    fn test_incomplete_object() {
        let input = br#"{"key":"value""#;
        assert_eq!(find_json_object_end_bytes(input), None);
    }

    #[test]
    fn test_escaped_quotes() {
        let input = br#"{"quote": "some \"nested\" quote"} and more"#;
        let end = find_json_object_end_bytes(input).unwrap();
        assert_eq!(&input[..end], br#"{"quote": "some \"nested\" quote"}"#);
    }

    #[test]
    fn test_empty_object() {
        let input = br#"{} junk"#;
        let end = find_json_object_end_bytes(input).unwrap();
        assert_eq!(&input[..end], br#"{}"#);
    }
}
