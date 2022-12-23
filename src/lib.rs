use instruction::{Address, Instruction, Opcode};

mod instruction;

//TODO: Reduce code duplication for clock cycles

pub struct Cpu<B, C> {
    bus: B,
    pc: u16, // program counter
    sp: u8,  // stack pointer
    // index registers
    x: u8,
    y: u8,

    status: u8,
    accumulator: u8,

    clock: C,
}

include!(concat!(env!("OUT_DIR"), "/parsing.rs"));

impl<B: Bus, C> Cpu<B, C> {
    /// Loads the value `pc` is pointing to and increments `pc`.
    fn load_pc(&mut self) -> u8 {
        let value = self.bus.load(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    fn load_pc_u16(&mut self) -> u16 {
        let value = self.bus.load_u16(self.pc);
        self.pc = self.pc.wrapping_add(2);
        value
    }

    /// Pushes a value onto the stack.
    fn push(&mut self, value: u8) {
        // the stack is in the 0x01 memory page
        self.bus.store(0x0100 & self.sp as u16, value);
        self.sp = self.sp.wrapping_add(1);
    }

    fn push_u16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.push(bytes[0]);
        self.push(bytes[1]);
    }

    /// Pops a value from the stack.
    fn pop(&mut self) -> u8 {
        let value = self.bus.load(0x0100 & self.sp as u16);
        self.sp = self.sp.wrapping_sub(1);
        value
    }

    fn pop_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }
}

impl<B: Bus, C: Clock> Cpu<B, C> {
    pub fn run(&mut self) {
        loop {
            let instruction = self.fetch();
            if self.execute(instruction) {
                break;
            }
        }
    }
    /// Executes an instruction, the bool indicates if the instruction was BRK.
    fn execute(&mut self, instruction: Instruction) -> bool {
        match instruction.opcode {
            Opcode::BRK => {
                // Push the program counter + 2 onto the stack.
                self.push_u16(self.pc.wrapping_add(2));

                // Set the break flag to true and push the status register onto the stack.
                self.set_break(true);
                self.push(self.status);
                self.clock.cycles(7);
                return true;
            }
            Opcode::PHP => {
                self.set_break(true);
                self.set_reserved(true);
                self.push(self.status);
                self.clock.cycles(3);
            }
            Opcode::BPL => self.branch(!self.negative(), instruction.addr),
            Opcode::CLC => self.set_carry(false),
            Opcode::ORA => {
                let (value, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    Address::AbsoluteX(addr) => {
                        let final_addr = addr.wrapping_add(self.x as u16);
                        let value = self.bus.load(final_addr);
                        let ncycles = if addr & 0xff00 == final_addr & 0xff00 {
                            // same memory page
                            5
                        } else {
                            // different memory page
                            6
                        };
                        (value, ncycles)
                    }
                    Address::AbsoluteY(addr) => {
                        //TODO: Reduce code duplication
                        let final_addr = addr.wrapping_add(self.y as u16);
                        let value = self.bus.load(final_addr);
                        let ncycles = if addr & 0xff00 == final_addr & 0xff00 {
                            // same memory page
                            5
                        } else {
                            // different memory page
                            6
                        };
                        (value, ncycles)
                    }
                    Address::ZeroX(addr) => (self.bus.load(addr.wrapping_add(self.x) as u16), 4),
                    Address::IndirectX(indirect) => {
                        let address = self
                            .bus
                            .load_u16(indirect as u16)
                            .wrapping_add(self.x as u16);
                        (self.bus.load(address), 6)
                    }
                    Address::IndirectY(indirect) => {
                        // load the address stored in zero page
                        let address = self.bus.load_u16(indirect as u16);
                        // add the y register to it.
                        let final_address = address.wrapping_add(self.y as u16);
                        let value = self.bus.load(final_address);
                        let ncycles = if address & 0xff00 == final_address & 0xff00 {
                            // same memory page
                            5
                        } else {
                            // different memory page
                            6
                        };
                        (value, ncycles)
                    }
                    Address::Immediate(value) => (value, 2),
                    _ => unreachable!(),
                };
                self.accumulator |= value;
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 != 0);
                self.clock.cycles(ncycles)
            }
            Opcode::ASL => {
                let (value, ncycles, addr) = match instruction.addr {
                    Address::Accumulator => {
                        (self.accumulator, 2, None)
                    }
                    Address::Zero(addr) => (self.bus.load(addr as u16), 5, Some(addr as u16)),
                    Address::ZeroX(addr) => {
                        let addr = addr.wrapping_add(self.x) as u16;
                        (self.bus.load(addr), 6, Some(addr))
                    },
                    Address::Absolute(addr) => (self.bus.load(addr), 6, Some(addr)),
                    Address::AbsoluteX(addr) => {
                        let addr = addr.wrapping_add(self.x as u16);
                        (self.bus.load(addr), 7, Some(addr))
                    }
                    _ => unreachable!(),
                };
                if value & 0x80 == 1 {
                    // check for carry
                    self.set_carry(true)
                }

                if let Some(addr) = addr {
                    self.bus.store(addr, value << 1)
                } else {
                    self.accumulator = value << 1;
                }
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 != 0);
                self.clock.cycles(ncycles)
            }
            Opcode::JSR => todo!(),
            Opcode::BIT => todo!(),
            Opcode::PLP => todo!(),
            Opcode::BMI => self.branch(self.negative(), instruction.addr),
            Opcode::SEC => todo!(),
            Opcode::AND => todo!(),
            Opcode::ROL => todo!(),
            Opcode::RTI => todo!(),
            Opcode::PHA => todo!(),
            Opcode::JMP => todo!(),
            Opcode::BVC => self.branch(!self.overflow(), instruction.addr),
            Opcode::CLI => todo!(),
            Opcode::EOR => todo!(),
            Opcode::LSR => todo!(),
            Opcode::RTS => todo!(),
            Opcode::PLA => todo!(),
            Opcode::BVS => self.branch(self.overflow(), instruction.addr),
            Opcode::SEI => todo!(),
            Opcode::ADC => todo!(),
            Opcode::ROR => todo!(),
            Opcode::STY => todo!(),
            Opcode::DEY => todo!(),
            Opcode::BCC => self.branch(!self.carry(), instruction.addr),
            Opcode::TYA => todo!(),
            Opcode::STA => todo!(),
            Opcode::STX => todo!(),
            Opcode::TXA => todo!(),
            Opcode::TXS => todo!(),
            Opcode::LDY => todo!(),
            Opcode::TAY => todo!(),
            Opcode::BCS => self.branch(self.carry(), instruction.addr),
            Opcode::CLV => todo!(),
            Opcode::LDA => todo!(),
            Opcode::LDX => todo!(),
            Opcode::TAX => todo!(),
            Opcode::TSX => todo!(),
            Opcode::CPY => todo!(),
            Opcode::INY => todo!(),
            Opcode::BNE => self.branch(!self.zero(), instruction.addr),
            Opcode::CLD => todo!(),
            Opcode::CMP => todo!(),
            Opcode::DEC => todo!(),
            Opcode::DEX => todo!(),
            Opcode::CPX => todo!(),
            Opcode::INX => todo!(),
            Opcode::BEQ => self.branch(self.zero(), instruction.addr),
            Opcode::SED => todo!(),
            Opcode::SBC => todo!(),
            Opcode::INC => todo!(),
            Opcode::NOP => {
                self.clock.cycles(2);
            }
        }
        false
    }

    fn branch(&mut self, flag: bool, address: Address) {
        if flag {
            if let Address::Relative(address) = address {
                let most_significant = self.pc.to_le_bytes()[1]; //
                self.pc = (self.pc as i16).wrapping_add((address as i8).into()) as u16;
                if self.pc.to_le_bytes()[1] == most_significant {
                    // branch on same memory page
                    self.clock.cycles(3)
                } else {
                    // branch on a different memory page
                    self.clock.cycles(4)
                }
            } else {
                panic!("illegal addressing mode")
            }
        } else {
            self.clock.cycles(2)
        }
    }
}

// In order to avoid writing the same code 8 times, I defined a macro that does it for me.
macro_rules! flag {
    ($name:ident, $set_name:ident, $bit:literal) => {
        pub fn $name(&self) -> bool {
            // This gets the nth bit of a byte and returns it as a boolean
            ((self.status & (1 << $bit)) >> $bit) != 0
        }
        pub fn $set_name(&mut self, bit: bool) {
            self.status &= (bit as u8) << $bit
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
    fn load_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load(addr), self.load(addr.wrapping_add(1))])
    }
    fn store(&mut self, addr: u16, value: u8);
    fn store_u16(&mut self, addr: u16, value: u16) {
        let bytes = value.to_le_bytes();
        self.store(addr, bytes[0]);
        self.store(addr.wrapping_add(1), bytes[1]);
    }
}

pub trait Clock {
    /// Waits for n amount of cycles.
    fn cycles(&mut self, n: u8);
}
