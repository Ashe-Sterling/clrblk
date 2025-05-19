use clap::Parser;
use std::str;

use crate::{
    ansi::{named_color_to_ansi, print_block_ansi, print_blocks_ansi}, 
    hex::{print_block_hex, print_hex_gradient}, 
    validate::is_valid_hex_color
};

#[derive(Parser, Debug)]
#[command(version, about = "A simple utility to show and test pretty (and not so pretty) colors in the terminal.")]

pub struct Args {
    /// Width of blocks
    #[arg(short, long, default_value_t = 6)]
    pub width: u8,

    /// Multiple colors shown in one line (only for ANSI ranges)
    #[arg(short, long)]
    pub inline: bool,

    /// Print color number(s) before each block (only for ANSI)
    #[arg(short, long)]
    pub numbered: bool,

    /// Fit hex gradient to full terminal width
    #[arg(short, long)]
    pub fit: bool,

    /// Color(s) to display: ANSI codes, names, or hex strings (#RRGGBB)
    #[arg(num_args = 1..=2)]
    pub values: Vec<String>,

    /// Print a full 6-phase RGB rainbow
    #[arg(short, long)]
    pub rainbow: bool,

    /// Print a grayscale gradient
    #[arg(short, long)]
    pub grayscale: bool,

    /// Show a fullscreen rainbow gradient loop (+/- to adjust brightness)
    #[arg(short, long)]
    pub loop_gradient: bool,

    /// testing
    #[arg(long)]
    pub testing: bool
}

pub fn single(values: &[String], width: u8, numbered: bool) {
    let input = &values[0];
    if let Some(code) = named_color_to_ansi(input) {
        print_block_ansi(code, width, numbered);
    } else if let Ok(code) = input.parse::<u8>() {
        print_block_ansi(code, width, numbered);
    } else if is_valid_hex_color(input) {
        let hex = input.strip_prefix('#').unwrap_or(input);
        let pairs: Vec<&str> = hex.as_bytes().chunks(2)
            .map(|c| str::from_utf8(c).unwrap())
            .collect();
        print_block_hex(pairs, width);
    } else {
        eprintln!("⚠️  Input color `{}` not recognized (see --help)", input);
    }
}


pub fn many(values: &[String], width: u8, inline: bool, numbered: bool, fit_width: bool) {
    let a = &values[0];
    let b = &values[1];
    if let (Ok(c1), Ok(c2)) = (a.parse::<u8>(), b.parse::<u8>()) {
        print_blocks_ansi(c1, c2, width, inline, numbered);
    } else if is_valid_hex_color(a) && is_valid_hex_color(b) {
        let h1 = a.strip_prefix('#').unwrap_or(a);
        let h2 = b.strip_prefix('#').unwrap_or(b);
        let p1: Vec<&str> = h1.as_bytes().chunks(2)
            .map(|c| str::from_utf8(c).unwrap())
            .collect();
        let p2: Vec<&str> = h2.as_bytes().chunks(2)
            .map(|c| str::from_utf8(c).unwrap())
            .collect();
        print_hex_gradient(p1, p2, fit_width);
    } else {
        eprintln!("⚠️  Invalid color/range: `{}` and `{}`", a, b);
    }
}