use std::{
    io::{self, stdout, BufWriter, Read, Write}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread, time::Duration
};

use rand::{rngs::ThreadRng, Rng};
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

// number of SIMD lanes for u8's
#[cfg(target_feature = "avx512f")]
const LANES: usize = 64;
#[cfg(all(not(target_feature = "avx512f"), target_feature = "avx2"))]
const LANES: usize = 32;
#[cfg(all(not(target_feature = "avx512f"), not(target_feature = "avx2"), target_feature = "sse2"))]
const LANES: usize = 16;
#[cfg(not(any(target_feature = "avx512f", target_feature = "avx2", target_feature = "sse2")))]
const LANES: usize = 1; // fallback to scalar

pub fn crazyfn() -> io::Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    write!(stdout, "\x1b[?25l")?;
    stdout.flush()?;

    let mut stdin = async_stdin().bytes();
    let mut buffer = Buffer::new();

    loop {
        if let Some(Ok(b)) = stdin.next() {
            if b == 3 {
                break;
            }
        }

        buffer.resize();
        buffer.tick();
        buffer.render(&mut stdout)?;
        stdout.flush()?;  // ensure every frame is fully drawn
        thread::sleep(Duration::from_millis(20));
    }

    write!(stdout, "\x1b[0m\x1b[?25h")?;
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
    rng: ThreadRng,
}

impl Buffer {
    fn new() -> Self {
        let (w, h) = terminal_size().unwrap();
        let size = (w as usize) * (h as usize);
        let mut rng = rand::rng();

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

        Buffer { width: w, height: h, r, g, b, gr, gg, gb, rng }
    }

    fn resize(&mut self) {
        let (w, h) = terminal_size().unwrap();
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            let size = (w as usize) * (h as usize);

            self.r.resize(size, 0);
            self.g.resize(size, 0);
            self.b.resize(size, 0);
            self.gr.resize(size, 0);
            self.gg.resize(size, 0);
            self.gb.resize(size, 0);

            self.rng.fill(&mut self.r[..]);
            self.rng.fill(&mut self.g[..]);
            self.rng.fill(&mut self.b[..]);
            self.rng.fill(&mut self.gr[..]);
            self.rng.fill(&mut self.gg[..]);
            self.rng.fill(&mut self.gb[..]);
        }
    }

    fn tick(&mut self) {
        // get the chunk size
        let len = self.r.len();
        let chunks = len / LANES;

        // looping through each chunk
        for i in 0..chunks {
            let base = i * LANES;

            // grab the current and goal colors as vectors
            let r_vec  = Simd::<u8, LANES>::from_slice(&self.r[base..][..LANES]);
            let g_vec  = Simd::<u8, LANES>::from_slice(&self.g[base..][..LANES]);
            let b_vec  = Simd::<u8, LANES>::from_slice(&self.b[base..][..LANES]);
            let gr_vec = Simd::<u8, LANES>::from_slice(&self.gr[base..][..LANES]);
            let gg_vec = Simd::<u8, LANES>::from_slice(&self.gg[base..][..LANES]);
            let gb_vec = Simd::<u8, LANES>::from_slice(&self.gb[base..][..LANES]);

            let one = Simd::splat(1);

            // use simd < and > to determine whether we are adding or subtracting 1 for this tick
            let r_new = r_vec.simd_lt(gr_vec)
                .select(r_vec + one, r_vec.simd_gt(gr_vec).select(r_vec - one, r_vec));
            let g_new = g_vec.simd_lt(gg_vec)
                .select(g_vec + one, g_vec.simd_gt(gg_vec).select(g_vec - one, g_vec));
            let b_new = b_vec.simd_lt(gb_vec)
                .select(b_vec + one, b_vec.simd_gt(gb_vec).select(b_vec - one, b_vec));

            // write this tick's color values to the output buffer
            r_new.copy_to_slice(&mut self.r[base..][..LANES]);
            g_new.copy_to_slice(&mut self.g[base..][..LANES]);
            b_new.copy_to_slice(&mut self.b[base..][..LANES]);

            // use simd == to determine whether we have reached the goal color 
            let done = r_new.simd_eq(gr_vec) & g_new.simd_eq(gg_vec) & b_new.simd_eq(gb_vec);

            // create a buffer of new random values for this chunk
            let mut gr_buf = [0u8; LANES];
            let mut gg_buf = [0u8; LANES];
            let mut gb_buf = [0u8; LANES];
            self.rng.fill(&mut gr_buf[..]);
            self.rng.fill(&mut gg_buf[..]);
            self.rng.fill(&mut gb_buf[..]);

            // make it simd
            let new_gr = Simd::from_array(gr_buf);
            let new_gg = Simd::from_array(gg_buf);
            let new_gb = Simd::from_array(gb_buf);

            // apply the new random simd colors buffer to the goal colors, masked to the cells which have reached the goal color this tick
            new_gr.store_select(&mut self.gr[base..][..LANES], done);
            new_gg.store_select(&mut self.gg[base..][..LANES], done);
            new_gb.store_select(&mut self.gb[base..][..LANES], done);
        }

            // handle the remaining unaligned chunks
            let remaining = len % LANES;
            if remaining != 0 {
                let start = chunks * LANES;

                // load valid lanes into full-width arrays
                let mut r_buf  = [0u8; LANES];
                let mut g_buf  = [0u8; LANES];
                let mut b_buf  = [0u8; LANES];
                let mut gr_buf = [0u8; LANES];
                let mut gg_buf = [0u8; LANES];
                let mut gb_buf = [0u8; LANES];
                r_buf[..remaining].copy_from_slice(&self.r[start..]);
                g_buf[..remaining].copy_from_slice(&self.g[start..]);
                b_buf[..remaining].copy_from_slice(&self.b[start..]);
                gr_buf[..remaining].copy_from_slice(&self.gr[start..]);
                gg_buf[..remaining].copy_from_slice(&self.gg[start..]);
                gb_buf[..remaining].copy_from_slice(&self.gb[start..]);

                // grab current and goal colors as vectors
                let r_vec  = Simd::from_array(r_buf);
                let g_vec  = Simd::from_array(g_buf);
                let b_vec  = Simd::from_array(b_buf);
                let gr_vec = Simd::from_array(gr_buf);
                let gg_vec = Simd::from_array(gg_buf);
                let gb_vec = Simd::from_array(gb_buf);

                let one   = Simd::splat(1u8);

                // compute ±1 step
                let r_new = r_vec.simd_lt(gr_vec)
                    .select(r_vec + one, r_vec.simd_gt(gr_vec).select(r_vec - one, r_vec));
                let g_new = g_vec.simd_lt(gg_vec)
                    .select(g_vec + one, g_vec.simd_gt(gg_vec).select(g_vec - one, g_vec));
                let b_new = b_vec.simd_lt(gb_vec)
                    .select(b_vec + one, b_vec.simd_gt(gb_vec).select(b_vec - one, b_vec));
                let done  = r_new.simd_eq(gr_vec) & g_new.simd_eq(gg_vec) & b_new.simd_eq(gb_vec);

                // write this tick's values
                let mut tmp_r = [0u8; LANES];
                let mut tmp_g = [0u8; LANES];
                let mut tmp_b = [0u8; LANES];
                r_new.copy_to_slice(&mut tmp_r);
                g_new.copy_to_slice(&mut tmp_g);
                b_new.copy_to_slice(&mut tmp_b);

                self.r[start..start+remaining].copy_from_slice(&tmp_r[..remaining]);
                self.g[start..start+remaining].copy_from_slice(&tmp_g[..remaining]);
                self.b[start..start+remaining].copy_from_slice(&tmp_b[..remaining]);

                // get the mask as an array we can index into and apply goal colors maskingly
                let done_arr: [bool; LANES] = done.to_array();
                for i in 0..remaining {
                    if done_arr[i] {
                        self.gr[start + i] = self.rng.random();
                        self.gg[start + i] = self.rng.random();
                        self.gb[start + i] = self.rng.random();
                    }
                }
            }
    }

    fn render(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "{}{}", cursor::Goto(1, 1), clear::All)?;
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row as usize) * (self.width as usize) + (col as usize);
                write!(out, "\x1b[48;2;{};{};{}m ", self.r[idx], self.g[idx], self.b[idx])?;
            }
            if row < self.height - 1 {
                write!(out, "\r\n")?;
            }
        }
        write!(out, "\x1b[0m")?;
        Ok(())
    }
}

/// End of random gradient looping per-cell
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
