//! A very simplistic interner based around giving out (wrapped) references
//! rather than some placeholder symbol. This means that e.g. strings can be interned in systems
//! based around `&str` without rewriting to support a new `Symbol` type.
//!
//! The typical use case for something like this is text processing chunks, where chunks are very
//! likely to be repeated. For example, when parsing source code, identifiers are likely to come up
//! multiple times. Rather than have a `Token::Identifier(String)` and allocate every occurrence of
//! those identifiers separately, interners allow you to store `Token::Identifier(Symbol)`, and
//! compare identifier equality by the interned symbol.
//!
//! This crate exists to give the option of using a simplistic interface. If you want or need
//! further power, there are multiple other options available on crates.io.

#![forbid(missing_debug_implementations, unconditional_recursion, future_incompatible)]
#![deny(bad_style, unsafe_code, missing_docs)]

#[macro_use]
#[cfg(feature = "serde")]
extern crate serde;

#[cfg(feature = "parking_lot")]
extern crate parking_lot;
mod parking_lot_shim;

#[cfg(all(feature = "serde", feature = "parking_lot"))]
compile_error!(
    "At this time, parking_lot does not implement De/Serialize.\n\
     As such, simple-interner cannot implement De/Serialize when using parking_lot."
);

mod interner;
pub use interner::Interner;

mod interned;
pub use interned::Interned;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_usage() {
        // Create the interner
        let interner = Interner::default();

        // Intern some strings
        let a1 = interner.get_or_insert(Box::<str>::from("a"));
        let b1 = interner.get_or_insert(String::from("b"));
        let c1 = interner.get_or_insert("c");

        // Get the interned strings
        let a2 = interner.get_or_insert("a");
        let b2 = interner.get_or_insert("b");
        let c2 = interner.get_or_insert("c");

        let a3 = interner.get("a").unwrap();
        let b3 = interner.get("b").unwrap();
        let c3 = interner.get("c").unwrap();

        // The same strings better be the same pointers or it's broken
        assert_eq!(a1.as_ptr(), a2.as_ptr());
        assert_eq!(a2.as_ptr(), a3.as_ptr());
        assert_eq!(b1.as_ptr(), b2.as_ptr());
        assert_eq!(b2.as_ptr(), b3.as_ptr());
        assert_eq!(c1.as_ptr(), c2.as_ptr());
        assert_eq!(c2.as_ptr(), c3.as_ptr());
    }

    #[test]
    fn slice_usage() {
        // Create the interner
        let interner = Interner::default();

        // Intern some strings
        let a1 = interner.get_or_insert(Box::<[u8]>::from(&[0][..]));
        let b1 = interner.get_or_insert(&[1][..]);
        let c1 = interner.get_or_insert(&[2][..]);

        // Get the interned strings
        let a2 = interner.get_or_insert(&[0][..]);
        let b2 = interner.get_or_insert(&[1][..]);
        let c2 = interner.get_or_insert(&[2][..]);

        let a3 = interner.get(&[0]).unwrap();
        let b3 = interner.get(&[1]).unwrap();
        let c3 = interner.get(&[2]).unwrap();

        // They better be the same or it's broken
        assert_eq!(a1, a2);
        assert_eq!(a2, a3);
        assert_eq!(b1, b2);
        assert_eq!(b2, b3);
        assert_eq!(c1, c2);
        assert_eq!(c2, c3);
    }
}
