use std::{
<<<<<<< HEAD
    io::{self, stdout, BufWriter, Read, Write}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread, time::Duration
=======
        io::{self, stdout, BufWriter, Read, Write},
        sync::{
            atomic::{AtomicBool, Ordering}, Arc
        },
        thread,
        time::Duration,
>>>>>>> a848777fd501049345a6ee903198d26b6af59c0f
};

use rand::Rng;
use termion::{
    async_stdin,
    clear,
<<<<<<< HEAD
    cursor,
    raw::IntoRawMode,
    terminal_size,
=======
    cursor, 
    raw::IntoRawMode,
    terminal_size
>>>>>>> a848777fd501049345a6ee903198d26b6af59c0f
};

use std::simd::{cmp::SimdPartialOrd, prelude::{Simd, SimdPartialEq, SimdUint}};

//////////////////////////////////////////////////////////////////////////////////////////
/// Random gradient looping per-cell, now with SIMD™

pub fn crazyfn() -> std::io::Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    write!(stdout, "\x1b[?25l")?; // hide cursor
    stdout.flush()?;

    let mut stdin = async_stdin().bytes();
    let mut buffer = Buffer::new();

    loop {
        if let Some(Ok(b)) = stdin.next() {
            if b == 3 { // ctrl-c byte
                break;
            }
        }

        buffer.resize();
        buffer.tick();
        buffer.render(&mut stdout);
        stdout.flush()?;

        thread::sleep(Duration::from_millis(20));
    }

    write!(stdout, "\x1b[0m\x1b[?25h")?; // reset attrs + show cursor
    stdout.flush()?;
    Ok(())
}

// helper to increment or decrement towards goal
fn approach(current: u8, target: u8) -> u8 {
    if current < target {
        current.saturating_add(1)
    } else if current > target {
        current.saturating_sub(1)
    } else {
        current
    }
}


struct Buffer {
    width: u16,
    height: u16,
    r: Vec<u8>,
    g: Vec<u8>,
    b: Vec<u8>,
    gr: Vec<u8>,
    gg: Vec<u8>,
    gb: Vec<u8>,
}

impl Buffer {
    fn new() -> Self {
        let (w, h) = terminal_size().unwrap();
        let mut rng = rand::rng();
        let size = (w as usize) * (h as usize);

        let mut r = Vec::with_capacity(size);
        let mut g = Vec::with_capacity(size);
        let mut b = Vec::with_capacity(size);
        let mut gr = Vec::with_capacity(size);
        let mut gg = Vec::with_capacity(size);
        let mut gb = Vec::with_capacity(size);

        for _ in 0..size {
            r.push(rng.random());
            g.push(rng.random());
            b.push(rng.random());
            gr.push(rng.random());
            gg.push(rng.random());
            gb.push(rng.random());
        }

        Buffer { width: w, height: h, r, g, b, gr, gg, gb }
    }

    fn resize(&mut self) {
        let (w, h) = terminal_size().unwrap();
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            let mut rng = rand::rng();
            let size = (w as usize) * (h as usize);

            self.r.clear(); self.g.clear(); self.b.clear();
            self.gr.clear(); self.gg.clear(); self.gb.clear();

            for _ in 0..size {
                self.r.push(rng.random());
                self.g.push(rng.random());
                self.b.push(rng.random());
                self.gr.push(rng.random());
                self.gg.push(rng.random());
                self.gb.push(rng.random());
            }
        }
    }

    fn tick(&mut self) {
        let mut rng = rand::rng();
        const LANES: usize = 64;
        let len = self.r.len();
        let chunks = len / LANES;

        for i in 0..chunks {
            let base = i * LANES;

            let r = Simd::<u8, LANES>::from_slice(&self.r[base..base + LANES]);
            let g = Simd::<u8, LANES>::from_slice(&self.g[base..base + LANES]);
            let b = Simd::<u8, LANES>::from_slice(&self.b[base..base + LANES]);
            let gr = Simd::<u8, LANES>::from_slice(&self.gr[base..base + LANES]);
            let gg = Simd::<u8, LANES>::from_slice(&self.gg[base..base + LANES]);
            let gb = Simd::<u8, LANES>::from_slice(&self.gb[base..base + LANES]);

            let r_new = r.simd_lt(gr).select(r.saturating_add(Simd::splat(1)),
              r.simd_gt(gr).select(r.saturating_sub(Simd::splat(1)), r));
            let g_new = g.simd_lt(gg).select(g.saturating_add(Simd::splat(1)),
              g.simd_gt(gg).select(g.saturating_sub(Simd::splat(1)), g));
            let b_new = b.simd_lt(gb).select(b.saturating_add(Simd::splat(1)),
              b.simd_gt(gb).select(b.saturating_sub(Simd::splat(1)), b));

            // write back
            self.r[base..base + LANES].copy_from_slice(r_new.as_array());
            self.g[base..base + LANES].copy_from_slice(g_new.as_array());
            self.b[base..base + LANES].copy_from_slice(b_new.as_array());

            // compare
            let done = r_new.simd_eq(gr) & g_new.simd_eq(gg) & b_new.simd_eq(gb);
            let mask = done.to_bitmask();

            for j in 0..LANES {
                if (mask & (1 << j)) != 0 {
                    self.gr[base + j] = rng.random();
                    self.gg[base + j] = rng.random();
                    self.gb[base + j] = rng.random();
                }
            }
        }

        for i in (chunks * LANES)..len {
            self.r[i] = approach(self.r[i], self.gr[i]);
            self.g[i] = approach(self.g[i], self.gg[i]);
            self.b[i] = approach(self.b[i], self.gb[i]);

            if self.r[i] == self.gr[i] && self.g[i] == self.gg[i] && self.b[i] == self.gb[i] {
                self.gr[i] = rng.random();
                self.gg[i] = rng.random();
                self.gb[i] = rng.random();
            }
        }
    }

    fn render(&self, out: &mut impl Write) {
        write!(out, "{}{}", cursor::Goto(1, 1), clear::All).unwrap();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row as usize) * (self.width as usize) + (col as usize);
                write!(out, "\x1b[48;2;{};{};{}m ", self.r[idx], self.g[idx], self.b[idx]).unwrap();
            }
            if row < self.height - 1 {
                write!(out, "\r\n").unwrap();
            }
        }
        write!(out, "\x1b[0m").unwrap();
    }
}

/// End random gradient looping per-cell
//////////////////////////////////////////////////////////////////////////////////////////



//////////////////////////////////////////////////////////////////////////////////////////
/// Random gradient looping per-cell

pub fn crazyfn() -> std::io::Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    write!(stdout, "\x1b[?25l")?; // hide cursor
    stdout.flush()?;

    let mut stdin = async_stdin().bytes();
    let mut buffer = Buffer::new();

    loop {
        if let Some(Ok(b)) = stdin.next() {
            if b == 3 { // ctrl-c byte
                break;
            }
        }

        buffer.resize();
        buffer.tick();
        buffer.render(&mut stdout);
        stdout.flush()?;

        thread::sleep(Duration::from_millis(15));
    }

    write!(stdout, "\x1b[0m\x1b[?25h")?; // reset attrs + show cursor
    stdout.flush()?;
    Ok(())
}


// a single cell containing current and goal RGB values
struct Cell {
    r: u8,
    g: u8,
    b: u8,
    gr: u8,
    gg: u8,
    gb: u8,
}

impl Cell {
    // initialize a cell with random current and goal colors
    fn new<R: Rng>(rng: &mut R) -> Self {
        let r  = rng.random(); let g  = rng.random(); let b  = rng.random();
        let gr = rng.random(); let gg = rng.random(); let gb = rng.random();
        Cell { r, g, b, gr, gg, gb }
    }

    // move current color one step towards goal
    fn step<R: Rng>(&mut self, rng: &mut R) {
        self.r = approach(self.r, self.gr);
        self.g = approach(self.g, self.gg);
        self.b = approach(self.b, self.gb);
        
        // when goal is reached, pick a new random goal
        if self.r == self.gr && self.g == self.gg && self.b == self.gb {
            self.gr = rng.random();
            self.gg = rng.random();
            self.gb = rng.random();
        }
    }
}

// helper to increment or decrement towards goal
fn approach(current: u8, target: u8) -> u8 {
    if current < target {
        current.saturating_add(1)
    } else if current > target {
        current.saturating_sub(1)
    } else {
        current
    }
}


struct Buffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    // create buffer matching current terminal size, with random cell colors and goals
    fn new() -> Self {
        let (w, h) = terminal_size().unwrap();
        let mut rng = rand::rng();
        let size = (w as usize) * (h as usize);
        let cells = (0..size).map(|_| Cell::new(&mut rng)).collect();
        Buffer { width: w, height: h, cells }
    }

    // resize & reallocate new cells (call if terminal resized)
    fn resize(&mut self) {
        let (w, h) = terminal_size().unwrap();
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            let mut rng = rand::rng();
            let size = (w as usize) * (h as usize);
            self.cells = (0..size).map(|_| Cell::new(&mut rng)).collect();
        }
    }

    // advance each cell one step toward its goal
    fn tick(&mut self) {
        let mut rng = rand::rng();
        for cell in &mut self.cells {
            cell.step(&mut rng);
        }
    }

    fn render(&self, out: &mut impl Write) {
        write!(out, "{}{}", cursor::Goto(1, 1), clear::All).unwrap();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row as usize) * (self.width as usize) + (col as usize);
                let c = &self.cells[idx];
                write!(out, "\x1b[48;2;{};{};{}m ", c.r, c.g, c.b).unwrap();
            }
            write!(out, "\r\n").unwrap();
        }
        write!(out, "\x1b[0m").unwrap();
        out.flush().unwrap();
    }
}

/// End random gradient looping per-cell
//////////////////////////////////////////////////////////////////////////////////////////


pub fn print_grayscale() {
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


pub fn print_rainbow() {
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


pub fn fullscreen_rainbow() {
    let running = Arc::new(AtomicBool::new(true));
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    write!(stdout, "\x1b[?25l").unwrap();
    stdout.flush().unwrap();

    let mut phase: u8 = 0;
    let mut value: u8 = 255;
    let mut stdin = async_stdin().bytes();

    while running.load(Ordering::SeqCst) {
        if let Some(Ok(input)) = stdin.next() {
            match input {
                b'+' | b'=' if value < 255 => {
                    value = value.saturating_add(5);
                }
                b'-' if value > 0 => {
                    value = value.saturating_sub(5);
                }
                3 => { // Ctrl-C byte
                    running.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        }

        let hue = phase;
        let (r_val, g_val, b_val) = match hue {
            0..=85   => (255 - hue * 3, hue * 3, 0),
            86..=170 => (0, 255 - (hue - 85) * 3, (hue - 85) * 3),
            _        => ((hue - 170) * 3, 0, 255 - (hue - 170) * 3),
        };

        let r_scaled = (r_val as u16 * value as u16 / 255) as u8;
        let g_scaled = (g_val as u16 * value as u16 / 255) as u8;
        let b_scaled = (b_val as u16 * value as u16 / 255) as u8;

        write!(stdout, "\x1b[H\x1b[2J\x1b[48;2;{};{};{}m", r_scaled, g_scaled, b_scaled).unwrap();
        stdout.flush().unwrap();

        phase = phase.wrapping_add(1);
        thread::sleep(Duration::from_millis(20));
    }

    write!(stdout, "\x1b[0m\x1b[?25h").unwrap();
    stdout.flush().unwrap();
}