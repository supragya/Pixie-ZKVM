#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        plonk::config::{
            GenericConfig,
            PoseidonGoldilocksConfig,
        },
        util::timing::TimingTree,
    };
    use starky::{
        config::StarkConfig,
        proof::StarkProofWithPublicInputs,
        prover::prove,
        verifier::verify_stark_proof,
    };

    use crate::{
        stark_program_instructions::ProgramInstructionsStark,
        vm_specs::{
            Instruction,
            MemoryLocation,
            Program,
            Register,
        },
    };

    #[test]
    fn test_add_program() {
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

        // Generate the static part of the proof
        let program_proof = {
            const D: usize = 2;
            type C = PoseidonGoldilocksConfig;
            type F = <C as GenericConfig<D>>::F;
            type S = ProgramInstructionsStark<F, D>;
            type PR = StarkProofWithPublicInputs<GoldilocksField, C, 2>;

            let stark = S::new();
            let mut config = StarkConfig::standard_fast_config();
            // Need to do this since our table is small. Need atleast 1<<5
            // sized table to not affect this
            config
                .fri_config
                .cap_height = 1;
            let program = Program::default();
            let trace =
                ProgramInstructionsStark::<F, D>::generate_trace(&program);
            let proof: Result<PR, anyhow::Error> = prove(
                stark.clone(),
                &config,
                trace,
                &[],
                &mut TimingTree::default(),
            );
            assert!(proof.is_ok());
            let proof = proof.unwrap();
            let verification =
                verify_stark_proof(stark, proof.clone(), &config);
            assert!(verification.is_ok());
            proof
        };
    }
}
