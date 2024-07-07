//! This file describes the structures that defines our VM

use std::collections::HashMap;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Register {
    #[default]
    R0 = 0,
    R1,
}

impl From<Register> for usize {
    fn from(value: Register) -> Self {
        match value {
            Register::R0 => 0,
            Register::R1 => 1,
        }
    }
}

pub const REGISTER_COUNT: usize = std::mem::variant_count::<Register>();

/// All memory locations in this VM are addressed via u8.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemoryLocation(pub u8);

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Instruction {
    Add(Register, Register),
    Sub(Register, Register),
    Mul(Register, Register),
    Div(Register, Register),
    Shl(Register, Register),
    Shr(Register, Register),
    Lb(Register, MemoryLocation),
    Sb(Register, MemoryLocation),
    #[default]
    Halt,
}

impl Instruction {
    /// Not the best of the implementations. But written it like this
    /// for demonstration purposes
    pub fn get_opcode(&self) -> u8 {
        match self {
            Instruction::Add(_, _) => 0,
            Instruction::Sub(_, _) => 1,
            Instruction::Mul(_, _) => 2,
            Instruction::Div(_, _) => 3,
            Instruction::Shl(_, _) => 4,
            Instruction::Shr(_, _) => 5,
            Instruction::Lb(_, _) => 6,
            Instruction::Sb(_, _) => 7,
            Instruction::Halt => 8,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    /// The entrypoint of the program
    pub entry_point: u8,

    /// The code
    pub code: HashMap<u8, Instruction>,

    /// Initial memory layout at the start of the program
    pub memory_init: HashMap<u8, u8>,
}
