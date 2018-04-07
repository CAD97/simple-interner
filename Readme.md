# simple-interner

A very simplistic interner based around giving out (wrapped) references rather
than some placeholder symbol. This means that e.g. strings can be interned in
systems based around `&str` without rewriting to support a new `Symbol` type.

The typical use case for something like this is text processing chunks, where
chunks are very likely to be repeated. For example, when parsing source code,
identifiers are likely to come up multiple times. Rather than have a
`Token::Identifier(String)` and allocate every occurrence of those identifiers
separately, interners allow you to store `Token::Identifier(Symbol)`,
and compare identifier equality by the interned symbol.

This crate exists to give the option of using a simplistic interface. If you
want or need further power, there are multiple other options available on
crates.io.
