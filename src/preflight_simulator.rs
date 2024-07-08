use std::collections::HashMap;

use anyhow::{
    anyhow,
    Context,
    Result,
};

use crate::vm_specs::{
    Instruction,
    Program,
    REGISTER_COUNT,
};

/// Each `SimulationRow` describes the state of simulation at each step
/// of execution
#[derive(Debug)]
pub struct SimulationRow {
    /// Encodes the instruction executed during this "row". This would
    /// be useful when we go for SN/TARK constraining.
    pub instruction: Instruction,

    /// Clock cycle during execution. Supports large cycle count till
    /// `u32::MAX`
    pub clock: u32,

    /// Address of the program instruction
    pub program_counter: u8,

    /// Whether at this row the execution halted. Should only be true
    /// for the last row in any `PreflightSimulation`
    pub is_halted: bool,

    /// Registers
    pub registers: [u8; REGISTER_COUNT],

    /// This ideally should be something like `im::HashMap`, see:
    /// https://crates.io/crates/im for immutable collections.
    /// This is because, more often than not, each subsequent `SimulationRow`
    /// will have very slightly changed memory snapshots. Maybe only one
    /// address's value would have changed for example. Makes sense to
    /// only store the `delta` from the previous hashmap rather than the
    /// full hashmap like we are doing here.
    ///
    /// However, that optimization is not used for simplicity's sake and
    /// since our VM is small, this is not a large performance hit.
    pub memory_snapshot: HashMap<u8, u8>,
}

impl SimulationRow {
    pub fn generate_first_row(prog: &Program) -> Result<Self> {
        let program_counter = prog.entry_point;
        let instruction = prog
            .code
            .get(&program_counter)
            .cloned()
            .context("instruction not found")?;
        Ok(Self {
            instruction,
            clock: 1, // `0` is reserved for memory init
            program_counter,
            is_halted: false,
            registers: [0; REGISTER_COUNT],
            memory_snapshot: prog
                .memory_init
                .clone(),
        })
    }

    pub fn execute_one_cycle(
        &self,
        prog: &Program,
    ) -> Result<Self> {
        // This is mutable precisely because jump instructions can change it
        // in weird ways. This is good default for many other operations though
        let mut program_counter = self.program_counter + 1;
        let clock = self.clock + 1;

        let mut registers = self.registers;

        let mut memory_snapshot = self
            .memory_snapshot
            .clone();

        println!(
            "[Exec] clk: {}, pc: {}, inst: {:?}",
            self.clock, self.program_counter, self.instruction
        );

        match self.instruction {
            Instruction::Add(a, b) => {
                registers[usize::from(a)] = registers[usize::from(a)]
                    .wrapping_add(registers[usize::from(b)]);
            }
            Instruction::Sub(a, b) => {
                registers[usize::from(a)] = registers[usize::from(a)]
                    .wrapping_sub(registers[usize::from(b)]);
            }
            Instruction::Mul(a, b) => {
                registers[usize::from(a)] = registers[usize::from(a)]
                    .wrapping_mul(registers[usize::from(b)]);
            }
            Instruction::Div(a, b) => {
                registers[usize::from(a)] = registers[usize::from(a)]
                    .wrapping_div(registers[usize::from(b)]);
            }
            Instruction::Shl(reg, amount) => {
                registers[usize::from(reg)] = registers[usize::from(reg)]
                    .wrapping_shl(registers[usize::from(amount)].into());
            }
            Instruction::Shr(reg, amount) => {
                registers[usize::from(reg)] = registers[usize::from(reg)]
                    .wrapping_shr(registers[usize::from(amount)].into());
            }
            Instruction::Jz(reg, instloc) => {
                if registers[usize::from(reg)] == 0 {
                    program_counter = instloc.0
                }
            }
            Instruction::Jnz(reg, instloc) => {
                if registers[usize::from(reg)] != 0 {
                    program_counter = instloc.0
                }
            }
            Instruction::Lb(reg, memloc) => {
                registers[usize::from(reg)] = self
                    .memory_snapshot
                    .get(&memloc.0)
                    .copied()
                    .unwrap_or_default(); // We treat uninitialized memory as 0
            }
            Instruction::Sb(reg, memloc) => {
                memory_snapshot
                    .entry(memloc.0)
                    .and_modify(|elem| *elem = registers[usize::from(reg)])
                    .or_insert(registers[usize::from(reg)]);
            }
            Instruction::Halt => { // is a no-op
            }
        };

        let instruction = prog
            .code
            .get(&program_counter)
            .cloned()
            .context("instruction not found")?;

        let is_halted = instruction == Instruction::Halt;

        Ok(Self {
            instruction,
            clock,
            program_counter,
            is_halted,
            registers,
            memory_snapshot,
        })
    }

    pub fn get_memory_at(
        &self,
        address: &u8,
    ) -> Option<u8> {
        self.memory_snapshot
            .get(address)
            .copied()
    }

    pub fn get_registers(&self) -> [u8; REGISTER_COUNT] {
        self.registers
            .clone()
    }
}

/// Unconstrainted Preflight Simulation of the program built
/// by running the code.
#[derive(Debug)]
pub struct PreflightSimulation {
    /// Memory before starting the program, a.k.a `clk = 0`
    pub memory_init: HashMap<u8, u8>,
    /// Step wise execution from `clk = 1`
    pub trace_rows: Vec<SimulationRow>,
}

impl PreflightSimulation {
    /// Maximum number of CPU cycles allowed
    const MAX_CPU_CYCLES_ALLOWED: usize = 1_000;

    /// Entry point to simulate a program and generate a `PreflightSimulation`
    /// to be used to generate tables
    pub fn simulate(prog: &Program) -> Result<Self> {
        if prog
            .code
            .is_empty()
        {
            return Ok(Self {
                memory_init: prog
                    .memory_init
                    .clone(),
                trace_rows: vec![],
            });
        }
        let mut trace_rows =
            Vec::with_capacity(Self::MAX_CPU_CYCLES_ALLOWED / 4);
        let first_row = SimulationRow::generate_first_row(prog)?;
        trace_rows.push(first_row);

        while trace_rows.len() <= Self::MAX_CPU_CYCLES_ALLOWED
            && !trace_rows[trace_rows.len() - 1].is_halted
        {
            let current_row =
                trace_rows[trace_rows.len() - 1].execute_one_cycle(prog)?;
            trace_rows.push(current_row);
        }

        if !trace_rows[trace_rows.len() - 1].is_halted {
            return Err(anyhow!(
                "simulation halted since MAX_CPU_CYCLES_ALLOWED reached"
            ));
        }

        Ok(Self {
            memory_init: prog
                .memory_init
                .clone(),
            trace_rows,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::vm_specs::{
        Instruction,
        InstructionLocation,
        MemoryLocation,
        Program,
        Register,
    };

    #[test]
    /// Tests whether two numbers in memory can be added together
    /// in the ZKVM
    fn test_preflight_add_memory() {
        let instructions = vec![
            Instruction::Lb(Register::R0, MemoryLocation(0x40)),
            Instruction::Lb(Register::R1, MemoryLocation(0x41)),
            Instruction::Add(Register::R0, Register::R1),
            Instruction::Sb(Register::R0, MemoryLocation(0x42)),
            Instruction::Halt,
        ];

        let code = instructions
            .into_iter()
            .enumerate()
            .map(|(idx, inst)| (idx as u8, inst))
            .collect::<HashMap<u8, Instruction>>();

        let memory_init: HashMap<u8, u8> =
            HashMap::from_iter(vec![(0x40, 0x20), (0x41, 0x45)]);

        let program = Program {
            entry_point: 0,
            code,
            memory_init,
        };

        let expected = (0x42, 0x65);

        let simulation = PreflightSimulation::simulate(&program);
        assert!(simulation.is_ok());
        let simulation = simulation.unwrap();

        assert_eq!(
            simulation.trace_rows[simulation
                .trace_rows
                .len()
                - 1]
            .get_memory_at(&expected.0)
            .unwrap(),
            expected.1
        );
    }

    #[test]
    /// Tests whether execution stops on reaching `MAX_CPU_CYCLES_ALLOWED`
    fn test_max_cpu_cycles() {
        let instructions = vec![
            Instruction::Jz(Register::R0, InstructionLocation(0x00)),
            Instruction::Halt,
        ];

        let code = instructions
            .into_iter()
            .enumerate()
            .map(|(idx, inst)| (idx as u8, inst))
            .collect::<HashMap<u8, Instruction>>();

        let program = Program {
            entry_point: 0,
            code,
            ..Default::default()
        };

        let simulation = PreflightSimulation::simulate(&program);
        assert!(simulation.is_err());
    }

    #[test]
    /// Tests whether execution halts
    fn test_haltable() {
        let instructions = vec![
            Instruction::Lb(Register::R0, MemoryLocation(0x40)),
            Instruction::Lb(Register::R1, MemoryLocation(0x41)),
            Instruction::Sub(Register::R0, Register::R1),
            Instruction::Jnz(Register::R0, InstructionLocation(0x02)),
            Instruction::Halt,
        ];

        let code = instructions
            .into_iter()
            .enumerate()
            .map(|(idx, inst)| (idx as u8, inst))
            .collect::<HashMap<u8, Instruction>>();

        let memory_init: HashMap<u8, u8> =
            HashMap::from_iter(vec![(0x40, 0x05), (0x41, 0x01)]);

        let program = Program {
            entry_point: 0,
            code,
            memory_init,
        };

        let simulation = PreflightSimulation::simulate(&program);
        assert!(simulation.is_ok());
    }
}
