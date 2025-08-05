pub fn typewriter(text: &str, delay_ms: u64) {
    let delay = std::time::Duration::from_millis(delay_ms);
    for c in text.chars() {
        print!("{c}");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(delay);
    }
}
