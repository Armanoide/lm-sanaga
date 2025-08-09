pub fn extract_layer_index(name: &str) -> Option<usize> {
    name.split('.')
        .enumerate()
        .find(|&(_, part)| part == "layers")
        .and_then(|(i, _)| name.split('.').nth(i + 1))
        .and_then(|s| s.parse::<usize>().ok())
}
