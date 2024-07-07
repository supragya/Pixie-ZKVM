//! While macros shouln't really be used in a "toy" implementation,
//! they can help a lot! But they are also cumbersome to use. In the
//! sense that they are hard to read.
//!
//! Macros below are intended to be easy to read, comments are provided
//! where necessary

macro_rules! derive_get_number_of_columns {
    ($structure: ident) => {
        impl<T> $structure<T> {
            pub const fn get_number_of_columns() -> usize {
                // `u8` is guaranteed to have a `size_of` of 1
                std::mem::size_of::<$structure<u8>>();
            }
        }
    };
}
