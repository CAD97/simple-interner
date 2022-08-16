//! A very simplistic interner based around giving out references rather than
//! some placeholder symbol. This means that you can mostly transparently add
//! interning into a system without requiring rewriting all of the code to work
//! on a new `Symbol` type, asking the interener to concretize the symbols.
//!
//! The typical use case for something like this is text processing chunks,
//! where chunks are very likely to be repeated. For example, when parsing
//! source code, identifiers are likely to come up multiple times. Rather than
//! have a `String` allocated for every occurrence of the identifier separately,
//! interners allow you to store `Symbol`. This additionally allows comparing
//! symbols to be much quicker than comparing the full interned string.
//!
//! This crate exists to give the option of using the simplest interface. For
//! a more featureful interner, consider using a different crate, such as
//!
//! |             crate |    global   |  local | `'static` opt[^1] | non-`str` |  symbol size  | symbols deref |
//! | ----------------: | :---------: | :----: | :---------------: | :-------: | :-----------: | :-----------: |
//! |   simple-interner |  manual[^2] |   yes  |         no        |    yes    |     `&T`      |      yes      |
//! |        [intaglio] |     no      |   yes  |         yes       |    no     |     `u32`     |      no       |
//! |      [internment] |    rc[^3]   |   yes  |         no        |    yes    |     `&T`      |      yes      |
//! |           [lasso] |     no      |   yes  |         yes       |    no     | `u8`–`usize`  |      no       |
//! | [string-interner] |     no      |   yes  |     optionally    |    no     | `u16`–`usize` |      no       |
//! |    [string_cache] | static only | rc[^3] |     buildscript   |    no     |     `u64`     |      yes      |
//! |    [symbol_table] |     yes     |   yes  |         no        |    no     |     `u32`     |  global only  |
//! |            [ustr] |     yes     |   no   |         no        |    no     |    `usize`    |      yes      |
//!
//! (PRs to this table are welcome!) <!-- crate must have seen activity in the last year -->
//!
//! [^1]: The interner stores `&'static` references without copying the pointee
//!       into the store, e.g. storing `Cow<'static, str>` instead of `Box<str>`.
//!
//! [^2]: At the moment, creating the [`Interner`] inside a `static`, using
//!       [`Interner::with_hasher`], requires the `hashbrown` feature to
//!       be enabled.
//!
//! [^3]: Uses reference counting to collect globally unused symbols.
//!
//! [intaglio]: https://lib.rs/crates/intaglio
//! [lasso]: https://lib.rs/crates/lasso
//! [internment]: https://lib.rs/crates/internment
//! [string-interner]: https://lib.rs/crates/string-interner
//! [string_cache]: https://lib.rs/crates/string_cache
//! [symbol_table]: https://lib.rs/crates/symbol_table
//! [ustr]: https://lib.rs/crates/ustr

#![forbid(unconditional_recursion, future_incompatible)]
#![warn(unsafe_code, bad_style, missing_docs, missing_debug_implementations)]

#[cfg(feature = "parking_lot")]
mod parking_lot_shim;

mod interner;
pub use interner::Interner;

mod interned;
pub use interned::Interned;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_usage() {
        // Create the interner
        let interner = Interner::new();

        // Intern some strings
        let a1 = interner.intern(Box::<str>::from("a"));
        let b1 = interner.intern(String::from("b"));
        let c1 = interner.intern("c");

        // Get the interned strings
        let a2 = interner.intern("a");
        let b2 = interner.intern("b");
        let c2 = interner.intern("c");

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
        let interner = Interner::new();

        // Intern some strings
        let a1 = interner.intern(Box::<[u8]>::from([0]));
        let b1 = interner.intern(Vec::from([1]));
        let c1 = interner.intern::<[u8; 1]>([2]);
        let d1 = interner.intern::<&[u8]>(&[3][..]);

        // Get the interned strings
        let a2 = interner.intern([0]);
        let b2 = interner.intern([1]);
        let c2 = interner.intern([2]);
        let d2 = interner.intern([3]);

        let a3 = interner.get(&[0][..]).unwrap();
        let b3 = interner.get(&[1]).unwrap();
        let c3 = interner.get(&[2]).unwrap();
        let d3 = interner.get(&[3]).unwrap();

        // They better be the same or it's broken
        assert_eq!(a1.as_ptr(), a2.as_ptr());
        assert_eq!(a2.as_ptr(), a3.as_ptr());
        assert_eq!(b1.as_ptr(), b2.as_ptr());
        assert_eq!(b2.as_ptr(), b3.as_ptr());
        assert_eq!(c1.as_ptr(), c2.as_ptr());
        assert_eq!(c2.as_ptr(), c3.as_ptr());
        assert_eq!(d1.as_ptr(), d2.as_ptr());
        assert_eq!(d2.as_ptr(), d3.as_ptr());
    }

    #[cfg(feature = "hashbrown")]
    #[test]
    fn static_interner() {
        use hash32::{BuildHasherDefault, FnvHasher};

        static INTERNER: Interner<str, BuildHasherDefault<FnvHasher>> =
            Interner::with_hasher(BuildHasherDefault::new());

        let non_static_str = String::from("a");

        let interned = INTERNER.intern(non_static_str);

        let static_str: &'static str = Interned::get(&interned);

        assert_eq!(static_str, "a");
    }
}
