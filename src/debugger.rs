use std::{
    fs,
    io::{self, BufRead, Write},
    process::exit,
};

use crate::engine::Engine;

enum Command {
    Quit,
    Print,
    Run,
    Next,
    Continue,
    Logon,
    Logoff,
    Break(String),
}

enum ParseVal {
    Comment,
    Command(Command),
}

struct Ast {
    commands: Vec<Command>,
}

impl Ast {
    fn new(file: &str) -> Self {
        let mut commands = Vec::new();

        let mut idx = 0;
        let lines: Vec<&str> = file.lines().collect();
        while let Some((value, next_idx)) = Self::parse_command(idx, &lines) {
            if let ParseVal::Command(command) = value {
                commands.push(command);
            }
            idx = next_idx;
        }

        Self { commands }
    }

    fn parse_command(idx: usize, lines: &[&str]) -> Option<(ParseVal, usize)> {
        if idx >= lines.len() {
            return None;
        }

        let line = lines[idx];
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return Some((ParseVal::Comment, idx + 1));
        }

        let (command, size) = if line == "q" || line == "quit" || line == "exit" {
            (Command::Quit, 1)
        } else if line == "p" || line == "print" {
            (Command::Print, 1)
        } else if line == "r" || line == "run" {
            (Command::Run, 1)
        } else if line == "n" || line == "next" {
            (Command::Next, 1)
        } else if line == "c" || line == "continue" {
            (Command::Continue, 1)
        } else if line == "logon" {
            (Command::Logon, 1)
        } else if line == "logoff" {
            (Command::Logoff, 1)
        } else if line.starts_with("b ") || line.starts_with("break ") {
            (Command::Break(line.into()), 1)
        } else {
            panic!("Unknown command {line} on line {}", idx + 1);
        };

        Some((ParseVal::Command(command), idx + size))
    }
}

pub struct Debugger<'a> {
    pub engine: Engine<'a>,
}

impl<'a> Debugger<'a> {
    pub fn new(engine: Engine<'a>) -> Self {
        Self { engine }
    }

    fn run(&mut self) {
        if self.engine.exited() {
            exit(0);
        }
        self.engine.start();
    }

    fn cont(&mut self) {
        if self.engine.exited() {
            exit(0);
        }

        self.engine.cont();
    }

    fn next(&mut self) {
        if self.engine.exited() {
            exit(0);
        }

        self.engine.step();
    }

    fn add_break(&mut self, cmd: &str) {
        let addr = cmd.split_whitespace().nth(1).unwrap();
        let addr = if let Some(addrs) = addr.split_once(':') {
            let segment = u64::from_str_radix(addrs.0, 16).unwrap();
            let offset = u64::from_str_radix(addrs.1, 16).unwrap();
            segment * 16 + offset
        } else {
            u64::from_str_radix(addr, 16).unwrap()
        };

        self.engine.add_break(addr);
    }

    fn run_ast(&mut self, ast: &Ast) {
        for command in &ast.commands {
            match command {
                Command::Quit => exit(0),
                Command::Print => println!("{}", self.engine.read_cpu()),
                Command::Run => self.run(),
                Command::Next => self.next(),
                Command::Continue => self.cont(),
                Command::Logon => self.engine.set_verbose(true),
                Command::Logoff => self.engine.set_verbose(false),
                Command::Break(cmd) => self.add_break(cmd),
            }
        }
    }

    pub fn run_file(&mut self, path: &str) {
        let file_data = fs::read_to_string(path).unwrap();
        let ast = Ast::new(&file_data);
        self.run_ast(&ast);
    }

    pub fn repl(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut cmd = String::new();
            let _ = io::stdin().lock().read_line(&mut cmd).unwrap();
            let ast = Ast::new(&cmd);
            self.run_ast(&ast);
        }
    }
}
