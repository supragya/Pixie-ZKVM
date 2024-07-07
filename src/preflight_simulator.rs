//! This simulator parses the program and executes it, formulating
//! an `ExecutionTrace` containing all the information of the preflight
//! simulation.
//!
//! This file does not involve itself with any S(N/T)ARK systems

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::vm_specs::{Instruction, Program, REGISTER_COUNT};

/// Each `SimulationRow` describes the state of simulation at each step
/// of execution
#[derive(Debug)]
pub struct SimulationRow {
    /// Encodes the instruction executed during this "row". This would
    /// be useful when we go for SN/TARK constraining.
    instruction: Instruction,

    /// Clock cycle during execution. Supports large cycle count till
    /// `u32::MAX`
    clock: u32,

    /// Address of the program instruction
    program_counter: u8,

    /// Whether at this row the execution halted. Should only be true
    /// for the last row in any `PreflightSimulation`
    is_halted: bool,

    /// Registers
    registers: [u8; REGISTER_COUNT],

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
    memory_snapshot: HashMap<u8, u8>,
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
            clock: 0,
            program_counter,
            is_halted: false,
            registers: [0; REGISTER_COUNT],
            memory_snapshot: prog.memory_init.clone(),
        })
    }

    pub fn execute_one_cycle(&self, prog: &Program) -> Result<Self> {
        // Remember, we have no jump instructions in our VM ISA
        // Hence, this following is safe. Otherwise, more complicated
        // logic needs to be implemented.
        let program_counter = self.program_counter + 1;
        let clock = self.clock + 1;

        let instruction = prog
            .code
            .get(&program_counter)
            .cloned()
            .context("instruction not found")?;

        let is_halted = instruction == Instruction::Halt;
        let mut registers = self.registers;

        let mut memory_snapshot = self.memory_snapshot.clone();

        match self.instruction {
            Instruction::Add(a, b) => registers[usize::from(a)] += registers[usize::from(b)],
            Instruction::Sub(a, b) => registers[usize::from(a)] += registers[usize::from(b)],
            Instruction::Mul(a, b) => registers[usize::from(a)] += registers[usize::from(b)],
            Instruction::Div(a, b) => registers[usize::from(a)] += registers[usize::from(b)],
            Instruction::Bsl(reg, amount) => {
                if registers[usize::from(amount)] >= 8 {
                    return Err(anyhow!("invalid shift amount"));
                }
                registers[usize::from(reg)] <<= registers[usize::from(amount)];
            }
            Instruction::Bsr(reg, amount) => {
                if registers[usize::from(amount)] >= 8 {
                    return Err(anyhow!("invalid shift amount"));
                }
                registers[usize::from(reg)] >>= registers[usize::from(amount)];
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

        Ok(Self {
            instruction,
            clock,
            program_counter,
            is_halted,
            registers,
            memory_snapshot,
        })
    }

    pub fn get_memory_at(&self, address: &u8) -> Option<u8> {
        self.memory_snapshot.get(address).copied()
    }

    pub fn get_registers(&self) -> [u8; REGISTER_COUNT] {
        self.registers
    }
}

/// Unconstrainted Preflight Simulation of the program built
/// by running the code.
#[derive(Debug)]
pub struct PreflightSimulation {
    pub trace_rows: Vec<SimulationRow>,
}

impl PreflightSimulation {
    /// Entry point to simulate a program and generate a `PreflightSimulation`
    /// to be used to generate tables
    pub fn simulate(prog: &Program) -> Result<Self> {
        const MAX_CPU_CYCLES_ALLOWED: usize = 1_000;

        let mut trace_rows = Vec::with_capacity(MAX_CPU_CYCLES_ALLOWED / 4);
        let first_row = SimulationRow::generate_first_row(prog)?;
        trace_rows.push(first_row);

        while trace_rows.len() < MAX_CPU_CYCLES_ALLOWED
            && !trace_rows[trace_rows.len() - 1].is_halted
        {
            let current_row = trace_rows[trace_rows.len() - 1].execute_one_cycle(prog)?;
            trace_rows.push(current_row);
        }

        if !trace_rows[trace_rows.len() - 1].is_halted {
            return Err(anyhow!(
                "simulation halted since MAX_CPU_CYCLES_ALLOWED reached"
            ));
        }

        Ok(Self { trace_rows })
    }
}
