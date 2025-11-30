use elf::{ElfBytes, endian::LittleEndian};
use unicorn_engine::{Mode, Prot, RegisterX86, Unicorn};

const ASM_FILE: &'static [u8] = include_bytes!("../asm/hello.out");

fn main() {
    let start = 0x0000000000401000;

    let mut engine = Unicorn::new(unicorn_engine::Arch::X86, Mode::MODE_64).unwrap();
    engine.mem_map(0, 8 * 1024 * 1024, Prot::ALL).unwrap();
    let file = ElfBytes::<LittleEndian>::minimal_parse(ASM_FILE).unwrap();

    for section in file.section_headers().unwrap() {
        if section.sh_addr > 0 {
            let offset = section.sh_offset as usize;
            engine
                .mem_write(
                    section.sh_addr,
                    &ASM_FILE[offset..offset + section.sh_size as usize],
                )
                .unwrap();
        }
    }

    engine
        .add_insn_sys_hook(unicorn_engine::X86Insn::SYSCALL, start, 0, |emu| {
            let syscall = emu.reg_read(RegisterX86::RAX).unwrap();

            if syscall == 1 {
                let fd = emu.reg_read(RegisterX86::RDI).unwrap();
                let data_ptr = emu.reg_read(RegisterX86::RSI).unwrap();
                let data_len = emu.reg_read(RegisterX86::RDX).unwrap();
                let data_from_mem = emu.mem_read_as_vec(data_ptr, data_len as usize).unwrap();

                if fd == 1 {
                    print!("{}", String::from_utf8(data_from_mem).unwrap())
                } else if fd == 2 {
                    eprint!("{}", String::from_utf8(data_from_mem).unwrap())
                } else {
                    println!("cannot write to fd '{fd}'");
                    emu.emu_stop().unwrap();
                }
            } else if syscall == 60 {
                emu.emu_stop().unwrap();
                println!("exit captured, stopping emulation");
            } else {
                println!("unknown syscall '{syscall}' captured, stopping emulation");
                emu.emu_stop().unwrap();
            }
        })
        .unwrap();

    // 0xcc is the INT3 debuggin code
    // https://en.wikipedia.org/wiki/INT_(x86_instruction)#INT3
    engine.mem_write(start, &[0xcc]).unwrap();
    engine
        .add_intr_hook(|emu, _| {
            println!("Debug interruption found!");
            let int_ip = emu.pc_read().unwrap() - 1;
            let orig_byte = emu.mem_read_as_vec(int_ip, 1).unwrap();
            // make sure it's the CC byte
            if orig_byte[0] == 0xCC {
                // 0xb8 is the original byte in the hello.out binary
                emu.mem_write(start, &[0xb8]).unwrap();
                emu.set_pc(int_ip).unwrap();
                // we need to invalidate the cache to make sure the code changes are applied
                // https://github.com/unicorn-engine/unicorn/wiki/FAQ#editing-an-instruction-doesnt-take-effecthooks-added-during-emulation-are-not-called
                emu.ctl_remove_cache(0, 8 * 1024 * 1024).unwrap();
            } else {
                println!(
                    "Unknown interruption instr 0x{:x} stopping emu",
                    orig_byte[0]
                );
                emu.emu_stop().unwrap();
            }
        })
        .unwrap();

    engine
        .emu_start(start, ASM_FILE.len() as u64, 0, 0)
        .unwrap();
}
