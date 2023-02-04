# simple-interner

<!-- cargo-rdme start -->

A very simplistic interner based around giving out references rather than
some placeholder symbol. This means that you can mostly transparently add
interning into a system without requiring rewriting all of the code to work
on a new `Symbol` type, asking the interener to concretize the symbols.

The typical use case for something like this is text processing chunks,
where chunks are very likely to be repeated. For example, when parsing
source code, identifiers are likely to come up multiple times. Rather than
have a `String` allocated for every occurrence of the identifier separately,
interners allow you to store `Symbol`. This additionally allows comparing
symbols to be much quicker than comparing the full interned string.

This crate exists to give the option of using the simplest interface. For
a more featureful interner, consider using a different crate, such as

|             crate |    global   |  local | `'static` opt[^1] | `str`-only |  symbol size  | symbols deref |
| ----------------: | :---------: | :----: | :---------------: | :--------: | :-----------: | :-----------: |
|   simple-interner |  manual[^2] |   yes  |         no        |     no     |     `&T`      |      yes      |
|        [intaglio] |     no      |   yes  |         yes       |     yes    |     `u32`     |      no       |
|      [internment] |    rc[^3]   |   yes  |         no        |     no     |     `&T`      |      yes      |
|           [lasso] |     no      |   yes  |         yes       |     yes    | `u8`–`usize`  |      no       |
| [string-interner] |     no      |   yes  |     optionally    |     yes    | `u16`–`usize` |      no       |
|    [string_cache] | static only | rc[^3] |     buildscript   |     yes    |     `u64`     |      yes      |
|    [symbol_table] |     yes     |   yes  |         no        |     yes    |     `u32`     |  global only  |
|            [ustr] |     yes     |   no   |         no        |     yes    |    `usize`    |      yes      |

(PRs to this table are welcome!) <!-- crate must have seen activity in the last year -->

[^1]: The interner stores `&'static` references without copying the pointee
    into the store, e.g. storing `Cow<'static, str>` instead of `Box<str>`.

[^2]: At the moment, creating the `Interner` inside a `static`, using
    `Interner::with_hasher`, requires the `hashbrown` feature to be enabled.

[^3]: Uses reference counting to collect globally unused symbols.

[intaglio]: https://lib.rs/crates/intaglio
[lasso]: https://lib.rs/crates/lasso
[internment]: https://lib.rs/crates/internment
[string-interner]: https://lib.rs/crates/string-interner
[string_cache]: https://lib.rs/crates/string_cache
[symbol_table]: https://lib.rs/crates/symbol_table
[ustr]: https://lib.rs/crates/ustr

<!-- cargo-rdme end -->

