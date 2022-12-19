use instruction::{Instruction, Opcode, AddressingMode};

mod instruction;

pub struct Cpu<B, C> {
    bus: B,
    pc: u16, // program counter
    sp: u8, // stack pointer
    // index registers
    x: u8,
    y: u8,

    flags: u8,
    accumulator: u8,

    clock: C,
}

include!(concat!(env!("OUT_DIR"),"/parsing.rs"));

impl<B: Bus, C> Cpu<B, C> {
    /// Loads the value `pc` is pointing to and increments `pc`.
    fn load_pc(&mut self) -> u8 {
        let value = self.bus.load(self.pc);
        self.pc+=1;
        value
    }

    fn load_pc_u16(&mut self) -> u16 {
        // least significant byte
        let ls = self.load_pc();
        // most significant byte
        let ms = self.load_pc();
        ((ms as u16) << 8) & ls as u16 
    }
}

impl<B: Bus, C: Clock> Cpu<B, C> {
    pub fn run(&mut self) {
        loop {
            let instruction = self.fetch();
            if self.execute(instruction) {
                break
            }
        }
    }
    /// Executes an instruction, the bool indicates if the instruction was BRK.
    fn execute(&mut self, instruction: Instruction) -> bool { // I had to return this because I can't stop the loop inside of this function
        match instruction.opcode {
            Opcode::BRK => todo!(),
            Opcode::PHP => todo!(),
            Opcode::BPL => todo!(),
            Opcode::CLC => todo!(),
            Opcode::ORA => todo!(),
            Opcode::ASL => todo!(),
            Opcode::JSR => todo!(),
            Opcode::BIT => todo!(),
            Opcode::PLP => todo!(),
            Opcode::BMI => todo!(),
            Opcode::SEC => todo!(),
            Opcode::AND => todo!(),
            Opcode::ROL => todo!(),
            Opcode::RTI => todo!(),
            Opcode::PHA => todo!(),
            Opcode::JMP => todo!(),
            Opcode::BVC => todo!(),
            Opcode::CLI => todo!(),
            Opcode::EOR => todo!(),
            Opcode::LSR => todo!(),
            Opcode::RTS => todo!(),
            Opcode::PLA => todo!(),
            Opcode::BVS => todo!(),
            Opcode::SEI => todo!(),
            Opcode::ADC => todo!(),
            Opcode::ROR => todo!(),
            Opcode::STY => todo!(),
            Opcode::DEY => todo!(),
            Opcode::BCC => todo!(),
            Opcode::TYA => todo!(),
            Opcode::STA => todo!(),
            Opcode::STX => todo!(),
            Opcode::TXA => todo!(),
            Opcode::TXS => todo!(),
            Opcode::LDY => todo!(),
            Opcode::TAY => todo!(),
            Opcode::BCS => todo!(),
            Opcode::CLV => todo!(),
            Opcode::LDA => todo!(),
            Opcode::LDX => todo!(),
            Opcode::TAX => todo!(),
            Opcode::TSX => todo!(),
            Opcode::CPY => todo!(),
            Opcode::INY => todo!(),
            Opcode::BNE => todo!(),
            Opcode::CLD => todo!(),
            Opcode::CMP => todo!(),
            Opcode::DEC => todo!(),
            Opcode::DEX => todo!(),
            Opcode::CPX => todo!(),
            Opcode::INX => todo!(),
            Opcode::BEQ => todo!(),
            Opcode::SED => todo!(),
            Opcode::SBC => todo!(),
            Opcode::INC => todo!(),
            Opcode::NOP => todo!(),
        }
    }
}

// In order to avoid writing the same code 8 times, I defined a macro that does it for me.
macro_rules! flag {
    ($name:ident, $set_name:ident, $bit:literal) => {
        pub fn $name(&self) -> bool {
            // This gets the nth bit of a byte and returns it as a boolean
            ((self.flags & (1 << $bit)) >> $bit) != 0
        }
        pub fn $set_name(&mut self, bit: bool) {
            self.flags &= (bit as u8) << $bit 
        }
    };
}

impl<B, C> Cpu<B, C> {    
    flag!(negative, set_negative, 7);
    flag!(overflow, set_overflow, 6);
    flag!(reserved, set_reserved, 5);
    flag!(r#break, set_break, 4);
    flag!(decimal, set_decimal, 3);
    flag!(interrupt_disable, set_interrupt_disable, 2);
    flag!(zero, set_zero, 1);
    flag!(carry, set_carry, 0);
}

pub trait Bus {
    fn load(&self, addr: u16) -> u8;
    fn store(&mut self, addr: u16, value: u8) -> u8;
}

pub trait Clock {
    /// Waits for n amount of cycles.
    fn cycles(n: u8);
}