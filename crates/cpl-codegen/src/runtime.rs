//! The C++ the emitted program leans on: input, output, and the few
//! operations C++ spells differently. Each piece is emitted only when the
//! program uses it, so a small program is a small file.

mod fns;
mod io;
mod lists;
mod maps;
mod num;
mod opt;
mod ranges;
pub(crate) mod shake;
mod strings;

/// One piece of the runtime: its name, the pieces it needs, and its code.
pub(crate) struct Piece {
    pub name: &'static str,
    pub needs: &'static [&'static str],
    pub code: &'static str,
}

/// Every piece, in the order they are emitted: each after the pieces it needs.
pub(crate) fn pieces() -> impl Iterator<Item = &'static Piece> {
    [
        io::PIECES,
        num::PIECES,
        opt::PIECES,
        lists::PIECES,
        ranges::PIECES,
        maps::PIECES,
        strings::PIECES,
        fns::PIECES,
    ]
    .into_iter()
    .flatten()
}
