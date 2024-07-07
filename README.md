# PixieZKVM
A toy, simple an playful ZERO-KNOWLEDGE / STARK based ZKVM to showcase the intricacies of | Trace Generation | Cross-Table Lookups | RangeChecks | Proof Composition.

## VM description
- [ ] Single byte-only memory addressible memory of range: `0x0000` to `0xFFFF`.
- [ ] No Stack, No Heap, No Allocator.
- [ ] CPU instruction set described below, 3 registers: `r1`, `r2`, `r3` each capable of storing one byte at a time.
- [ ] Harvard CPU architecture, no self-modifying code.

## Instruction Set
This VM intends to support the smallest subset of instructions to describe the
major design elements of ZKVMs with as less of fluff as possible. For this, the
following instructions are chosen to be implemented:

- [ ] *ADD*: `ADD r1 r2` Adds registers `r1` and `r2` such that `r1 = r1 + r2`.
- [ ] *SUB*: `SUB r1 r2` Subtracts registers `r1` and `r2` such that `r1 = r1 - r2`.
- [ ] *MUL*: `MUL r1 r2` Multiplies registers `r1` and `r2` such that `r1 = r1 * r2`.
- [ ] *DIV*: `DIV r1 r2` Divides registers `r1` and `r2` such that `r1 = r1 / r2`.
- [ ] *BSL*: `BSL r1 r2` BitShifts `r1` by `r2` to the left, panics if `r2 >= 8`. `r1 = r1 << r2`.
- [ ] *BSR*: `BSR r1 r2` BitShift analog towards the right.
- [ ] *LB*: `LB r1 0x1000` Loads a single byte at `0x1000` into register `r1`.
- [ ] *SB*: `SB r1 0x1000` Stores a single byte in register `r1` to memory location `0x1000`.

## Program Writing
Since our instruction set and VM description is bespoke, we do not have compilation
toolkit from any programming language for PixieZKVM. All programs are built by
hand in assembly format.

## Testing the Project
Clone and test:
```
git clone git@github.com:supragya/PixieZKVM.git
cargo test
```
