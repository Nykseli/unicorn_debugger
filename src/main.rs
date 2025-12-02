use std::env;

use crate::{debugger::Debugger, engine::Engine, program::Program};

mod debugger;
mod engine;
mod program;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = Program::new("TXLIST.EXE", 0x1000);
    let mut engine = Engine::new(program);
    if args.len() > 1 && args[1] == "-d" {
        let mut debug = Debugger::new(engine);
        if args.len() > 2 {
            debug.run_file(&args[2]);
        } else {
            debug.repl();
        }
    } else {
        engine.start();
    }
}
