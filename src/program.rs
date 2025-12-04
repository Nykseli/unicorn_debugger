use byteorder::{ByteOrder, LittleEndian};
use std::fs::read;

pub struct Program {
    // TODO: mapp the section header data directly here so it maps 1-1 with the program memory addresses
    data: Vec<u8>,
    /// Where does execution start
    start: u64,
    header: Header,
}

impl Program {
    pub fn new(path: &str, start: u64) -> Self {
        let mut data = read(path).unwrap();
        let header = Header::new(&mut data);
        data.drain(0..(header.header_size as usize * 16));

        Self {
            data,
            start,
            header,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

pub struct Header {
    last_page_bytes: u16,
    pages_in_file: u16,
    relocations: u16,
    header_size: u16,
    min_allocation: u16,
    max_allocation: u16,
    pub initial_ss: u16,
    pub initial_sp: u16,
    checksum: u16,
    pub initial_ip: u16,
    pub initial_cs: u16,
    relocation_table: u16,
    overlay: u16,
}

impl Header {
    pub fn new(bytes: &[u8]) -> Header {
        Header {
            last_page_bytes: LittleEndian::read_u16(&bytes[2..4]),
            pages_in_file: LittleEndian::read_u16(&bytes[4..6]),
            relocations: LittleEndian::read_u16(&bytes[6..8]),
            header_size: LittleEndian::read_u16(&bytes[8..10]),
            min_allocation: LittleEndian::read_u16(&bytes[10..12]),
            max_allocation: LittleEndian::read_u16(&bytes[12..14]),
            initial_ss: LittleEndian::read_u16(&bytes[14..16]),
            initial_sp: LittleEndian::read_u16(&bytes[16..18]),
            checksum: LittleEndian::read_u16(&bytes[18..20]),
            initial_ip: LittleEndian::read_u16(&bytes[20..22]),
            initial_cs: LittleEndian::read_u16(&bytes[22..24]),
            relocation_table: LittleEndian::read_u16(&bytes[24..26]),
            overlay: LittleEndian::read_u16(&bytes[26..28]),
        }
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{ByteOrder, LittleEndian};

    use crate::program::Header;

    #[test]
    fn parse_header() {
        let header: [u8; 0x1D] = [
            0x4D, 0x5A, 0x56, 0x00, 0x84, 0x00, 0x00, 0x00, 0x20, 0x00, 0xF9, 0x02, 0xFF, 0xFF,
            0x82, 0x10, 0x80, 0x00, 0x00, 0x00, 0x10, 0x00, 0x2B, 0x10, 0x1E, 0x00, 0x00, 0x00,
            0x01,
        ];

        let header = Header::new(&header);

        assert_eq!(
            header.last_page_bytes,
            LittleEndian::read_u16(&[0x56, 0x00])
        );
        assert_eq!(header.pages_in_file, LittleEndian::read_u16(&[0x84, 0x00]));
        assert_eq!(header.relocations, LittleEndian::read_u16(&[0x00, 0x00]));
        assert_eq!(header.header_size, LittleEndian::read_u16(&[0x20, 0x00]));
        assert_eq!(header.min_allocation, LittleEndian::read_u16(&[0xf9, 0x02]));
        assert_eq!(header.max_allocation, LittleEndian::read_u16(&[0xff, 0xff]));
        assert_eq!(header.initial_ss, LittleEndian::read_u16(&[0x82, 0x10]));
        assert_eq!(header.initial_sp, LittleEndian::read_u16(&[0x80, 0x00]));
        assert_eq!(header.checksum, LittleEndian::read_u16(&[0x00, 0x00]));
        assert_eq!(header.initial_ip, LittleEndian::read_u16(&[0x10, 0x00]));
        assert_eq!(header.initial_cs, LittleEndian::read_u16(&[0x2b, 0x10]));
        assert_eq!(
            header.relocation_table,
            LittleEndian::read_u16(&[0x1e, 0x00])
        );
        assert_eq!(header.overlay, LittleEndian::read_u16(&[0x00, 0x00]));
    }
}

#[repr(C)]
#[repr(packed)]
#[derive(Clone, Copy)]
pub struct PSP {
    // Usually set to INT 0x20 (0xcd20) prog terminate
    exit_interrupt: u16,
    /// "Segment of the first byte beyond the memory allocated to the program"
    /// Yeah that wording sucks. Basically the segment alloc ends
    /// So if we've alloc'd 0x0 -> 0x2000 it'd be 0x2001 /probably/
    alloc_end: u16,
    resv: u8,
    /// Far call instruction to MSDos function dispatcher
    call_disp: [u8; 5],
    /// .COM programs bytes available in segment (CP/M)
    com_bytes: u16,
    /// Terminate address used by INT 22, we need to jump to this addr on exit
    /// This forces a child program to return to it's parent program
    term_addr: u32,
    /// The Ctrl-Break exit address, a location of a subroutine for us to run
    /// when we encounter a Ctrl-Break
    ctrl_break_addr: u32,
    /// Similar to the above. If we critically error, run the routine here
    crit_err_addr: u32,
    /// Parent process's segment address
    parent_addr: u16,
    /// File handle array for the process. It's completely undocumented for 2.x+
    /// /probably/ not in use for our case
    file_handle_array: [u8; 20],
    /// Segment address of the environment, or zero
    env_segment_addr: u16,
    /// SS:SP of the last program that called INT 0x21,0
    last_exit_addr: u32,
    /// File handle array size
    file_handle_size: u16,
    /// File handle array pointer
    file_handle_addr: u32,
    /// Pointer to previous PSP
    prev_psp: u32,
    spacer: [u8; 20],
    // DOS function dispatcher CDh 21h CBh (Undoc. 3.x+) Ų
    dispatcher: [u8; 3],
    spacer_2: [u8; 9],
    unopened_fcb_1: [u8; 36],
    unopened_fcb_2: [u8; 20],
    cmd_trail_chars: u8,
    cmd_trail: [u8; 127],
}

impl PSP {
    pub fn new(alloc_end: u16, call_disp: [u8; 5]) -> Self {
        Self {
            exit_interrupt: 0xCD20,
            alloc_end,
            resv: 0x0,
            call_disp,
            com_bytes: 0x0,
            term_addr: 0xFFFF_FFFF,
            ctrl_break_addr: 0x9999_9999,
            crit_err_addr: 0x9999_9999,
            parent_addr: 0x0,
            file_handle_array: [0; 20],
            env_segment_addr: 0x0,
            last_exit_addr: 0x9999_9999,
            file_handle_size: 0x0,
            file_handle_addr: 0x9999_9999,
            prev_psp: 0x0,
            spacer: [0x0; 20],
            dispatcher: [0x0; 3],
            spacer_2: [0x0; 9],
            unopened_fcb_1: [0x0; 36],
            unopened_fcb_2: [0x0; 20],
            cmd_trail_chars: 0x0,
            cmd_trail: [0x0; 127],
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

impl<'a> From<&'a PSP> for &'a [u8] {
    fn from(value: &'a PSP) -> &'a [u8] {
        unsafe { any_as_u8_slice(value) }
    }
}
