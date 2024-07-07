//! This simulator parses the program and executes it, formulating
//! an `ExecutionTrace` containing all the information of the preflight
//! simulation.
//!
//! This file does not involve itself with any S(N/T)ARK systems

use std::collections::HashMap;

use anyhow::{
    Context,
    Result,
};

use crate::vm_specs::{
    Instruction,
    Program,
    Register,
    REGISTER_COUNT,
};

/// Each `SimulationRow` describes the state of simulation at each step
/// of execution
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
            memory_snapshot: prog
                .memory_init
                .clone(),
        })
    }
}

/// Unconstrainted Preflight Simulation of the program built
/// by running the code.
pub struct PreflightSimulation {
    trace_rows: Vec<SimulationRow>,
}

impl PreflightSimulation {
    /// Entry point to simulate a program and generate a `PreflightSimulation`
    /// to be used to generate tables
    pub fn simulate(prog: &Program) -> Result<Self> {
        let mut current_row = SimulationRow::generate_first_row(prog)?;
        Ok(Self { trace_rows: vec![] })
    }
}
