use std::fs::read;

pub struct Program {
    // TODO: mapp the section header data directly here so it maps 1-1 with the program memory addresses
    data: Vec<u8>,
    /// Where does execution start
    start: u64,
}

impl Program {
    pub fn new(path: &str, start: u64) -> Self {
        let data = read(path).unwrap();
        Self { data, start }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
