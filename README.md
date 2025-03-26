# clrblk

clrblk is a command-line utility for printing color blocks in the terminal. It supports various color input formats, including ANSI codes, hexadecimal values, and named ANSI colors. Users can also specify a range of ANSI colors, adjust block width, and enable inline range printing.

## Features

- Print color blocks using:
  - ANSI color codes
  - Hexadecimal colors
  - Named ANSI colors
  - Ranges of ANSI colors
- Customize block width
- Inline range printing for compact output
- Lightweight and fast, written in Rust

## Installation


### Manual Installation

```sh
git clone https://gitlab.com/ashe.sterling/clrblk.git
cd clrblk
cargo build --release --locked
```
then copy to somewhere in your $PATH
```sh
cp /path/to/path/in/$PATH
```

## Usage

```sh
clrblk [OPTIONS] <color(1) [color(2)]>
```

### Examples

#### Print a single ANSI color block
```sh
clrblk 34
```

#### Print a hex color block
```sh
clrblk #ff5733
```

#### Print a named ANSI color block
```sh
clrblk bright_magenta
```

#### Print a range of ANSI colors
```sh
clrblk 16 231
```

#### Set block width
```sh
clrblk -w 10
```

#### Print an inline range of ANSI colors
```sh
clrblk -i 16 231
```
