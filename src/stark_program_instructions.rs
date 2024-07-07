//! This file is an encoding of all the "Program". It is "static"
//! part of the proof generation process, in the sense that the "program"
//! a.k.a. resting code is known prior to proof generation. This
//! needs to be differentiated from actual running process trace, since
//! that may be longer than "program" owing to actual execution of jumps.

use core::marker::PhantomData;
use plonky2::{
    field::{
        extension::{
            Extendable,
            FieldExtension,
        },
        packed::PackedField,
    },
    hash::hash_types::RichField,
    iop::ext_target::ExtensionTarget,
};
use starky::{
    constraint_consumer::ConstraintConsumer,
    evaluation_frame::StarkFrame,
    stark::Stark,
};

pub struct ProgramInstructions<T> {
    pub program_counter: T,
    pub instruction_data: T,
}

const NUMBER_OF_COLS: usize = 2;
const PUBLIC_INPUTS: usize = 0;

pub struct ProgramInstructionsStark<F, const D: usize> {
    pub _f: PhantomData<F>,
}

impl<F, const D: usize> Stark<F, D> for ProgramInstructionsStark<F, D>
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
        builder: &mut plonky2::plonk::circuit_builder::CircuitBuilder<F, D>,
        vars: &Self::EvaluationFrameTarget,
        yield_constr: &mut starky::constraint_consumer::RecursiveConstraintConsumer<F, D>,
    ) {
    }

    fn constraint_degree(&self) -> usize {
        3
    }
}
