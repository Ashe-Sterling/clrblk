use termion::terminal_size;
use std::io::{self, BufWriter, Write};

pub fn print_hex_gradient(hex1: Vec<&str>, hex2: Vec<&str>, fit_width: bool) {
    let r1 = u8::from_str_radix(hex1[0], 16).unwrap_or(0);
    let g1 = u8::from_str_radix(hex1[1], 16).unwrap_or(0);
    let b1 = u8::from_str_radix(hex1[2], 16).unwrap_or(0);

    let r2 = u8::from_str_radix(hex2[0], 16).unwrap_or(0);
    let g2 = u8::from_str_radix(hex2[1], 16).unwrap_or(0);
    let b2 = u8::from_str_radix(hex2[2], 16).unwrap_or(0);

    let dr = (r2 as i16 - r1 as i16).abs() as usize;
    let dg = (g2 as i16 - g1 as i16).abs() as usize;
    let db = (b2 as i16 - b1 as i16).abs() as usize;

    let default_steps = dr.max(dg).max(db).max(1);

    let steps = if fit_width {
        match terminal_size() {
            Ok((w, _)) if w >= 1 => (w - 1) as usize,
            _ => default_steps,
        }
    } else {
        default_steps
    };

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let ri = (r1 as f32 + (r2 as f32 - r1 as f32) * t).round() as u8;
        let gi = (g1 as f32 + (g2 as f32 - g1 as f32) * t).round() as u8;
        let bi = (b1 as f32 + (b2 as f32 - b1 as f32) * t).round() as u8;

        let _ = write!(out, "\x1b[48;2;{};{};{}m ", ri, gi, bi);
    }

    let _ = writeln!(out, "\x1b[0m");
    let _ = out.flush();
}


pub fn print_block_hex(hex_pairs: Vec<&str>, width: u8) {
    let r = u8::from_str_radix(hex_pairs[0], 16).unwrap_or(0);
    let g = u8::from_str_radix(hex_pairs[1], 16).unwrap_or(0);
    let b = u8::from_str_radix(hex_pairs[2], 16).unwrap_or(0);

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let _ = write!(out, "\x1b[48;2;{};{};{}m", r, g, b);
    for _ in 0..width {
        let _ = write!(out, " ");
    }
    let _ = writeln!(out, "\x1b[0m");
    let _ = out.flush();
}