pub struct Instruction {
    pub opcode: Opcode,
    pub address: Address,
}

include!(concat!(env!("OUT_DIR"), "/opcodes.rs"));

/// The addressing mode and operands.
#[derive(PartialEq, Eq, Debug)]
pub enum Address {
    Zero(u8),
    Implied,
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    ZeroX(u8),
    ZeroY(u8),
    Relative(u8),
    Accumulator,
    Indirect(u16),
    IndirectX(u8),
    IndirectY(u8),
    Immediate(u8),
}
