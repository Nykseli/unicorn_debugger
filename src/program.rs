use elf::{ElfBytes, endian::LittleEndian, section::SectionHeader};
use std::fs::read;

pub struct Program {
    // TODO: mapp the section header data directly here so it maps 1-1 with the program memory addresses
    data: Vec<u8>,
    /// Where does execution start
    start: u64,
    elf_sections: Vec<SectionHeader>,
}

impl Program {
    pub fn new(path: &str, start: u64) -> Self {
        let data = read(path).unwrap();
        let mut elf_sections = Vec::new();
        let file = ElfBytes::<LittleEndian>::minimal_parse(&data).unwrap();
        for section in file.section_headers().unwrap() {
            if section.sh_addr > 0 {
                elf_sections.push(section);
            }
        }

        Self {
            data,
            start,
            elf_sections,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn sections(&self) -> &[SectionHeader] {
        &self.elf_sections
    }

    pub fn section_data(&self, section: &SectionHeader) -> Option<&[u8]> {
        if section.sh_addr > 0 {
            let offset = section.sh_offset as usize;
            return Some(&self.data[offset..offset + section.sh_size as usize]);
        }

        None
    }
}
