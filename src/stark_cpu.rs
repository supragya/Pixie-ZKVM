//! This file is an encoding of all the execution seen at the "CPU"
//! It is dynamic and changes depending on the memory_init. It has
//! to be linked to the static code "Program" by having a cross-table
//! -lookup with `ProgramInstructionsStark`.

use core::marker::PhantomData;
use plonky2::{
    field::{
        extension::{
            Extendable,
            FieldExtension,
        },
        packed::PackedField,
        polynomial::PolynomialValues,
    },
    hash::hash_types::RichField,
    iop::ext_target::ExtensionTarget,
    plonk::circuit_builder::CircuitBuilder,
};
use starky::{
    constraint_consumer::{
        ConstraintConsumer,
        RecursiveConstraintConsumer,
    },
    evaluation_frame::{
        StarkEvaluationFrame,
        StarkFrame,
    },
    stark::Stark,
    util::trace_rows_to_poly_values,
};

use crate::vm_specs::Program;

// Table description:
// +-----+----+--------+--------+--------------+---------+
// | Clk | PC | Reg R0 | Reg R1 | MemoryAddr   | Opcode* |
// +-----+----+--------+--------+--------------+---------+
// | ..  | .. | ...    | ...    |  ....        |  ...    |
// +-----+----+--------+--------+--------------+---------+
//
// `Opcode*` means `Opcode` that is one-hot encoded
// 5 Columns for `Clk`, `PC`, `Reg R0`, `Reg R1`, `MemoryAccess`
// 10 Columns for opcodes. See `Instruction::get_opcode`.
const NUMBER_OF_COLS: usize = 5 + 10;
const PUBLIC_INPUTS: usize = 0;

#[derive(Clone, Copy)]
pub struct CPUStark<F, const D: usize> {
    pub _f: PhantomData<F>,
}

impl<F, const D: usize> CPUStark<F, D>
where
    F: RichField + Extendable<D>,
{
    pub fn new() -> Self {
        Self { _f: PhantomData }
    }

    pub fn generate_trace(prog: ) -> Vec<PolynomialValues<F>>
    where
        F: RichField,
    {
        let mut trace = prog
            .code
            .iter()
            .map(|(pc, inst)| {
                [
                    // Program Counter (ID = 0)
                    F::from_canonical_u8(*pc),
                    // Instruction Opcode (ID = 1)
                    F::from_canonical_u8(inst.get_opcode()),
                    // Filter, true if actual instructions (ID = 2)
                    F::ONE,
                ]
            })
            .collect::<Vec<[F; NUMBER_OF_COLS]>>();

        // Need to pad the trace to a len of some power of 2
        let pow2_len = trace
            .len()
            .next_power_of_two();
        trace.resize(pow2_len, [F::ZERO, F::ZERO, F::ZERO]);

        // Convert into polynomial values
        trace_rows_to_poly_values(trace)
    }
}

impl<F, const D: usize> Stark<F, D> for CPUStark<F, D>
where
    F: RichField + Extendable<D>,
{
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<P, P::Scalar, NUMBER_OF_COLS, PUBLIC_INPUTS>
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>;
    type EvaluationFrameTarget = StarkFrame<
        ExtensionTarget<D>,
        ExtensionTarget<D>,
        NUMBER_OF_COLS,
        PUBLIC_INPUTS,
    >;

    const COLUMNS: usize = NUMBER_OF_COLS;
    const PUBLIC_INPUTS: usize = PUBLIC_INPUTS;

    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        vars: &Self::EvaluationFrame<FE, P, D2>,
        yield_constr: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
    }

    fn eval_ext_circuit(
        &self,
        _builder: &mut CircuitBuilder<F, D>,
        _vars: &Self::EvaluationFrameTarget,
        _yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    ) {
        unimplemented!()
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}

#[cfg(test)]
mod tests {

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

    use super::*;

    #[test]
    fn test_nil_program() {
        const D: usize = 2;
        type C = PoseidonGoldilocksConfig;
        type F = <C as GenericConfig<D>>::F;
        type S = CPUStark<F, D>;
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
            CPUStark::<F, D>::generate_program_instructions_trace(&program);
        let proof: Result<PR, anyhow::Error> = prove(
            stark.clone(),
            &config,
            trace,
            &[],
            &mut TimingTree::default(),
        );
        assert!(proof.is_ok());
        let verification = verify_stark_proof(stark, proof.unwrap(), &config);
        assert!(verification.is_ok());
    }
}
