use std::time::{SystemTime, UNIX_EPOCH};

// simple xorshift* RNG
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        Self { 
            state: if seed == 0 { 1 } else { seed } 
        }
    }
    
    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
    
    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }
    
    pub fn random<T>(&mut self) -> T 
    where 
        T: From<u8>
    {
        self.next_u8().into()
    }
    
    pub fn fill(&mut self, slice: &mut [u8]) {
        for byte in slice.iter_mut() {
            *byte = self.next_u8();
        }
    }
}

impl Default for SimpleRng {
    fn default() -> Self {
        Self::new()
    }
}