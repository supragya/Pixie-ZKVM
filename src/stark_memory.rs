//! This file is an encoding of all the execution seen at the "Memory"
//! It is dynamic and changes depending on the memory_init. It has
//! to be linked to the execution stark "CPU" by having a cross-table
//! -lookup with `MemoryStark`.

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

use crate::{
    preflight_simulator::PreflightSimulation,
    vm_specs::{
        Instruction,
        Program,
    },
};

// Table description:
// +---------------+-------+-------+-------+-------+---------+-------------+
// | MemoryAddress | Clock | Value | Is_LB | Is_SB | Is_Init | Is_Executed |
// +---------------+-------+-------+-------+-------+---------+-------------+
// |  ...          |  ...  |  ...  |  ...  |  ...  |   ...   |  ...        |
// +---------------+-------+-------+-------+-------+---------+-------------+
//
const NUMBER_OF_COLS: usize = 7;
const PUBLIC_INPUTS: usize = 0;

#[derive(Clone, Copy)]
pub struct MemoryStark<F, const D: usize> {
    pub _f: PhantomData<F>,
}

impl<F, const D: usize> MemoryStark<F, D>
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
        let mut trace: Vec<[F; NUMBER_OF_COLS]> = sim
            .memory_init
            .iter()
            .map(|(addr, value)| {
                [
                    // Memory Address
                    F::from_canonical_u8(*addr),
                    // Clock
                    F::ZERO,
                    // Value
                    F::from_canonical_u8(*value),
                    // Is_LB and Is_SB
                    F::ZERO,
                    F::ZERO,
                    // Is_Init
                    F::ONE,
                    // Is_Executed
                    F::ONE,
                ]
            })
            .collect();

        sim.trace_rows
            .iter()
            .for_each(|row| {
                let (mut is_lb, mut is_sb, mut addr) = (false, false, 0);
                match row.instruction {
                    Instruction::Lb(_, memloc) => {
                        is_lb = true;
                        addr = memloc.0;
                    }
                    Instruction::Sb(_, memloc) => {
                        is_sb = true;
                        addr = memloc.0;
                    }
                    _ => {
                        return ();
                    }
                }
                let value = row
                    .memory_snapshot
                    .get(&addr)
                    .expect("execution trace should have value for memop");
                trace.push([
                    // Memory Addrss
                    F::from_canonical_u8(addr),
                    // Clock
                    F::from_canonical_u32(row.clock),
                    // Value
                    F::from_canonical_u8(*value),
                    // Is_LB
                    F::from_canonical_u8(u8::from(is_lb)),
                    // Is_SB
                    F::from_canonical_u8(u8::from(is_sb)),
                    // Is_Init
                    F::ZERO,
                    // Is_Executed
                    F::ONE,
                ]);
            });
        // Need to pad the trace to a len of some power of 2
        let pow2_len = trace
            .len()
            .next_power_of_two();
        trace.resize(pow2_len, [F::ZERO; NUMBER_OF_COLS]);

        // Convert into polynomial values
        trace_rows_to_poly_values(trace)
    }
}

impl<F, const D: usize> Stark<F, D> for MemoryStark<F, D>
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
        type S = MemoryStark<F, D>;
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
        let trace = MemoryStark::<F, D>::generate_trace(&simulation);
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
