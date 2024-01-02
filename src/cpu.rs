use std::{
    fs::File,
    io::Read,
    ops::{Index, IndexMut},
    path::Path,
    slice::SliceIndex,
    sync::{Arc, RwLock},
};

use crate::Screen;

pub const PAGE: usize = 0x100;
pub const BANK: usize = PAGE * 256;
pub const MEMORY: usize = BANK * 256;
pub const FULL_MEMORY: usize = MEMORY + 8;
// pub const SAMPLE_RATE: usize = PAGE * 60;

pub const INPUT: usize = 0;
pub const PC: usize = 2;
pub const VIDEO: usize = 5;
// pub const AUDIO: usize = 6;

#[derive(Debug)]
pub struct Cpu {
    memory: Memory,
    screen: Arc<RwLock<Screen>>,
}

impl Cpu {
    pub fn new(memory: Memory, screen: Arc<RwLock<Screen>>) -> Self {
        Self { memory, screen }
    }

    pub fn tick(&mut self) {
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
    }

    pub(crate) fn process_input(&mut self, value: &[u8]) {
        self.memory[INPUT] = value[0];
        self.memory[INPUT + 1] = value[1];
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
        (self[index] as usize) << 16 | (self[index + 1] as usize) << 8 | self[index + 2] as usize
    }

    fn get_video_data(&self) -> &[u8] {
        let offset = (self[VIDEO] as usize) << 16;
        &self[offset..offset + BANK]
    }

    pub fn load_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let mut file = File::open(path)?;
        let _ = file.read(&mut self.data[..MEMORY])?;
        Ok(())
    }
}

impl<T: SliceIndex<[u8]>> Index<T> for Memory {
    type Output = T::Output;

    fn index(&self, index: T) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: SliceIndex<[u8]>> IndexMut<T> for Memory {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        &mut self.data[index]
    }
}
