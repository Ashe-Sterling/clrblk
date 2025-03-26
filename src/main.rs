use std::str;
use clap::Parser;


/// Example program demonstrating Clap's derive API
#[derive(Parser, Debug)]
#[command(version, about = "A simple utility to show and test pretty colors")]
struct Args {
    /// Width of something
    #[arg(short, long, default_value_t = 6)]
    width: u8,

    /// Multiple colors shown in one line
    #[arg(short, long)]
    inline: bool,

    /// Colors to print
    /// Supports ANSI decimal codes, 1-16 in words, and hexcodes with or without #
    #[arg(required = true, num_args = 1..=2)]
    values: Vec<String>,
}

fn print_block_ansi(color: u8, i: u8) {
    // \x1b is the actual escape character (decimal 27)
    print!("\x1b[48;5;{}m", color); 
    for _ in 0..i {
        print!(" ")
    }
    // Reset the color, then print a newline
    print!("\x1b[0m\n");
}

fn print_block_hex(hex_pairs: Vec<&str>, width: u8) {
    // Expect exactly 3 slices like ["ff", "bb", "ee"]
    if hex_pairs.len() != 3 {
        eprintln!("Hex input should be 6 characters, e.g. 'ffbbee'.");
        return;
    }

    // Convert each pair from hex to u8
    let r = u8::from_str_radix(hex_pairs[0], 16).unwrap_or(0);
    let g = u8::from_str_radix(hex_pairs[1], 16).unwrap_or(0);
    let b = u8::from_str_radix(hex_pairs[2], 16).unwrap_or(0);

    // Use the actual escape character \x1b and 24-bit color mode (48=bg,2=RGB)
    print!("\x1b[48;2;{};{};{}m", r, g, b);

    // Print a “block” of spaces
    for _ in 0..width {
        print!(" ");
    }

    // Reset color and print a newline
    print!("\x1b[0m\n");
}

fn print_blocks_ansi(color1: u8, color2: u8, i: u8, inline: bool) {
    if inline {
        for color in color1..=color2 {
            print!("\x1b[48;5;{}m", color); 
            for _ in 0..i {
                print!(" ")
            }
        }
        // Reset the color, then print a newline
        print!("\x1b[0m\n");
    } else {
        for color in color1..=color2 {
            print!("\x1b[48;5;{}m", color); 
            for _ in 0..i {
                print!(" ")
            }
            print!("\n");
        }
        // Reset the color, then print a newline
        print!("\x1b[0m\n");
    }
}
// Return Some(u8) if the input is a recognized color name
fn named_color_to_ansi(input: &str) -> Option<u8> {
    match input.to_lowercase().as_str() {
        "black" => Some(0),
        "red" => Some(1),
        "green" => Some(2),
        "yellow" => Some(3),
        "blue" => Some(4),
        "magenta" => Some(5),
        "cyan" => Some(6),
        "white" => Some(7),
        "bright_black" | "gray" | "grey" => Some(8),
        "bright_red" => Some(9),
        "bright_green" => Some(10),
        "bright_yellow" => Some(11),
        "bright_blue" => Some(12),
        "bright_magenta" => Some(13),
        "bright_cyan" => Some(14),
        "bright_white" => Some(15),
        _ => None
    }
}


fn single(values: &[String], width: u8) {
    let color_input: &str = &values[0];
    // 1) Check if it's a known named color
    if let Some(ansi_code) = named_color_to_ansi(color_input) {
        print_block_ansi(ansi_code, width);

    // 2) If not a named color, try parsing as a numeric ANSI code (0-255)
    } else if let Ok(ansi_code) = color_input.parse::<u8>() {
        print_block_ansi(ansi_code, width);

    // 3) Otherwise, treat it as a hex code
    } else {
        let hexcode = color_input.strip_prefix('#').unwrap_or(color_input);
        let hex_pairs: Vec<&str> = hexcode
            .as_bytes()
            .chunks(2)
            .map(|chunk| str::from_utf8(chunk).unwrap())
            .collect();
        print_block_hex(hex_pairs, width);
    }
}

/// Called when we have more than two positional arguments
fn many(values: &[String], width: u8, inline: bool) {
    let color_input1 = &values[0].parse::<u8>().unwrap();
    let color_input2 = &values[1].parse::<u8>().unwrap();
    print_blocks_ansi(*color_input1, *color_input2, width, inline);
}

fn main() {
    let args = Args::parse();

    // If we have more than two positional arguments, call a separate function
    if args.values.len() >= 2 {
        many(&args.values, args.width, args.inline);
    } else {
        single(&args.values, args.width);
    }
}