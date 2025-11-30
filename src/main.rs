use crate::{engine::Engine, program::Program};

mod engine;
mod program;

fn main() {
    let program = Program::new("asm/hello.out", 0x0000000000401000);
    let mut engine = Engine::new(program);
    engine.start();
}
