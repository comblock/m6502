use instruction::{Address, Instruction, Opcode};

mod instruction;

//TODO: Reduce code duplication

#[derive(PartialEq, Eq, Debug)]
pub struct Cpu<B, C> {
    pub bus: B,
    pub pc: u16, // program counter
    pub sp: u8,  // stack pointer
    // index registers
    pub x: u8,
    pub y: u8,

    pub status: u8,
    pub accumulator: u8,

    pub clock: C,
}

include!(concat!(env!("OUT_DIR"), "/parsing.rs"));

impl<B: Bus, C> Cpu<B, C> {
    pub fn new(bus: B, clock: C) -> Self {
        Self {
            x: 0,
            y: 0,
            status: 0b00100000,
            accumulator: 0,
            sp: 0,
            clock,
            pc: 0x0200,
            bus
        }
    }

    pub fn with_state(bus: B, clock: C, x: u8, y: u8, status: u8, accumulator: u8, sp: u8, pc: u16) -> Self {
        let mut this = Self {
            bus,
            clock,
            pc,
            sp,
            x,
            y,
            status,
            accumulator,

        };
        this.set_reserved(true);
        this
    }
}

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
        self.bus.store(0x0100 | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_u16(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        // pushed in reverse order because it's stored as LE but the stack decrements.
        self.push(bytes[1]);
        self.push(bytes[0]);
    }

    /// Pops a value from the stack.
    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let value = self.bus.load(0x0100 | self.sp as u16);
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
                    4
                } else {
                    // different memory page
                    5
                };
                (value, ncycles)
            }
            Address::AbsoluteY(addr) => {
                let final_addr = addr.wrapping_add(self.y as u16);
                let value = self.bus.load(final_addr);
                let ncycles = if addr & 0xff00 == final_addr & 0xff00 {
                    // same memory page
                    4
                } else {
                    // different memory page
                    5
                };
                (value, ncycles)
            }
            Address::ZeroX(addr) => (self.bus.load(addr.wrapping_add(self.x) as u16), 4),
            Address::IndirectX(indirect) => {
                let addr = self
                    .bus
                    .load_u16_zp(indirect.wrapping_add(self.x));
                (self.bus.load(addr), 6)
            }
            Address::IndirectY(indirect) => {
                // load the address stored in zero page
                let addr = self.bus.load_u16_zp(indirect);
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
    pub fn execute(&mut self, instruction: Instruction) -> bool {
        let ncycles = match instruction.opcode {
            Opcode::BRK => {
                // Push the program counter + 2 onto the stack.
                self.push_u16(self.pc.wrapping_add(1)); // this is 1 because we already incremented by 1 while fetching

                // Set the break flag to true and push the status register onto the stack.
                self.push(self.status | 0b00010000);
                self.set_interrupt_disable(true);
                self.pc = self.bus.load_u16(0xFFFE);
                self.clock.cycles(7);
                return true;
            }
            Opcode::PHP => {
                self.push(self.status | 0b00110000);
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
                self.set_carry(value & 0x80 == 0x80);

                let result = value << 1;
                if let Some(addr) = addr {
                    self.bus.store(addr, result)
                } else {
                    self.accumulator = result;
                }
                self.set_zero(result == 0);
                self.set_negative(result & 0x80 == 0x80);
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
                self.set_negative(value & 0x80 == 0x80);
                self.set_overflow(value & 0x40 == 0x40);
                let result = value & self.accumulator;
                self.set_zero(result == 0);
                ncycles
            }
            Opcode::PLP => {
                self.status = (self.pop() & 0xEF) | 0x20;
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
                let (value, ncycles, addr): (u8, u8, Option<u16>) = self.shift_operands(instruction.addr);
                let old_carry = self.carry();
                self.set_carry(value & 0x80 == 0x80);
                let result = value << 1 | old_carry as u8;
                self.set_negative(result & 0x80 == 0x80);
                self.set_zero(result == 0);
                if let Some(addr) = addr {
                    self.bus.store(addr, result)
                } else {
                    self.accumulator = result
                }
                ncycles
            },
            Opcode::RTI => {
                self.status = (self.pop() & 0xEF) | 0x20;
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
                    Address::Indirect(addr) => {
                        let ls = self.bus.load(addr);
                        let ms = self.bus.load((addr as u8).wrapping_add(1) as u16 | (addr & 0xff00));
                        (u16::from_le_bytes([ls, ms]), 5)
                    },
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
                self.set_carry(value & 1 == 1);

                let result = value >> 1;
                if let Some(addr) = addr {
                    self.bus.store(addr, result)
                } else {
                    self.accumulator = result;
                }
                self.set_zero(result == 0);
                self.set_negative(result & 0x80 == 0x80);
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
                self.adc(value);
                ncycles
            },
            Opcode::ROR => {
                let (value, ncycles, addr): (u8, u8, Option<u16>) = self.shift_operands(instruction.addr);
                let old_carry = self.carry();
                self.set_carry(value & 1 == 1);
                let result = value >> 1 | ((old_carry as u8) << 7);
                self.set_negative(result & 0x80 == 0x80);
                self.set_zero(result == 0);
                if let Some(addr) = addr {
                    self.bus.store(addr, result)
                } else {
                    self.accumulator = result
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
                self.y = self.y.wrapping_sub(1);
                self.set_zero(self.y == 0);
                self.set_negative(self.y & 0x80 == 0x80);
                2
            },
            Opcode::BCC => self.branch(!self.carry(), instruction.addr),
            Opcode::TYA => {
                self.accumulator = self.y;
                self.set_negative(self.accumulator & 0x80 == 0x80);
                self.set_zero(self.accumulator == 0);
                2
            },
            Opcode::STA => {
                let (addr, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (addr as u16, 3),
                    Address::Absolute(addr) => (addr, 4),
                    Address::AbsoluteX(addr) => {
                        (addr.wrapping_add(self.x as u16), 5)
                    }
                    Address::AbsoluteY(addr) => {
                        (addr.wrapping_add(self.y as u16), 5)
                    }
                    Address::ZeroX(addr) => (addr.wrapping_add(self.x) as u16, 4),
                    Address::IndirectX(indirect) => {
                        let addr = self
                            .bus
                            .load_u16_zp(indirect.wrapping_add(self.x));
                        (addr, 6)
                    }
                    Address::IndirectY(indirect) => {
                        // load the address stored in zero page
                        let addr = self.bus.load_u16_zp(indirect);
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
                    Address::ZeroY(addr) => (addr.wrapping_add(self.y) as u16, 4),
                    Address::Absolute(addr) => (addr, 4),
                    _ => unreachable!()
                };
                self.bus.store(addr, self.x);
                ncycles
            },
            Opcode::TXA => {
                self.accumulator = self.x;
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
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
                self.set_zero(self.y == 0);
                self.set_negative(self.y & 0x80 == 0x80);
                ncycles
            },
            Opcode::TAY => {
                self.y = self.accumulator;
                self.set_zero(self.y == 0);
                self.set_negative(self.y & 0x80 == 0x80);
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
                self.set_zero(self.accumulator == 0);
                self.set_negative(self.accumulator & 0x80 == 0x80);
                ncycles
            },
            Opcode::LDX => {
                let (value, ncycles) = match instruction.addr {
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::ZeroY(addr) => (self.bus.load(addr.wrapping_add(self.y) as u16), 4),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    Address::AbsoluteY(addr) => {
                        let final_addr = addr.wrapping_add(self.y as u16);
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
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 0x80 == 0x80);
                ncycles
            },
            Opcode::CPY => {
                let (value, ncycles) = match instruction.addr {
                    Address::Immediate(value) => (value, 2),
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    _ => unreachable!() 
                };
                self.set_carry(self.y >= value);
                let temp = self.y.wrapping_sub(value);
                self.set_zero(temp == 0);
                self.set_negative(temp & 0x80 == 0x80);
                ncycles
            },
            Opcode::TAX => {
                self.x = self.accumulator;
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 0x80 == 0x80);
                2
            },
            Opcode::TSX => {
                self.x = self.sp;
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 0x80 == 0x80);
                2
            },
            Opcode::INY => {
                self.y = self.y.wrapping_add(1);
                self.set_zero(self.y == 0);
                self.set_negative(self.y & 0x80 == 0x80);
                2
            },
            Opcode::BNE => self.branch(!self.zero(), instruction.addr),
            Opcode::CLD => {
                self.set_decimal(false); // decimal mode is not supported even though I implemented this instruction;
                2
            },
            Opcode::CMP => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                self.set_carry(self.accumulator >= value);
                let temp = self.accumulator.wrapping_sub(value);
                self.set_zero(temp == 0);
                self.set_negative(temp & 0x80 == 0x80);
                ncycles
                // 0b 0110 0101
                // 0b 1110 0100

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
                self.x = self.x.wrapping_sub(1);
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 0x80 == 0x80);
                2
            },
            Opcode::CPX => {
                let (value, ncycles) = match instruction.addr {
                    Address::Immediate(value) => (value, 2),
                    Address::Zero(addr) => (self.bus.load(addr as u16), 3),
                    Address::Absolute(addr) => (self.bus.load(addr), 4),
                    _ => unreachable!() 
                };
                self.set_carry(self.x >= value);
                let temp = self.x.wrapping_sub(value);
                self.set_zero(temp == 0);
                self.set_negative(temp & 0x80 == 0x80);
                ncycles
            },
            Opcode::INX => {
                self.x = self.x.wrapping_add(1);
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 0x80 == 0x80);
                2
            },
            Opcode::BEQ => {self.branch(self.zero(), instruction.addr)},
            Opcode::SED => {
                eprintln!("decimal mode set but it's not supported!");
                self.set_decimal(true); // even though this instruction is implemented, decimal mode is not supported
                2
            },
            Opcode::SBC => {
                let (value, ncycles) = self.alu_operands(instruction.addr);
                self.adc(!value);
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
                self.set_negative(value & 0x80 == 0x80);
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
                self.pc = (self.pc as i16).wrapping_add(address as i8 as i16) as u16;
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

    fn adc(&mut self, value: u8) {
        let mut temp = self.accumulator as u16;
        temp += value as u16 + self.carry() as u16; // can't overflow
        self.set_overflow((self.accumulator & value & 0x80 != temp as u8 & 0x80) && self.accumulator & 0x80 == value & 0x80);
        self.accumulator = temp as u8;
        
        self.set_carry((temp & 0x0100) == 0x0100);
        self.set_zero(self.accumulator == 0);
        self.set_negative(self.accumulator & 0x80 == 0x80);
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
            self.status = (self.status & !(1 << $bit)) | (bit as u8) << $bit
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
    fn load_u16_zp(&self, addr: u8) -> u16 {
        u16::from_le_bytes([self.load(addr as u16), self.load(addr.wrapping_add(1) as u16)])
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


#[cfg(test)]
mod test {
    use serde_derive::Deserialize;
    use serde_derive::Serialize;

    const URL: &str = "https://raw.githubusercontent.com/TomHarte/ProcessorTests/main/nes6502/v1/";

    #[test]
    fn test() {
        let opcodes = include_str!("../opcodes.txt");
        for opcode in opcodes.lines().map(|v| {
            v[2..4].to_ascii_lowercase()
        }) {
            let tests: Vec<Test> = serde_json::from_reader(ureq::get(&format!("{URL}{opcode}.json")).call().unwrap().into_reader()).unwrap();
            for test in tests {
                let mut cpu: Cpu = test.initial.clone().into();
                let instruction = cpu.fetch();
                cpu.execute(instruction);
                let mut r#final: Cpu = test.r#final.clone().into();
                r#final.clock = cpu.clock;
                assert_eq!(cpu, r#final);
                assert_eq!(cpu.clock.cpassed(), test.cycles.len() as u64);
            }
        }
    
    }

    type Cpu = super::Cpu<Bus, Clock>;
    
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Test {
        pub name: String,
        pub initial: State,
        #[serde(rename = "final")]
        pub r#final: State,
        pub cycles: Vec<(i64, i64, String)>,
    }
    
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct State {
        pub pc: u16,
        pub s: u8,
        pub a: u8,
        pub x: u8,
        pub y: u8,
        pub p: u8,
        pub ram: Vec<(u16, u8)>,
    }

    impl Into<Cpu> for State {
        fn into(self) -> super::Cpu<Bus, Clock> {
            let mut bus = Bus::new();
            for i in self.ram {
                (&mut bus as &mut dyn super::Bus).store(i.0, i.1 as u8);
            }
            super::Cpu::with_state(bus, Clock::new(), self.x, self.y, self.p, self.a, self.s, self.pc)
        }
    }

    #[derive(PartialEq, Eq)]
    struct Bus(Box<[u8; 2usize.pow(16)]>);
    
    impl std::fmt::Debug for Bus {
        fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    impl super::Bus for Bus {
        fn load(&self, addr: u16) -> u8 {
            self.0[addr as usize]
    }

        fn store(&mut self, addr: u16, value: u8) {
            self.0[addr as usize] = value;
    }
    }
    
    impl Bus {
        pub fn new() -> Self {
            Self(Box::new([0;2usize.pow(16)]))
        }

        pub fn print_stack(&self) {
            println!("{:?}", &self.0[0x0100..0x0200]);
        }

        pub fn print_zp(&self) {
            println!("{:?}", &self.0[0..0x0100])
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone, Copy)]
    struct Clock(u64);

    impl Clock {
        pub fn new() -> Self {
            Self(0)
        }

        /// Cycles passed
        pub fn cpassed(self) -> u64 {
            self.0
        }
    }

    impl super::Clock for Clock {
        fn cycles(&mut self, n: u8) {
            self.0+= n as u64;
        }
    }
}
