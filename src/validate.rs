pub fn is_valid_hex_color(s: &str) -> bool {
    let hex = s.strip_prefix('#').unwrap_or(s);
    hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit())
}