use std::{env,str};

use crate::{
    ansi::{named_color_to_ansi, print_block_ansi, print_blocks_ansi}, 
    hex::{print_block_hex, print_hex_gradient}, 
    validate::is_valid_hex_color
};


pub fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();
    let mut parsed_args = Args {
        width: 6,
        inline: false,
        numbered: false,
        fit: false,
        values: Vec::new(),
        rainbow: false,
        grayscale: false,
        loop_gradient: false,
        crazy: false,
        help: false,
        version: false,
        error: false
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-w" | "--width" => {
                if i + 1 < args.len() {
                    if let Ok(w) = args[i + 1].parse::<u8>() {
                        parsed_args.width = w;
                        i += 1;
                    } else {
                        eprintln!("Error: Invalid width value `{}`", args[i + 1]);
                        parsed_args.error = true;
                        return parsed_args;
                    }
                } else {
                    eprintln!("Error: Missing value for width");
                    parsed_args.error = true;
                    return parsed_args;
                }
            },
            "-i" | "--inline" => {
                parsed_args.inline = true;
            },
            "-n" | "--numbered" => {
                parsed_args.numbered = true;
            },
            "-f" | "--fit" => {
                parsed_args.fit = true;
            },
            "-r" | "--rainbow" => {
                parsed_args.rainbow = true;
            },
            "-g" | "--grayscale" => {
                parsed_args.grayscale = true;
            },
            "-l" | "--loop-gradient" => {
                parsed_args.loop_gradient = true;
            },
            "--crazy" => {
                parsed_args.crazy = true;
            },
            "-h" | "--help" => {
                parsed_args.help = true;
            },
            "-V" | "--version" => {
                parsed_args.version = true;
            },
            _ if !args[i].starts_with('-') => {
                parsed_args.values.push(args[i].clone());
            },
            _ => {
                eprintln!("Error: Unrecognized option `{}`", args[i]);
                parsed_args.error = true;
                return parsed_args;
            }
        }
        i += 1;
    }

    return parsed_args;
}

pub struct Args {
    /// Width of blocks
    pub width: u8,

    /// Multiple colors shown in one line (only for ANSI ranges)
    pub inline: bool,

    /// Print color number(s) before each block (only for ANSI)
    pub numbered: bool,

    /// Fit hex gradient to full terminal width
    pub fit: bool,

    /// Color(s) to display: ANSI codes, names, or hex strings (#RRGGBB)
    pub values: Vec<String>,

    /// Print a full 6-phase RGB rainbow
    pub rainbow: bool,

    /// Print a grayscale gradient
    pub grayscale: bool,

    /// Show a fullscreen rainbow gradient loop (+/- to adjust brightness)
    pub loop_gradient: bool,

    /// Show a fullscreen grid of cells of random colors that each fade to new random colors
    pub crazy: bool,

    /// Print help information
    pub help: bool,

    /// Print version information
    pub version: bool,

    pub error: bool
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
        eprintln!("Error: Input color `{}` not recognized (see --help)", input);
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
        eprintln!("Error: Invalid color/range: `{}` and `{}`", a, b);
    }
}


pub fn print_help() {
    println!("A simple utility to show and test pretty (and not so pretty) colors in the terminal.");
    println!();
    println!("\u{001b}[4mUsage:\u{001b}[24m clrblk [OPTIONS] [VALUES]...");
    println!();
    println!("\u{001b}[4mArguments:\u{001b}[24m");
    println!("  [VALUES]...  Color(s) to display: ANSI codes, names, or hex strings (#RRGGBB)");
    println!();
    println!("\u{001b}[4mOptions:\u{001b}[24m");
    println!("  -w, --width <WIDTH>  Width of blocks [default: 6 character spaces]");
    println!("  -i, --inline         Multiple colors shown in one line (only for ANSI ranges)");
    println!("  -n, --numbered       Print color number(s) before each block (only for ANSI)");
    println!("  -f, --fit           Fit hex gradient to full terminal width");
    println!("  -r, --rainbow        Print a full   6-phase RGB rainbow");
    println!("  -g, --grayscale      Print a grayscale gradient");
    println!("  -l, --loop-gradient  Show a fullscreen rainbow gradient loop (+/- to adjust brightness)");
    println!("      --crazy          Show a fullscreen grid of cells of random colors that each fade to new random colors");
    println!("  -h, --help           Print help information");
    println!("  -V, --version        Print version information");
    return;
}