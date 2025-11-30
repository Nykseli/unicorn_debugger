use crate::program::Program;
use std::rc::Rc;
use unicorn_engine::{Arch, Mode, Prot, RegisterX86, Unicorn};

pub struct Engine<'a> {
    engine: Unicorn<'a, Rc<Program>>,
}

impl<'a> Engine<'a> {
    pub fn new(program: Program) -> Self {
        let program = Rc::new(program);
        let mut engine = Unicorn::new_with_data(Arch::X86, Mode::MODE_64, program).unwrap();
        engine.mem_map(0, 8 * 1024 * 1024, Prot::ALL).unwrap();
        let program = engine.get_data().clone();

        for section in program.sections() {
            if let Some(data) = program.section_data(section) {
                engine.mem_write(section.sh_addr, data).unwrap();
            }
        }

        engine
            .add_insn_sys_hook(
                unicorn_engine::X86Insn::SYSCALL,
                program.start(),
                0,
                |emu| {
                    let syscall = emu.reg_read(RegisterX86::RAX).unwrap();

                    if syscall == 1 {
                        let fd = emu.reg_read(RegisterX86::RDI).unwrap();
                        let data_ptr = emu.reg_read(RegisterX86::RSI).unwrap();
                        let data_len = emu.reg_read(RegisterX86::RDX).unwrap();
                        let data_from_mem =
                            emu.mem_read_as_vec(data_ptr, data_len as usize).unwrap();

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
                },
            )
            .unwrap();

        // 0xcc is the INT3 debuggin code
        // https://en.wikipedia.org/wiki/INT_(x86_instruction)#INT3
        engine.mem_write(program.start(), &[0xcc]).unwrap();
        engine
            .add_intr_hook(|emu, _| {
                println!("Debug interruption found!");
                let int_ip = emu.pc_read().unwrap() - 1;
                let orig_byte = emu.mem_read_as_vec(int_ip, 1).unwrap();
                // make sure it's the CC byte
                if orig_byte[0] == 0xCC {
                    // 0xb8 is the original byte in the hello.out binary
                    emu.mem_write(emu.get_data().start(), &[0xb8]).unwrap();
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

        Self { engine }
    }

    pub fn start(&mut self) {
        self.engine
            .emu_start(self.engine.get_data().start(), 8192, 0, 0)
            .unwrap()
    }
}
