use std::io::{self, Write};
use std::str;
extern crate clap;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "A simple utility to show and test pretty (and not so pretty) colors.")]

struct Args {
    /// Width of blocks
    #[arg(short, long, default_value_t = 6)]
    width: u8,

    /// Multiple colors shown in one line
    #[arg(short, long)]
    inline: bool,

    /// Print color number(s) before printing the color block(s)
    #[arg(short, long)]
    numbered: bool,

    /// Supports ANSI color codes, a range of ANSI color codes, ANSI_colors_in_words, and hexcodes with or without #
    #[arg(num_args = 1..=2)]
    values: Vec<String>,

    /// Prints a pretty rainbow!
    #[arg(short, long)]
    rainbow: bool,

    // Prints a grayscale gradient
    #[arg(short, long)]
    grayscale: bool,
}

fn print_block_ansi(color: u8, width: u8, numbered: bool) {
    let mut buffer = String::new();
    let space_block = " ".repeat(width.into());
    if numbered {
        buffer.push_str(&color.to_string());
    }
    buffer.push_str(&format!("\x1b[48;5;{}m", color));
    buffer.push_str(&space_block);
    buffer.push_str("\x1b[0m\n");

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(buffer.as_bytes());
    let _ = handle.flush();
}

fn print_blocks_ansi(color1: u8, color2: u8, width: u8, inline: bool, numbered: bool) {
    let mut buffer = String::new();
    let space_block = " ".repeat(width.into());
    if inline {
        if color1 >= color2 {
            for color in (color2..=color1).rev() {
                if numbered {
                    buffer.push_str("\x1b[0m ");
                    buffer.push_str(&color.to_string());
                    buffer.push_str(":");
                }
                buffer.push_str(&format!("\x1b[48;5;{}m", color));
                buffer.push_str(&space_block);    
            }
        } else {
            for color in color1..=color2 {
                if numbered {
                    buffer.push_str("\x1b[0m ");
                    buffer.push_str(&color.to_string());
                    buffer.push_str(":");
                }
                buffer.push_str(&format!("\x1b[48;5;{}m", color));
                buffer.push_str(&space_block);    
            }
        }
        buffer.push_str("\x1b[0m\n");
    } else {
        if color1 >= color2 {
            for color in (color2..=color1).rev() {
                if numbered {
                    buffer.push_str("\x1b[0m ");
                    buffer.push_str(&color.to_string());
                    buffer.push_str(":");
                }
                buffer.push_str(&format!("\x1b[48;5;{}m", color));
                buffer.push_str(&space_block);    
                buffer.push_str("\x1b[0m\n");
            }
        } else {
            for color in color1..=color2 {
                if numbered {
                    buffer.push_str("\x1b[0m ");
                    buffer.push_str(&color.to_string());
                    buffer.push_str(":");
                }
                buffer.push_str(&format!("\x1b[48;5;{}m", color));
                buffer.push_str(&space_block);    
                buffer.push_str("\x1b[0m\n");
            }
        }
    }
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(buffer.as_bytes());
    let _ = handle.flush();
}

fn print_block_hex(hex_pairs: Vec<&str>, width: u8) {
    if hex_pairs.len() != 3 {
        eprintln!("⚠️  Hex input should be 6 characters split into 3 parts, e.g. ['ff', 'bb', 'ee' = ffbbee or #ffbbee]");
        return;
    }

    let r = u8::from_str_radix(hex_pairs[0], 16).unwrap_or(0);
    let g = u8::from_str_radix(hex_pairs[1], 16).unwrap_or(0);
    let b = u8::from_str_radix(hex_pairs[2], 16).unwrap_or(0);

    let mut buffer = String::new();
    buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
    buffer.push_str(&" ".repeat(width.into()));
    buffer.push_str("\x1b[0m\n");

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(buffer.as_bytes());
    let _ = handle.flush();
}

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
        "orange" =>  {
            eprintln!("⚠️  Orange is not an official ANSI color; printing approximation (208).");
            Some(208)
        },
        "purple" =>  {
            eprintln!("⚠️  Purple is not an official ANSI color; printing approximation (129).");
            Some(129)
        },
        _ => None
    }
}

fn print_grayscale() {
    let mut buffer = String::new();

    for v in 0..=255 {
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", v, v, v));
        buffer.push_str(" ");
    }
    buffer.push_str("\n");
    for v in (0..=255).rev() {
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", v, v, v));
        buffer.push_str(" ");
    }
    buffer.push_str("\x1b[0m\n");

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(buffer.as_bytes());
    let _ = handle.flush();
}

fn print_rainbow() {
    let mut buffer = String::new();
    let mut r: u8 = 255;
    let mut g: u8 = 0;
    let mut b: u8 = 0;
    for i in 0..=255 {
        g = i;
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
        buffer.push_str(" ");
    }
    for i in (0..=255).rev(){
        r = i;
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
        buffer.push_str(" ");
    }
    for i in 0..=255{
        b = i;
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
        buffer.push_str(" ");
    }
    for i in (0..=255).rev(){
        g = i;
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
        buffer.push_str(" ");
    }
    for i in 0..=255 {
        r = i;
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
        buffer.push_str(" ");
    }
    for i in (0..=255).rev(){
        b = i;
        buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
        buffer.push_str(" ");
    }
    buffer.push_str("\x1b[0m\n");
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(buffer.as_bytes());
    let _ = handle.flush();
}

fn single(values: &[String], width: u8, numbered: bool) {
    let color_input: &str = &values[0];
    // it works but this logic is a mess and I hate it
    if let Some(ansi_code) = named_color_to_ansi(color_input) {
        print_block_ansi(ansi_code, width, numbered);
    } else if let Ok(ansi_code) = color_input.parse::<u8>() {
        print_block_ansi(ansi_code, width, numbered);
    } else if let Ok(ansi_code) = color_input.parse::<u16>() {
        eprintln!("⚠️  Input color or format ({}) not recognized (see -h or --help for more information.)", ansi_code);
    } else if color_input.len() == 6 {
        let hexcode = color_input;
        let hex_pairs: Vec<&str> = hexcode
            .as_bytes()
            .chunks(2)
            .map(|chunk| str::from_utf8(chunk).unwrap())
            .collect();
        print_block_hex(hex_pairs, width);
    } else if color_input.len() == 7 {
        let hexcode = color_input.strip_prefix('#').unwrap_or(color_input);
        let hex_pairs: Vec<&str> = hexcode
            .as_bytes()
            .chunks(2)
            .map(|chunk| str::from_utf8(chunk).unwrap())
            .collect();
        print_block_hex(hex_pairs, width);
    } else {
        eprintln!("⚠️  Input color format or name ({}) not recognized (see -h or --help for more information).", color_input);
    }
}

fn many(values: &[String], width: u8, inline: bool, numbered: bool) {
    if let Ok(color_input1) = values[0].parse::<u8>() {
        if let Ok(color_input2) = values[1].parse::<u8>() {
            print_blocks_ansi(color_input1, color_input2, width, inline, numbered);
        } else {
            eprintln!("⚠️  Input range end is not a valid ANSI color code (0-255 needed, {} provided).", values[1]);
        }
    } else if let Ok(_color_input2) = values[1].parse::<u8>() {
        eprintln!("⚠️  Input range start is not a valid ANSI color code (0-255 needed, {} provided).", values[0]);
    } else {
        eprintln!("⚠️  Input ranges provided are not valid ANSI color codes (0-255 needed, {} and {} provided.)", values[0], values[1])
    }
}

fn main() {
    let args = Args::parse();
    if args.rainbow {
        print_rainbow();
    } else if args.grayscale {
        print_grayscale();
    } else if args.values.len() == 2 { 
        many(&args.values, args.width, args.inline, args.numbered);
    } else if args.values.len() == 1 {
        single(&args.values, args.width, args.numbered);
    } else if args.values.len() == 0 {
        eprintln!("⚠️  No arguments provided (see -h or --help for details)");
    } else if args.values.len() >= 3 {
        eprintln!("⚠️  More than 2 positional arguments provided; could not parse color or color range. [!THIS SHOULD NEVER APPEAR IF args HAS PARSED CORRECTLY!]");
    } else {
        eprintln!("⚠️  !!!ARGUMENTS DID NOT PARSE CORRECTLY!!! (If you see this please consider creating an issue on the gitlab repo [https://gitlab.com/ashe.sterling/clrblk]) ")
    }
}
