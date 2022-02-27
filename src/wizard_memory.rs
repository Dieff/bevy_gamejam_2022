pub const MEMORY_SIZE: usize = 1_000_000;
pub const LOCAL_MEMORY: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryCellType {
    Blank,
    Field,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryCell {
    pub cell_type: MemoryCellType,
    pub value: u8,
}

impl MemoryCell {
    pub fn new(cell_type: MemoryCellType, value: u8) -> Self {
        if value > 9 {
            panic!("invalid value {} written to memory", value);
        }
        Self { cell_type, value }
    }
}

impl Default for MemoryCell {
    fn default() -> Self {
        MemoryCell::new(MemoryCellType::Blank, 0)
    }
}

pub struct MemoryBlob {
    // Even though this has a fixed size, I usee a vector instead of an array to allocate it on the heap
    memory: Vec<MemoryCell>,
}

impl MemoryBlob {
    pub fn new() -> Self {
        Self {
            memory: vec![MemoryCell::new(MemoryCellType::Blank, 0); MEMORY_SIZE],
        }
    }
    pub fn write_mem<'a, I: Iterator<Item = &'a MemoryCell>>(
        &mut self,
        values: I,
        start: usize,
    ) -> Result<(), ()> {
        if start > MEMORY_SIZE - 1 {
            Err(())?;
        }
        let mut index = start;
        for val in values {
            if index > MEMORY_SIZE {
                Err(())?;
            }
            self.memory[index] = *val;
            index += 1;
        }

        Ok(())
    }
    pub fn reset_player_memory(&mut self) {
        // clear player's working memory
        self.write_mem(
            std::iter::repeat(&MemoryCell::new(MemoryCellType::Blank, 0)).take(LOCAL_MEMORY - 1),
            0,
        )
        .unwrap();
    }
    pub fn get_many(&self, start: usize, count: usize) -> &[MemoryCell] {
      &self.memory[start..start + count]
    }
    pub fn get_one(&self, address: MemoryLocation) -> Option<&MemoryCell> {
      self.memory.get(address.pointer)
    }
}

#[derive(Default)]
pub struct MemoryLocation {
  pub pointer: usize,
}
