//! This simulator parses the program and executes it, formulating
//! an `ExecutionTrace` containing all the information of the preflight
//! simulation.
//!
//! This file does not involve itself with any S(N/T)ARK systems

use std::collections::HashMap;

use crate::vm_specs::{
    Instruction,
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

/// Unconstrainted Preflight Simulation of the program built
/// by running the code.
pub struct PreflightSimulation {
    trace_rows: Vec<SimulationRow>,
}
