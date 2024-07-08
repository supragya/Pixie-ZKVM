use anyhow::Result;
use plonky2::{
    field::{
        extension::Extendable,
        goldilocks_field::GoldilocksField,
        polynomial::PolynomialValues,
    },
    fri::oracle::PolynomialBatch,
    hash::{
        hash_types::RichField,
        merkle_tree::MerkleCap,
    },
    iop::challenger::Challenger,
    plonk::config::{
        AlgebraicHasher,
        GenericConfig,
        Hasher,
        PoseidonGoldilocksConfig,
    },
    util::timing::TimingTree,
};
use starky::{
    config::StarkConfig,
    proof::StarkProofWithPublicInputs,
};

use crate::{
    preflight_simulator::PreflightSimulation,
    stark_cpu::CPUStark,
    stark_memory::MemoryStark,
    stark_program_instructions::ProgramInstructionsStark,
    vm_specs::Program,
};

/// STARK Gadgets of Pixie ZKVM
///
/// ## Generics
/// `F`: The [Field] that the STARK is defined over (Goldilock's)
/// `D`: Extension Degree on `F`, generally `2`. Extension: (a + sqrt(7)b)
pub struct PixieZKVM<F, const D: usize>
where
    F: RichField + Extendable<D>,
{
    pub program_instructions: ProgramInstructionsStark<F, D>,
    pub cpu: CPUStark<F, D>,
    pub memory: MemoryStark<F, D>,
}

pub fn trace_to_merkle_caps<F, C, const D: usize>(
    config: &StarkConfig,
    trace: &Vec<PolynomialValues<F>>,
) -> MerkleCap<F, C::Hasher>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let mut timing_tree = TimingTree::default();
    PolynomialBatch::<F, C, D>::from_values(
        trace.to_owned(),
        config
            .fri_config
            .rate_bits,
        false,
        config
            .fri_config
            .cap_height,
        &mut timing_tree,
        None,
    )
    .merkle_tree
    .cap
}

pub fn generate_proof<F, C, const D: usize>(prog: &Program) -> Result<()>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    //type PR = StarkProofWithPublicInputs<GoldilocksField, C, D>;

    let mut config = StarkConfig::standard_fast_config();
    // Need to do this since our table can be small.
    config
        .fri_config
        .cap_height = 1;

    // Do a simulation
    let simulation = PreflightSimulation::simulate(prog)?;

    // Generate traces and commit to them
    let pi_trace = ProgramInstructionsStark::<F, D>::generate_trace(prog);
    let pi_comm_cap = trace_to_merkle_caps::<F, C, D>(&config, &pi_trace);
    let cpu_trace = CPUStark::<F, D>::generate_trace(&simulation);
    let cpu_comm_cap = trace_to_merkle_caps::<F, C, D>(&config, &cpu_trace);
    let mem_trace = MemoryStark::<F, D>::generate_trace(&simulation);
    let mem_comm_cap = trace_to_merkle_caps::<F, C, D>(&config, &mem_trace);

    // Create a new IOP challenger and let it observe all the commitments
    // This is Fiat-Shamir!
    // This challenger needs to be reproduced at the verifier's end as well
    // so make sure all the inputs are available to the verifier. We are putting
    // in commitments and not the traces directly as the latter is not available
    // in full to the verifier
    let mut iop_challenger = Challenger::<F, C::Hasher>::new();
    iop_challenger.observe_cap(&pi_comm_cap);
    iop_challenger.observe_cap(&cpu_comm_cap);
    iop_challenger.observe_cap(&mem_comm_cap);

    // Get `config.num_challenges` number of grand product challenge points
    // Each grand product challenge requires two elements in `F`: `beta` and
    // `gamma`. Hence, `2 * config.num_challenges` sampled
    let grand_product_challenges =
        iop_challenger.get_n_challenges(2 * config.num_challenges);

    Ok(())
}
