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
/// Random gradient looping per-cell, now with SIMD™ (Optimized)

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
    let mut stdout = BufWriter::new(stdout().into_raw_mode()?);
    write!(stdout, "\x1b[?25l")?;
    stdout.flush()?;

    let mut stdin = async_stdin().bytes();
    let mut buffer = Buffer::new();

    loop {
        if let Some(Ok(input)) = stdin.next() {
            if input == 3 { // 3 = ctrl-c
                break;
            }
        }

        buffer.resize();
        buffer.tick();
        buffer.render(&mut stdout)?;
        stdout.flush()?;
        thread::sleep(Duration::from_millis(20));
    }

    write!(stdout, "\x1b[0m\x1b[?25h")?;
    stdout.flush()?;
    Ok(())
}

struct Buffer {
    width: u16,
    height: u16,
    pixels: PixelBuffer,
    goals: PixelBuffer,
    rng: ThreadRng,
}

#[repr(align(64))]  // align to cache line boundary
struct PixelBuffer {
    r: Vec<u8>,
    g: Vec<u8>,
    b: Vec<u8>,
}

impl PixelBuffer {
    fn new(size: usize) -> Self {
        Self {
            r: vec![0; size],
            g: vec![0; size],
            b: vec![0; size],
        }
    }

    fn resize(&mut self, new_size: usize) {
        self.r.resize(new_size, 0);
        self.g.resize(new_size, 0);
        self.b.resize(new_size, 0);
    }

    fn fill_random(&mut self, rng: &mut ThreadRng) {
        rng.fill(&mut self.r[..]);
        rng.fill(&mut self.g[..]);
        rng.fill(&mut self.b[..]);
    }
}

impl Buffer {
    fn new() -> Self {
        let (w, h) = terminal_size().unwrap();
        let size = (w as usize) * (h as usize);
        let mut rng = rand::rng();

        let mut pixels = PixelBuffer::new(size);
        let mut goals = PixelBuffer::new(size);

        pixels.fill_random(&mut rng);
        goals.fill_random(&mut rng);

        Buffer { width: w, height: h, pixels, goals, rng }
    }

    fn resize(&mut self) {
        let (w, h) = terminal_size().unwrap();
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            let size = (w as usize) * (h as usize);

            self.pixels.resize(size);
            self.goals.resize(size);

            self.pixels.fill_random(&mut self.rng);
            self.goals.fill_random(&mut self.rng);
        }
    }

    fn tick(&mut self) {
        let len = self.pixels.r.len();
        let chunks = len / LANES;

        // pre-generate ALL random data in one massive batch (maximum RNG efficiency)
        let total_random_needed = chunks * LANES * 3;
        let mut rng_buffer = vec![0u8; total_random_needed];
        self.rng.fill(&mut rng_buffer[..]);

        // process chunks with manual loop unrolling for maximum SIMD throughput
        let mut chunk_idx = 0;
        let unroll_factor = 4;
        let unrolled_chunks = chunks / unroll_factor;
        
        // process 4 chunks at a time to maximize instruction pipeline utilization
        for _ in 0..unrolled_chunks {
            let base1 = chunk_idx * LANES;
            let base2 = (chunk_idx + 1) * LANES;
            let base3 = (chunk_idx + 2) * LANES;
            let base4 = (chunk_idx + 3) * LANES;
            
            self.process_chunk(base1, &rng_buffer, chunk_idx);
            self.process_chunk(base2, &rng_buffer, chunk_idx + 1);
            self.process_chunk(base3, &rng_buffer, chunk_idx + 2);
            self.process_chunk(base4, &rng_buffer, chunk_idx + 3);
            
            chunk_idx += unroll_factor;
        }

        for i in chunk_idx..chunks {
            let base = i * LANES;
            self.process_chunk(base, &rng_buffer, i);
        }

        // process remainder
        let remaining = len % LANES;
        if remaining != 0 {
            self.process_remaining_elements(chunks * LANES, remaining);
        }
    }

    #[inline(always)]
    fn process_chunk(&mut self, base: usize, rng_buffer: &[u8], chunk_idx: usize) {
        // load up the SIMD registers
        let r_vec = Simd::<u8, LANES>::from_slice(&self.pixels.r[base..base + LANES]);
        let g_vec = Simd::<u8, LANES>::from_slice(&self.pixels.g[base..base + LANES]);
        let b_vec = Simd::<u8, LANES>::from_slice(&self.pixels.b[base..base + LANES]);
        let gr_vec = Simd::<u8, LANES>::from_slice(&self.goals.r[base..base + LANES]);
        let gg_vec = Simd::<u8, LANES>::from_slice(&self.goals.g[base..base + LANES]);
        let gb_vec = Simd::<u8, LANES>::from_slice(&self.goals.b[base..base + LANES]);

        let one = Simd::splat(1u8);

        // check less than or greater than
        let r_lt = r_vec.simd_lt(gr_vec);
        let r_gt = r_vec.simd_gt(gr_vec);
        let g_lt = g_vec.simd_lt(gg_vec);
        let g_gt = g_vec.simd_gt(gg_vec);
        let b_lt = b_vec.simd_lt(gb_vec);
        let b_gt = b_vec.simd_gt(gb_vec);

        // step towards goal
        let r_new = r_lt.select(r_vec + one, r_gt.select(r_vec - one, r_vec));
        let g_new = g_lt.select(g_vec + one, g_gt.select(g_vec - one, g_vec));
        let b_new = b_lt.select(b_vec + one, b_gt.select(b_vec - one, b_vec));

        // write the new colors
        r_new.copy_to_slice(&mut self.pixels.r[base..base + LANES]);
        g_new.copy_to_slice(&mut self.pixels.g[base..base + LANES]);
        b_new.copy_to_slice(&mut self.pixels.b[base..base + LANES]);

        // check for reached goal
        let done = r_new.simd_eq(gr_vec) & g_new.simd_eq(gg_vec) & b_new.simd_eq(gb_vec);

        // load pre-generated random goals
        let rng_base = chunk_idx * LANES * 3;
        let new_gr = Simd::from_slice(&rng_buffer[rng_base..rng_base + LANES]);
        let new_gg = Simd::from_slice(&rng_buffer[rng_base + LANES..rng_base + 2 * LANES]);
        let new_gb = Simd::from_slice(&rng_buffer[rng_base + 2 * LANES..rng_base + 3 * LANES]);

        // store new goals (masked to completed cells)
        new_gr.store_select(&mut self.goals.r[base..base + LANES], done);
        new_gg.store_select(&mut self.goals.g[base..base + LANES], done);
        new_gb.store_select(&mut self.goals.b[base..base + LANES], done);
    }

    fn process_remaining_elements(&mut self, start: usize, remaining: usize) {
        // scalar fallback
        for i in start..start + remaining {
            // step towards goal
            self.pixels.r[i] = self.step_towards_goal(self.pixels.r[i], self.goals.r[i]);
            self.pixels.g[i] = self.step_towards_goal(self.pixels.g[i], self.goals.g[i]);
            self.pixels.b[i] = self.step_towards_goal(self.pixels.b[i], self.goals.b[i]);

            // check if goal is reached and assign new goal if so
            if self.pixels.r[i] == self.goals.r[i] && 
               self.pixels.g[i] == self.goals.g[i] && 
               self.pixels.b[i] == self.goals.b[i] {
                self.goals.r[i] = self.rng.random();
                self.goals.g[i] = self.rng.random();
                self.goals.b[i] = self.rng.random();
            }
        }
    }

    #[inline(always)]
    fn step_towards_goal(&self, current: u8, goal: u8) -> u8 {
        match current.cmp(&goal) {
            std::cmp::Ordering::Less => current + 1,
            std::cmp::Ordering::Greater => current - 1,
            std::cmp::Ordering::Equal => current,
        }
    }

    fn render(&self, out: &mut impl Write) -> io::Result<()> {
        write!(out, "{}{}", cursor::Goto(1, 1), clear::All)?;
        
        // pre-allocate string buffer for the entire frame
        let mut frame_buffer = String::with_capacity(
            (self.width as usize) * (self.height as usize) * 20 // rough estimate for escape sequences
        );

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row as usize) * (self.width as usize) + (col as usize);
                frame_buffer.push_str(&format!(
                    "\x1b[48;2;{};{};{}m ", 
                    self.pixels.r[idx], 
                    self.pixels.g[idx], 
                    self.pixels.b[idx]
                ));
            }
            if row < self.height - 1 {
                frame_buffer.push_str("\r\n");
            }
        }
        frame_buffer.push_str("\x1b[0m");

        write!(out, "{}", frame_buffer)?;
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

        phase = phase.saturating_add(1);
        thread::sleep(Duration::from_millis(20));
    }

    write!(stdout, "\x1b[0m\x1b[?25h").unwrap();
    stdout.flush().unwrap();
}
