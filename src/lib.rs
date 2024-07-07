//! We enable a few `nightly`-only features since by depending on
//! `plonky2`, we anyways need to use `nightly` toolchain. Since,
//! among other things `plonky2` enables `#![feature(specialization)]`.
//! We tend to not overuse these features in this crate however :).

// We enable `variant_count` since we want to access
// `std::mem::variant_count::<T>` which for any enum `T`
// produces the number of variants withing the enum.
// Take a look at `vm_spec.rs` for `REGISTER_COUNT`.
#![feature(variant_count)]

mod preflight_simulator;
mod vm_specs;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        preflight_simulator::PreflightSimulation,
        vm_specs::{
            Instruction,
            MemoryLocation,
            Program,
            Register,
        },
    };

    #[test]
    /// Tests whether two numbers in memory can be added together
    /// in the ZKVM
    fn test_preflight_1() {
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

        println!("{:#?}", simulation);
    }
}
