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
    evaluation_frame::StarkFrame,
    stark::Stark,
    util::trace_rows_to_poly_values,
};

use crate::{
    preflight_simulator::PreflightSimulation,
    utilities::debug_table,
    vm_specs::Instruction,
};

// Table description:
// +-----+----+--------+--------+--------------+---------+-------------+
// | Clk | PC | Reg R0 | Reg R1 | Location     | Opcode* | Is_Executed |
// +-----+----+--------+--------+--------------+---------+-------------+
// | ..  | .. | ...    | ...    |  ....        |  ...    |             |
// +-----+----+--------+--------+--------------+---------+-------------+
//
// `Opcode*` means `Opcode` that is one-hot encoded
// `Location` can be either Memory or Instruction location.
// 5 Columns for `Clk`, `PC`, `Reg R0`, `Reg R1`, `Location`
// 11 Columns for opcodes. See `Instruction::get_opcode`.
// 1 Column for `Is_Executed`
const NUM_DYNAMIC_COLS: usize = 5;
const NUM_OPCODE_ONEHOT: usize = 11;
const NUMBER_OF_COLS: usize = NUM_DYNAMIC_COLS + NUM_OPCODE_ONEHOT + 1;
const ROW_HEADINGS: [&str; NUMBER_OF_COLS] = [
    "clk", "pc", "r0", "r1", "loc", "op_add", "op_sub", "op_mul", "op_div",
    "op_shl", "op_shr", "op_jz", "op_jnz", "op_lb", "op_sb", "op_halt",
    "is_exec",
];
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

    pub fn generate_trace(sim: &PreflightSimulation) -> Vec<PolynomialValues<F>>
    where
        F: RichField,
    {
        let mut trace = sim
            .trace_rows
            .iter()
            .map(|row| {
                let dynamic_elems = [
                    // Clock
                    F::from_canonical_u32(row.clock),
                    // Program Counter
                    F::from_canonical_u8(row.program_counter),
                    // Registers
                    F::from_canonical_u8(row.registers[0]),
                    F::from_canonical_u8(row.registers[1]),
                    // Memory Address (if any accessed)
                    F::from_canonical_u8(match row.instruction {
                        Instruction::Jz(_, l) => l.0,
                        Instruction::Jnz(_, l) => l.0,
                        Instruction::Lb(_, l) => l.0,
                        Instruction::Sb(_, l) => l.0,
                        _ => 0,
                    }),
                ];
                let opcode_one_hot = row
                    .instruction
                    .one_hot_encode_and_apply::<F>();

                let mut table_row = [F::ZERO; NUMBER_OF_COLS];
                let mut idx = 0;
                for elem in dynamic_elems {
                    table_row[idx] = elem;
                    idx += 1;
                }
                for elem in opcode_one_hot {
                    table_row[idx] = elem;
                    idx += 1;
                }
                // `Is_Executed`
                table_row[NUMBER_OF_COLS - 1] = F::ONE;

                table_row
            })
            .collect::<Vec<[F; NUMBER_OF_COLS]>>();

        debug_table("CPU", ROW_HEADINGS, &trace);

        // Need to pad the trace to a len of some power of 2
        let pow2_len = trace
            .len()
            .next_power_of_two();
        trace.resize(pow2_len, [F::ZERO; NUMBER_OF_COLS]);

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

    use crate::vm_specs::Program;

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
        let simulation = PreflightSimulation::simulate(&program);
        assert!(simulation.is_ok());
        let simulation = simulation.unwrap();
        let trace = CPUStark::<F, D>::generate_trace(&simulation);
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
