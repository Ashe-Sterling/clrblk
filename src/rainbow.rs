use std::{
    io::{self, stdout, BufWriter, Read, Write},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread,
    time::Duration,
};

use rand::Rng;
use termion::{
    async_stdin,
    clear,
    cursor,
    raw::IntoRawMode,
    terminal_size,
};

use std::simd::{cmp::SimdPartialOrd, prelude::{Simd, SimdPartialEq}};

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

        let mut r = vec![0; size];
        let mut g = vec![0; size];
        let mut b = vec![0; size];
        let mut gr = vec![0; size];
        let mut gg = vec![0; size];
        let mut gb = vec![0; size];

        rng.fill(&mut r[..]);
        rng.fill(&mut g[..]);
        rng.fill(&mut b[..]);
        rng.fill(&mut gr[..]);
        rng.fill(&mut gg[..]);
        rng.fill(&mut gb[..]);

        Buffer { width: w, height: h, r, g, b, gr, gg, gb }
    }

    fn resize(&mut self) {
        let (w, h) = terminal_size().unwrap();
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            let mut rng = rand::rng();
            let size = (w as usize) * (h as usize);

            self.r = vec![0; size];
            self.g = vec![0; size];
            self.b = vec![0; size];
            self.gr = vec![0; size];
            self.gg = vec![0; size];
            self.gb = vec![0; size];

            rng.fill(&mut self.r[..]);
            rng.fill(&mut self.g[..]);
            rng.fill(&mut self.b[..]);
            rng.fill(&mut self.gr[..]);
            rng.fill(&mut self.gg[..]);
            rng.fill(&mut self.gb[..]);
        }
    }

    fn tick(&mut self) {
        let mut rng = rand::rng();
        const LANES: usize = 64;
        let len = self.r.len();
        let chunks = len / LANES;

        for i in 0..chunks {
            let base = i * LANES;

            let r  = Simd::<u8, LANES>::from_slice(&self.r[base..base + LANES]);
            let g  = Simd::<u8, LANES>::from_slice(&self.g[base..base + LANES]);
            let b  = Simd::<u8, LANES>::from_slice(&self.b[base..base + LANES]);
            let gr = Simd::<u8, LANES>::from_slice(&self.gr[base..base + LANES]);
            let gg = Simd::<u8, LANES>::from_slice(&self.gg[base..base + LANES]);
            let gb = Simd::<u8, LANES>::from_slice(&self.gb[base..base + LANES]);

            let one = Simd::splat(1);

            let r_new = r.simd_lt(gr)
                .select(r + one, r.simd_gt(gr).select(r - one, r));
            let g_new = g.simd_lt(gg)
                .select(g + one, g.simd_gt(gg).select(g - one, g));
            let b_new = b.simd_lt(gb)
                .select(b + one, b.simd_gt(gb).select(b - one, b));

            r_new.copy_to_slice(&mut self.r[base..base + LANES]);
            g_new.copy_to_slice(&mut self.g[base..base + LANES]);
            b_new.copy_to_slice(&mut self.b[base..base + LANES]);

            let done = r_new.simd_eq(gr) & g_new.simd_eq(gg) & b_new.simd_eq(gb);

            let mut gr_buf = [0u8; LANES];
            let mut gg_buf = [0u8; LANES];
            let mut gb_buf = [0u8; LANES];
            rng.fill(&mut gr_buf);
            rng.fill(&mut gg_buf);
            rng.fill(&mut gb_buf);

            let new_gr = Simd::from_array(gr_buf);
            let new_gg = Simd::from_array(gg_buf);
            let new_gb = Simd::from_array(gb_buf);

            new_gr.store_select(&mut self.gr[base..base + LANES], done);
            new_gg.store_select(&mut self.gg[base..base + LANES], done);
            new_gb.store_select(&mut self.gb[base..base + LANES], done);
        }

        // scalar fallback for the len/64 remainder
        let remainder = len % LANES;
        let tail_start = chunks * LANES;
        for i in 0..remainder {
            let idx = tail_start + i;
            for (v, g) in [(&mut self.r, &mut self.gr), (&mut self.g, &mut self.gg), (&mut self.b, &mut self.gb)] {
                if v[idx] < g[idx] {
                    v[idx] += 1;
                } else if v[idx] > g[idx] {
                    v[idx] -= 1;
                } else {
                    g[idx] = rng.random();
                }
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