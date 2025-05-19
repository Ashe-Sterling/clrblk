mod ansi;
mod cli;
mod hex;
mod rainbow;
mod validate;

use clap::Parser;
use cli::{many, single, Args};
use rainbow::{Buffer, fullscreen_rainbow, print_grayscale, print_rainbow};
use termion::raw::IntoRawMode;
use std::io::stdout;
use std::{self, thread};
use std::time::Duration;


fn testingfn() -> std::io::Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    let mut buffer = Buffer::new();

    loop {
        buffer.resize();
        buffer.tick();
        buffer.render(&mut stdout);
        thread::sleep(Duration::from_millis(15));
    }
}

fn main() {
    let args = Args::parse();
    if args.rainbow {
        print_rainbow();
    } else if args.grayscale {
        print_grayscale();
    } else if args.loop_gradient {
        fullscreen_rainbow();
    } else if args.testing {
        let _ = testingfn();
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
