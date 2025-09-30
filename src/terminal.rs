use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::mem;

use std::os::unix::io::RawFd;

const STDOUT_FILENO: RawFd = 1;
const TIOCGWINSZ: libc::c_ulong = 0x5413;

// apparently this can only be done with libc
#[repr(C)]
struct TermSize {
    row: libc::c_ushort,
    col: libc::c_ushort,
    x: libc::c_ushort,
    y: libc::c_ushort,
}

pub fn terminal_size() -> io::Result<(u16, u16)> {
    // shamelessly "borrowed" from termion
    unsafe {
        let mut size: TermSize = mem::zeroed();
        let result = libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size as *mut _);
        
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        
        Ok((size.col as u16, size.row as u16))
    }
}

pub fn enable_raw_mode() -> io::Result<()> {
    print!("\x1b[?25l"); // hide cursor
    io::stdout().flush()
}

pub fn disable_raw_mode() -> io::Result<()> {
    print!("\x1b[0m\x1b[?25h"); // reset attributes + show cursor
    io::stdout().flush()
}

pub fn clear_screen() -> io::Result<()> {
    print!("\x1b[H\x1b[2J"); // cursor home + clear screen
    io::stdout().flush()
}

// threaded input handler for non-blocking input
pub struct InputHandler {
    receiver: Receiver<u8>,
}

impl InputHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = [0u8; 1];
            
            loop {
                match stdin.read_exact(&mut buffer) {
                    Ok(_) => {
                        if sender.send(buffer[0]).is_err() {
                            break; // Receiver dropped, exit thread
                        }
                    }
                    Err(_) => break, // Error reading, exit thread
                }
            }
        });
        
        Self { receiver }
    }
    
    pub fn try_read(&self) -> Option<u8> {
        match self.receiver.try_recv() {
            Ok(byte) => Some(byte),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
    
    pub fn check_exit(&self) -> bool {
        while let Some(byte) = self.try_read() {
            match byte {
                3 => return true,        // Ctrl-C
                27 => return true,       // Escape
                b'q' | b'Q' => return true, // 'q' or 'Q'
                _ => {}
            }
        }
        false
    }
}