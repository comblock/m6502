use std::{io::Write, process::ExitStatus};

const OPCODES: &str = include_str!("opcodes.txt");

fn main() {
    // Code generation
    let output = std::env::var("OUT_DIR").unwrap();
    let mut opcodes = std::fs::File::create(format!("{output}/opcodes.rs")).unwrap();
    let mut parsing = std::fs::File::create(format!("{output}/parsing.rs")).unwrap();

    opcodes.write_all(b"#[derive(Debug, Clone, Copy)]pub enum Opcode{").unwrap();

    parsing.write_all(b"impl<B:Bus,C>Cpu<B,C>{\n///Fetches the next instruction and its operands.\npub(crate) fn fetch(&mut self)->Instruction{let opcode=self.load_pc();match opcode{").unwrap();

    let mut names = Vec::<&str>::new();

    for i in OPCODES.lines() {
        let line: Vec<&str> = i.split_whitespace().collect();

        let opcode = line[0];
        let name = line[1];
        let mode = line[2];
        let operands = match mode {
            "Implied" | "Accumulator" => "",
            "Zero" | "ZeroX" | "ZeroY" | "Relative" | "IndirectX" | "IndirectY" | "Immediate" => {
                "(self.load_pc())"
            }
            "Indirect" | "Absolute" | "AbsoluteX" | "AbsoluteY" => "(self.load_pc_u16())",
            _ => {
                println!("{mode}");
                unreachable!()
            }
        };
        if !names.contains(&name) {
            names.push(name);
        }

        parsing.write_all(format!("{opcode}=>Instruction{{opcode:Opcode::{name},addr:Address::{mode}{operands} }},").as_bytes()).unwrap();
    }

    for name in names {
        opcodes.write_all(format!("{name},").as_bytes()).unwrap();
    }

    opcodes.write_all(b"}").unwrap();
    parsing
        .write_all(b"_=>panic!(\"\\\"illegal\\\" opcode\")}}}")
        .unwrap();

    // format the output
    std::process::Command::new("rustfmt")
        .arg(format!("{output}/opcodes.rs"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    std::process::Command::new("rustfmt")
        .arg(format!("{output}/parsing.rs"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    // Compilation of C source
    let out = std::process::Command::new("make").arg("cc65").spawn().unwrap().wait_with_output().unwrap();
    println!("{}", String::from_utf8(out.stdout).unwrap());
    eprint!("{}", String::from_utf8(out.stderr).unwrap());
    if !out.status.success() {
        panic!("Error while compiling C code")
    }
    std::process::Command::new("da65").arg("${OUT_DIR} -o asm.a").spawn().unwrap().wait().unwrap();
}
