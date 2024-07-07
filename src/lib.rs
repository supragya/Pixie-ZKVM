mod preflight_simulator;
mod vm_specs;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::vm_specs::{
        Instruction,
        MemoryLocation,
        Register,
    };

    #[test]
    fn test_preflight_1() {
        // Tests that if two numbers can be added in the VM
        let instructions = vec![
            Instruction::Lb(Register::R1, MemoryLocation(0x40)),
            Instruction::Lb(Register::R2, MemoryLocation(0x41)),
            Instruction::Add(Register::R1, Register::R2),
            Instruction::Sb(Register::R1, MemoryLocation(0x42)),
            Instruction::Halt,
        ];

        let memory_init: HashMap<u8, u8> =
            HashMap::from_iter(vec![(0x40, 0x20), (0x41, 0x45)].into_iter());
        let expected = (0x42, 0x65);
    }
}
