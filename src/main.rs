#![feature(portable_simd)]

mod ansi;
mod cli;
mod hex;
mod rainbow;
mod validate;
mod terminal;
mod rng;

use cli::{Args, many, single, parse_args, print_help};
use rainbow::{print_grayscale, print_rainbow, crazyfn};



fn main() {
    let args: Args = parse_args();
    if args.error {
        return;
    }
    if args.help {
        print_help();
        return;
    }
    if args.version {
        let version = env!("CARGO_PKG_VERSION");
        println!("clrblk version {}", version);
        return;
    }
    if args.rainbow {
        print_rainbow();
    } else if args.grayscale {
        print_grayscale();
    } else if args.crazy {
        let _ = crazyfn();
    } else if args.values.len() == 2 {
        many(&args.values, args.width, args.inline, args.numbered, args.fit);
    } else if args.values.len() == 1 {
        single(&args.values, args.width, args.numbered);
    } else if args.values.is_empty() {
        eprintln!("Error: No arguments provided");
        print_help();
    } else {
        eprintln!("Error: Too many arguments (max 2 colors)");
        print_help();
    }
}
