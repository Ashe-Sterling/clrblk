use std::{
    io::{self, BufWriter, Write},
    str,
};

pub fn named_color_to_ansi(input: &str) -> Option<u8> {
    match input.to_lowercase().as_str() {
        "black"                             => Some(0),
        "red"                               => Some(1),
        "green"                             => Some(2),
        "yellow"                            => Some(3),
        "blue"                              => Some(4),
        "magenta"                           => Some(5),
        "cyan"                              => Some(6),
        "white"                             => Some(7),
        "bright_black" | "gray" | "grey"    => Some(8),
        "bright_red"                        => Some(9),
        "bright_green"                      => Some(10),
        "bright_yellow"                     => Some(11),
        "bright_blue"                       => Some(12),
        "bright_magenta"                    => Some(13),
        "bright_cyan"                       => Some(14),
        "bright_white"                      => Some(15),
        "orange"                            =>  {
            eprintln!("⚠️  Orange is not an official ANSI color; printing approximation (208).");
            Some(208)
        },
        "purple"                            =>  {
            eprintln!("⚠️  Purple is not an official ANSI color; printing approximation (129).");
            Some(129)
        },
        _                                   => None
    }
}

pub fn print_block_ansi(color: u8, width: u8, numbered: bool) {
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


pub fn print_blocks_ansi(color1: u8, color2: u8, width: u8, inline: bool, numbered: bool) {
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
