use std::{str, thread, time::Duration, io::{self, Write, BufWriter}};
extern crate clap;
use clap::Parser;
use terminal_size::{Width, terminal_size};


#[derive(Parser, Debug)]
#[command(version, about = "A simple utility to show and test pretty (and not so pretty) colors.")]


struct Args {
    /// Width of blocks
    #[arg(short, long, default_value_t = 6)]
    width: u8,

    /// Multiple colors shown in one line (only for ANSI ranges)
    #[arg(short, long)]
    inline: bool,

    /// Print color number(s) before each block (only for ANSI)
    #[arg(short, long)]
    numbered: bool,

    /// Fit hex gradient to full terminal width
    #[arg(short, long)]
    fit: bool,

    /// Color(s) to display: ANSI codes, names, or hex strings (#RRGGBB)
    #[arg(num_args = 1..=2)]
    values: Vec<String>,

    /// Print a full 6-phase RGB rainbow
    #[arg(short, long)]
    rainbow: bool,

    /// Print a grayscale gradient
    #[arg(short, long)]
    grayscale: bool,

    /// Show a rainbow screensaver
    #[arg(short, long)]
    screensaver: bool,
}


fn print_block_ansi(color: u8, width: u8, numbered: bool) {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if numbered {
        let _ = write!(out, "{} ", color);
    }

    let _ = write!(out, "\x1b[48;5;{}m", color);
    for _ in 0..width {
        let _ = write!(out, " ");
    }

    let _ = writeln!(out, "\x1b[0m");
    let _ = out.flush();
}


fn print_blocks_ansi(color1: u8, color2: u8, width: u8, inline: bool, numbered: bool) {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let space_block = " ".repeat(width.into());

    let ascending = color1 <= color2;
    let range_iter: Box<dyn Iterator<Item = u8>> = if ascending {
        Box::new(color1..=color2)
    } else {
        Box::new((color2..=color1).rev())
    };

    if inline {
        for color in range_iter {
            if numbered {
                let _ = write!(out, "\x1b[0m{}:", color);
            }
            let _ = write!(out, "\x1b[48;5;{}m{}", color, space_block);
        }
        let _ = writeln!(out, "\x1b[0m");
    } else {
        for color in range_iter {
            if numbered {
                let _ = write!(out, "\x1b[0m{}:", color);
                if color <= 9 {
                    let _ = write!(out, "  ");
                } else if color <= 99 {
                    let _ = write!(out, " ");
                }
            }
            let _ = write!(out, "\x1b[48;5;{}m{}", color, space_block);
            let _ = writeln!(out, "\x1b[0m");
        }
    }

    let _ = out.flush();
}


fn print_hex_gradient(hex1: Vec<&str>, hex2: Vec<&str>, fit_width: bool) {
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
            Some((Width(w), _)) if w >= 1 => (w - 1) as usize,
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


fn print_block_hex(hex_pairs: Vec<&str>, width: u8) {
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


fn print_grayscale() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    for v in 0..=255 {
        let _ = write!(out, "\x1b[48;2;{0};{0};{0}m ", v);
    }
    let _ = writeln!(out);

    for v in (0..=255).rev() {
        let _ = write!(out, "\x1b[48;2;{0};{0};{0}m ", v);
    }
    let _ = writeln!(out, "\x1b[0m");

    let _ = out.flush();
}


fn print_rainbow() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let mut r: u8 = 255;
    let mut g: u8 = 0;
    let mut b: u8 = 0;

    for i in 0..=255 {
        g = i;
        let _ = write!(out, "\x1b[48;2;{};{};{}m ", r, g, b);
    }
    for i in (0..=255).rev() {
        r = i;
        let _ = write!(out, "\x1b[48;2;{};{};{}m ", r, g, b);
    }
    for i in 0..=255 {
        b = i;
        let _ = write!(out, "\x1b[48;2;{};{};{}m ", r, g, b);
    }
    for i in (0..=255).rev() {
        g = i;
        let _ = write!(out, "\x1b[48;2;{};{};{}m ", r, g, b);
    }
    for i in 0..=255 {
        r = i;
        let _ = write!(out, "\x1b[48;2;{};{};{}m ", r, g, b);
    }
    for i in (0..=255).rev() {
        b = i;
        let _ = write!(out, "\x1b[48;2;{};{};{}m ", r, g, b);
    }

    let _ = writeln!(out, "\x1b[0m");
    let _ = out.flush();
}


fn fullscreen_rainbow() {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    // Hide the cursor for a cleaner animation
    write!(out, "\x1b[?25l").ok();
    out.flush().ok();

    let mut phase: u8 = 0;
    loop {
        write!(out, "\x1b[H").ok();

        let hue = phase;
        let (r, g, b) = match hue {
            0..=85   => (255 - hue * 3, hue * 3, 0),
            86..=170 => (0, 255 - (hue - 85) * 3, (hue - 85) * 3),
            _        => ((hue - 170) * 3, 0, 255 - (hue - 170) * 3),
        };

        write!(out, "\x1b[48;2;{};{};{}m\x1b[2J", r, g, b).ok();

        out.flush().ok();
        phase = phase.wrapping_add(1);

        thread::sleep(Duration::from_millis(10));
    }
}


fn is_valid_hex_color(s: &str) -> bool {
    let hex = s.strip_prefix('#').unwrap_or(s);
    hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit())
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


fn single(values: &[String], width: u8, numbered: bool) {
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


fn many(values: &[String], width: u8, inline: bool, numbered: bool, fit_width: bool) {
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


fn main() {
    let args = Args::parse();
    if args.rainbow {
        print_rainbow();
    } else if args.grayscale {
        print_grayscale();
    } else if args.screensaver {
        fullscreen_rainbow();
    } else if args.values.len() == 2 {
        many(&args.values, args.width, args.inline, args.numbered, args.fit);
    } else if args.values.len() == 1 {
        single(&args.values, args.width, args.numbered);
    } else if args.values.is_empty() {
        eprintln!("⚠️  No arguments provided (see --help)");
    } else {
        eprintln!("⚠️  Too many arguments (max 2 colors)");
    }
}
