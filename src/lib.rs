//! We enable a few `nightly`-only features since by depending on
//! `plonky2`, we anyways need to use `nightly` toolchain. Since,
//! among other things `plonky2` enables `#![feature(specialization)]`.
//! We tend to not overuse these features in this crate however :).

// We enable `variant_count` since we want to access
// `std::mem::variant_count::<T>` which for any enum `T`
// produces the number of variants withing the enum.
// Take a look at `vm_spec.rs` for `REGISTER_COUNT`.
#![feature(variant_count)]

// We allow for dead_code because a usage of such in test harnesses
// doesn't register as a usage for clippy
#[allow(dead_code)]
mod preflight_simulator;
#[allow(dead_code)]
mod utilities;
#[allow(dead_code)]
mod vm_specs;

// STARK tables -------------
#[allow(dead_code)]
mod stark_cpu;
#[allow(dead_code)]
mod stark_program_instructions;

#[allow(dead_code)]
mod stark_memory;
//mod stark_rangecheck_u8;
//mod stark_execution_program_subset;

// END TO END TEST ----------
#[allow(dead_code)]
mod e2e_tests;
