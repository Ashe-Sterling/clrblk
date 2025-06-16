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

use std::simd::{cmp::SimdPartialOrd, Mask, prelude::{Simd, SimdPartialEq}};

//////////////////////////////////////////////////////////////////////////////////////////
/// Random gradient looping per-cell, now with SIMD #fuckitweball edition™

// number of SIMD lanes for u8's
#[cfg(target_feature = "avx512f")]
const LANES: usize = 64;
#[cfg(all(not(target_feature = "avx512f"), target_feature = "avx2"))]
const LANES: usize = 32;
#[cfg(all(not(target_feature = "avx512f"), not(target_feature = "avx2"), target_feature = "sse2"))]
const LANES: usize = 16;
#[cfg(not(any(target_feature = "avx512f", target_feature = "avx2", target_feature = "sse2")))]
const LANES: usize = 1; // fallback to scalar


/// macro for stepping each lane of `$cur` ±1 toward `$goal`, using `$one` = Simd::splat(1u8)
macro_rules! step_toward {
    ($cur:expr, $goal:expr, $one:expr) => {{
        let lt = $cur.simd_lt($goal);
        let gt = $cur.simd_gt($goal);
        lt.select($cur + $one, gt.select($cur - $one, $cur))
    }};
}


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
        // this is the only thing I actually learned from Quake 3's fast 1/sqrt
        let one = Simd::splat(1u8);
    
        // process full LANES-sized chunks
        let mut r_chunks  = self.r.chunks_exact_mut(LANES);
        let mut g_chunks  = self.g.chunks_exact_mut(LANES);
        let mut b_chunks  = self.b.chunks_exact_mut(LANES);
        let mut gr_chunks = self.gr.chunks_exact_mut(LANES);
        let mut gg_chunks = self.gg.chunks_exact_mut(LANES);
        let mut gb_chunks = self.gb.chunks_exact_mut(LANES);
    
        while let (Some(r_slice), Some(g_slice), Some(b_slice),
                   Some(gr_slice), Some(gg_slice), Some(gb_slice)) = (
            r_chunks.next(), g_chunks.next(), b_chunks.next(),
            gr_chunks.next(), gg_chunks.next(), gb_chunks.next()
        ) {
            // load into SIMD registers
            let r_vec  = Simd::from_slice(r_slice);
            let g_vec  = Simd::from_slice(g_slice);
            let b_vec  = Simd::from_slice(b_slice);
            let gr_vec = Simd::from_slice(gr_slice);
            let gg_vec = Simd::from_slice(gg_slice);
            let gb_vec = Simd::from_slice(gb_slice);
    
            // step ±1 toward goal
            let r_new  = step_toward!(r_vec,  gr_vec, one);
            let g_new  = step_toward!(g_vec,  gg_vec, one);
            let b_new  = step_toward!(b_vec,  gb_vec, one);
            
    
            // write current values
            r_new.copy_to_slice(r_slice);
            g_new.copy_to_slice(g_slice);
            b_new.copy_to_slice(b_slice);
    
            // mask 'done' lanes
            let done = r_new.simd_eq(gr_vec) & g_new.simd_eq(gg_vec) & b_new.simd_eq(gb_vec);
    
            // fill new random goals
            let mut buf_gr = [0u8; LANES];
            let mut buf_gg = [0u8; LANES];
            let mut buf_gb = [0u8; LANES];
            self.rng.fill(&mut buf_gr);
            self.rng.fill(&mut buf_gg);
            self.rng.fill(&mut buf_gb);
    
            let new_gr = Simd::from_array(buf_gr);
            let new_gg = Simd::from_array(buf_gg);
            let new_gb = Simd::from_array(buf_gb);
    
            // store new goals maskingly
            new_gr.store_select(gr_slice, done);
            new_gg.store_select(gg_slice, done);
            new_gb.store_select(gb_slice, done);
        }
    
        // handle remaining
        let r_tail  = r_chunks.into_remainder();
        let g_tail  = g_chunks.into_remainder();
        let b_tail  = b_chunks.into_remainder();
        let gr_tail = gr_chunks.into_remainder();
        let gg_tail = gg_chunks.into_remainder();
        let gb_tail = gb_chunks.into_remainder();
    
        if !r_tail.is_empty() {
            // get an indexable array from the mask
            let mut mask_arr = [false; LANES];
            for i in 0..r_tail.len() {
                mask_arr[i] = true;
            }
            let tail_mask: Mask<i8, LANES> = Mask::from_array(mask_arr);
        
            // pad current & goal for r/g/b to fill SIMD lanes
            let mut cur_r = [0u8; LANES]; cur_r[..r_tail.len()].copy_from_slice(r_tail);
            let mut cur_g = [0u8; LANES]; cur_g[..g_tail.len()].copy_from_slice(g_tail);
            let mut cur_b = [0u8; LANES]; cur_b[..b_tail.len()].copy_from_slice(b_tail);
        
            let mut goal_r = [0u8; LANES]; goal_r[..gr_tail.len()].copy_from_slice(gr_tail);
            let mut goal_g = [0u8; LANES]; goal_g[..gg_tail.len()].copy_from_slice(gg_tail);
            let mut goal_b = [0u8; LANES]; goal_b[..gb_tail.len()].copy_from_slice(gb_tail);
        
            // load into SIMD registers
            let r_vec  = Simd::from_array(cur_r);
            let g_vec  = Simd::from_array(cur_g);
            let b_vec  = Simd::from_array(cur_b);
        
            let gr_vec = Simd::from_array(goal_r);
            let gg_vec = Simd::from_array(goal_g);
            let gb_vec = Simd::from_array(goal_b);
        
            // step each channel
            let r_next = step_toward!(r_vec, gr_vec, one);
            let g_next = step_toward!(g_vec, gg_vec, one);
            let b_next = step_toward!(b_vec, gb_vec, one);
        
            // write back only valid lanes maskingly
            r_next.store_select(r_tail, tail_mask);
            g_next.store_select(g_tail, tail_mask);
            b_next.store_select(b_tail, tail_mask);
        
            // refill new goals where all three reached
            let done_mask = (r_next.simd_eq(gr_vec)
                           & g_next.simd_eq(gg_vec)
                           & b_next.simd_eq(gb_vec))
                           & tail_mask;
        
            // set fresh random goals
            let mut buf_r = [0u8; LANES];
            let mut buf_g = [0u8; LANES];
            let mut buf_b = [0u8; LANES];
            self.rng.fill(&mut buf_r);
            self.rng.fill(&mut buf_g);
            self.rng.fill(&mut buf_b);
        
            let new_gr = Simd::from_array(buf_r);
            let new_gg = Simd::from_array(buf_g);
            let new_gb = Simd::from_array(buf_b);
            
            // store them maskingly
            new_gr.store_select(gr_tail, done_mask);
            new_gg.store_select(gg_tail, done_mask);
            new_gb.store_select(gb_tail, done_mask);
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

        phase = phase.saturating_add(1);
        thread::sleep(Duration::from_millis(20));
    }

    write!(stdout, "\x1b[0m\x1b[?25h").unwrap();
    stdout.flush().unwrap();
}
