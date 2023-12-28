use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use crate::Screen;

pub const PAGE: usize = 0x100;
pub const BANK: usize = PAGE * 256;
pub const MEMORY: usize = BANK * 256;
pub const FULL_MEMORY: usize = MEMORY + 8;
pub const SAMPLE_RATE: usize = PAGE * 60;

pub const INPUT: usize = 0;
pub const PC: usize = 2;
pub const VIDEO: usize = 5;
pub const AUDIO: usize = 6;

#[derive(Debug)]
pub struct Cpu {
    memory: Memory,
    tick: Duration,
    screen: Arc<RwLock<Screen>>,
}

impl Cpu {
    pub fn new(memory: Memory, screen: Arc<RwLock<Screen>>) -> Self {
        let tick = Duration::new(1, 0) / 60;
        Self {
            memory,
            tick,
            screen,
        }
    }

    pub fn tick(&mut self) {
        let instant = Instant::now();
        let mut pc = self.memory.get_value_at(PC);

        for _ in 0..65536 {
            let src = self.memory.get_value_at(pc);
            let dst = self.memory.get_value_at(pc + 3);
            self.memory.data[dst] = self.memory.data[src];
            pc = self.memory.get_value_at(pc + 6);
        }
        self.screen
            .write()
            .unwrap()
            .update(self.memory.get_video_data());
        let elapsed = instant.elapsed();
        let sleep = self.tick - elapsed;
        thread::sleep(sleep);
    }
}

#[derive(Debug)]
pub struct Memory {
    data: Box<[u8]>,
}

impl Memory {
    pub fn new() -> Self {
        let data: Vec<u8> = vec![0; FULL_MEMORY];
        let data = data.into_boxed_slice();
        Self { data }
    }

    fn get_value_at(&self, index: usize) -> usize {
        (self.data[index] as usize) << 16
            | (self.data[index + 1] as usize) << 8
            | self.data[index + 2] as usize
    }

    fn get_video_data(&self) -> &[u8] {
        let offset = (self.data[VIDEO] as usize) << 16;
        &self.data[offset..offset + BANK]
    }

    pub fn load_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let mut file = File::open(path)?;
        let _ = file.read(&mut self.data[..MEMORY])?;
        Ok(())
    }
}
