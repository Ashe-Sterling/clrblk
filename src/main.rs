mod cli;
mod ansi;
mod hex;
mod rainbow;
mod validate;

use clap::Parser;
use cli::{many, single, Args};
use rainbow::{fullscreen_rainbow, print_grayscale, print_rainbow};


fn main() {
    let args = Args::parse();
    if args.rainbow {
        print_rainbow();
    } else if args.grayscale {
        print_grayscale();
    } else if args.loop_gradient {
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
