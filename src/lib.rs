use instruction::{Address, Instruction, Opcode};

mod instruction;

//TODO: Reduce code duplication

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
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_u16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.push(bytes[0]);
        self.push(bytes[1]);
    }

    /// Pops a value from the stack.
    fn pop(&mut self) -> u8 {
        let value = self.bus.load(0x0100 & self.sp as u16);
        self.sp = self.sp.wrapping_add(1);
        value
    }

    fn pop_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    /// This is a helper method for ALU operations.
    /// Returns a value, ncycles and optionally an address.
    /// If the address is None, the accumulator should be used.
    fn alu_operands(&self, addr: Address) -> (u8, u8) {
        match addr {
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
                let addr = self
                    .bus
                    .load_u16(indirect as u16)
                    .wrapping_add(self.x as u16);
                (self.bus.load(addr), 6)
            }
            Address::IndirectY(indirect) => {
                // load the address stored in zero page
                let addr = self.bus.load_u16(indirect as u16);
                // add the y register to it.
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
            Address::Immediate(value) => (value, 2),
            _ => unreachable!(),
        }
    }

    /// This is a helper method for shift operations.
    /// Returns a value, ncycles and optionally an address.
    /// If the address is None, the accumulator should be used.
    fn shift_operands(&self, addr: Address) -> (u8, u8, Option<u16>) {
        match addr {
            Address::Accumulator => (self.accumulator, 2, None),
            Address::Zero(addr) => (self.bus.load(addr as u16), 5, Some(addr as u16)),
            Address::ZeroX(addr) => {
                let addr = addr.wrapping_add(self.x) as u16;
                (self.bus.load(addr), 6, Some(addr))
            }
            Address::Absolute(addr) => (self.bus.load(addr), 6, Some(addr)),
            Address::AbsoluteX(addr) => {
                let addr = addr.wrapping_add(self.x as u16);
                (self.bus.load(addr), 7, Some(addr))
            }
            _ => unreachable!(),
        }
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
        let ncycles = match instruction.opcode {
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
                3
            }
            Opcode::BPL => self.branch(!self.negative(), instruction.addr),
            Opcode::CLC => {
                self.set_carry(false);
                2
            }
            Opcode::ORA => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                self.accumulator |= value;
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            }
            Opcode::ASL => {
                let (value, ncycles, addr) = self.shift_operands(instruction.addr);
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
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            }
            Opcode::JSR => {
                let addr = if let Address::Absolute(addr) = instruction.addr {
                    addr
                } else {
                    unreachable!()
                };
                // push the last byte of the instruction to the stack
                self.push_u16(self.pc - 1);

                self.pc = addr;
                6
            }
            Opcode::BIT => {
                let (value, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    _ => unreachable!(),
                };
                let value = value & self.accumulator;
                if value == 0 {
                    self.set_zero(true)
                } else {
                    self.set_zero(false)
                }
                self.status &= value & 0xC0;
                ncycles
            }
            Opcode::PLP => {
                self.status &= self.pop() & 0xCF;
                4
            },
            Opcode::BMI => self.branch(self.negative(), instruction.addr),
            Opcode::SEC => {
                self.set_carry(true);
                2
            },
            Opcode::AND => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                self.accumulator &= value;
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            },
            Opcode::ROL => {
                let (mut value, ncycles, addr): (u8, u8, Option<u16>) = self.shift_operands(instruction.addr);
                self.set_carry(self.accumulator & 0x80 == 0x80);
                value = value.rotate_left(1);
                if let Some(addr) = addr {
                    self.bus.store(addr, value)
                } else {
                    self.accumulator = value
                }
                ncycles
            },
            Opcode::RTI => {
                self.status = self.pop() & 0xCF;
                self.pc = self.pop_u16();
                6
            },
            Opcode::PHA => {
                self.push(self.accumulator);
                3
            },
            Opcode::JMP => {
                let (value, ncycles) = match instruction.addr {
                    Address::Absolute(value) => (value, 3),
                    Address::Indirect(addr) => (self.bus.load_u16(addr), 5),
                    _ => unreachable!()
                };
                self.pc = value;
                ncycles
            },
            Opcode::BVC => self.branch(!self.overflow(), instruction.addr),
            Opcode::CLI => {
                self.set_interrupt_disable(false);
                2
            },
            Opcode::EOR => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                self.accumulator ^= value;
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            },
            Opcode::LSR => {
                let (value, ncycles, addr) = self.shift_operands(instruction.addr);
                if value & 0x80 == 1 {
                    // check for carry
                    self.set_carry(true)
                }

                if let Some(addr) = addr {
                    self.bus.store(addr, value >> 1)
                } else {
                    self.accumulator = value >> 1;
                }
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            },
            Opcode::RTS => {
                self.pc = self.pop_u16()+1;
                6
            },
            Opcode::PLA => {
                self.accumulator = self.pop();
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                4
            },
            Opcode::BVS => self.branch(self.overflow(), instruction.addr),
            Opcode::SEI => {
                self.set_interrupt_disable(true);
                2
            },
            Opcode::ADC => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                let mut temp = self.accumulator as u16;
                temp += value as u16 + self.carry() as u16; // can't overflow
                self.accumulator = temp as u8;
                
                self.set_carry((temp & 0x10) == 0x10);
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            },
            Opcode::ROR => {
                let (mut value, ncycles, addr): (u8, u8, Option<u16>) = self.shift_operands(instruction.addr);
                self.set_carry(self.accumulator & 0x01 != 0);
                value = value.rotate_right(1);
                if let Some(addr) = addr {
                    self.bus.store(addr, value)
                } else {
                    self.accumulator = value
                }
                ncycles
            },
            Opcode::STY => {
                let (addr, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (addr as u16, 3),
                    Address::ZeroX(addr) => (addr.wrapping_add(self.x) as u16, 4),
                    Address::Absolute(addr) => (addr, 4),
                    _ => unreachable!()
                };
                self.bus.store(addr, self.y);
                ncycles
            },
            Opcode::DEY => {
                self.set_carry(self.y == 0);
                self.y = self.y.wrapping_sub(1);
                self.set_zero(self.y == 0);
                2
            },
            Opcode::BCC => self.branch(!self.carry(), instruction.addr),
            Opcode::TYA => {
                self.accumulator = self.y;
                2
            },
            Opcode::STA => {
                let (addr, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (addr as u16, 3),
                    Address::Absolute(addr) => (addr, 4),
                    Address::AbsoluteX(addr) => {
                        (addr.wrapping_add(self.y as u16), 5)
                    }
                    Address::AbsoluteY(addr) => {
                        (addr.wrapping_add(self.y as u16), 5)
                    }
                    Address::ZeroX(addr) => (addr.wrapping_add(self.x) as u16, 4),
                    Address::IndirectX(indirect) => {
                        let addr = self
                            .bus
                            .load_u16(indirect as u16)
                            .wrapping_add(self.x as u16);
                        (addr, 6)
                    }
                    Address::IndirectY(indirect) => {
                        // load the address stored in zero page
                        let addr = self.bus.load_u16(indirect as u16);
                        // add the y register to it.
                        (addr.wrapping_add(self.y as u16), 6)
                    }
                    _ => unreachable!(),
                };
                self.bus.store(addr, self.accumulator);
                ncycles
                
            },
            Opcode::STX => {
                let (addr, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (addr as u16, 3),
                    Address::ZeroX(addr) => (addr.wrapping_add(self.x) as u16, 4),
                    Address::Absolute(addr) => (addr, 4),
                    _ => unreachable!()
                };
                self.bus.store(addr, self.x);
                ncycles
            },
            Opcode::TXA => {
                self.accumulator = self.x;
                2
            },
            Opcode::TXS => {
                self.sp = self.x;
                2
            },
            Opcode::LDY => {
                let (value, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::ZeroX(addr) => (self.bus.load(addr.wrapping_add(self.x) as u16), 4),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    Address::AbsoluteX(addr) => {
                        let final_addr = addr.wrapping_add(self.x as u16);
                        let ncycles = if final_addr & 0xff00 == addr & 0xff00 {
                            4
                        } else {
                            5
                        };
                        (self.bus.load(final_addr), ncycles)
                    },
                    Address::Immediate(value) => (value, 2),
                    _ => unreachable!()
                };
                self.y = value;
                ncycles
            },
            Opcode::TAY => {
                self.y = self.accumulator;
                2
            },
            Opcode::BCS => self.branch(self.carry(), instruction.addr),
            Opcode::CLV => {
                self.set_overflow(false);
                2
            },
            Opcode::LDA => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                self.accumulator = value;
                ncycles
            },
            Opcode::LDX => {
                let (value, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::ZeroX(addr) => (self.bus.load(addr.wrapping_add(self.x) as u16), 4),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    Address::AbsoluteX(addr) => {
                        let final_addr = addr.wrapping_add(self.x as u16);
                        let ncycles = if final_addr & 0xff00 == addr & 0xff00 {
                            4
                        } else {
                            5
                        };
                        (self.bus.load(final_addr), ncycles)
                    },
                    Address::Immediate(value) => (value, 2),
                    _ => unreachable!()
                };
                self.x = value;
                ncycles
            },
            Opcode::CPY => {
                let (value, ncycles) = match instruction.addr {
                    Address::Immediate(value) => (value, 2),
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    _ => unreachable!() 
                };
                let temp = self.y.wrapping_sub(value);
                self.set_zero(temp == 0);
                self.set_carry(value > self.y);
                self.set_negative(value & 0x80 == 0x80);
                ncycles
            },
            Opcode::TAX => {
                self.x = self.accumulator;
                2
            },
            Opcode::TSX => {
                self.x = self.sp;
                2
            },
            Opcode::INY => {
                self.set_carry(self.y == 255);
                self.y = self.y.wrapping_add(1);
                self.set_zero(self.y == 0);
                2
            },
            Opcode::BNE => self.branch(!self.zero(), instruction.addr),
            Opcode::CLD => {
                self.set_decimal(false); // decimal mode is not supported even though I implemented this instruction;
                2
            },
            Opcode::CMP => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                let temp = self.accumulator.wrapping_sub(value);
                self.set_zero(temp == 0);
                self.set_carry(value > self.accumulator);
                self.set_negative(value & 0x80 == 0x80);
                ncycles
            },
            Opcode::DEC => {
                let (addr, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (addr as u16, 5),
                    Address::ZeroX(addr) => (addr.wrapping_add(self.x) as u16, 6),
                    Address::Absolute(addr) => (addr, 6),
                    Address::AbsoluteX(addr) => (addr.wrapping_add(self.x as u16), 7),
                    _ => unreachable!()
                };
                let value = self.bus.load(addr).wrapping_sub(1);
                self.set_zero(value == 0);
                self.set_negative(value & 0x80 == 0x80); 
                self.bus.store(addr, value);
                ncycles
            },
            Opcode::DEX => {
                self.set_carry(self.x == 0);
                self.x = self.x.wrapping_sub(1);
                self.set_zero(self.x == 0);
                2
            },
            Opcode::CPX => {
                let (value, ncycles) = match instruction.addr {
                    Address::Immediate(value) => (value, 2),
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    _ => unreachable!() 
                };
                let temp = self.x.wrapping_sub(value);
                self.set_zero(temp == 0);
                self.set_carry(value > self.x);
                self.set_negative(value & 0x80 == 0x80);
                ncycles
            },
            Opcode::INX => {
                self.set_carry(self.x == 255);
                self.x = self.y.wrapping_add(1);
                self.set_zero(self.x == 0);
                2
            },
            Opcode::BEQ => self.branch(self.zero(), instruction.addr),
            Opcode::SED => {
                eprintln!("decimal mode set but it's not supported!");
                self.set_decimal(true); // even though this instruction is implemented, decimal mode is not supported
                2
            },
            Opcode::SBC => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                let temp = (self.accumulator as u16).wrapping_sub(value as u16).wrapping_sub(self.carry() as u16);
                self.accumulator = temp as u8;
                self.set_overflow(temp & 0x0100 == 0x0100); 
                self.set_carry(!(self.accumulator as i8).is_negative());
                self.set_negative(self.accumulator & 0x80 == 0x80);
                self.set_zero(self.accumulator == 0);
                ncycles
            },
            Opcode::INC => {
                let (addr, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (addr as u16, 5),
                    Address::ZeroX(addr) => (addr.wrapping_add(self.x) as u16, 6),
                    Address::Absolute(addr) => (addr, 6),
                    Address::AbsoluteX(addr) => (addr.wrapping_add(self.x as u16), 7),
                    _ => unreachable!()
                };
                let value = self.bus.load(addr).wrapping_add(1);
                self.set_zero(value == 0);
                self.set_negative(0x80 == 0x80);
                self.bus.store(addr, value);
                ncycles
            },
            Opcode::NOP => 2,
        };
        self.clock.cycles(ncycles);
        false
    }

    fn branch(&mut self, flag: bool, address: Address) -> u8 {
        if flag {
            if let Address::Relative(address) = address {
                let most_significant = self.pc.to_le_bytes()[1]; //
                self.pc = (self.pc as i16).wrapping_add((address as i8).into()) as u16;
                if self.pc.to_le_bytes()[1] == most_significant {
                    // branch on same memory page
                    3
                } else {
                    // branch on a different memory page
                    4
                }
            } else {
                panic!("illegal addressing mode")
            }
        } else {
            2
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
