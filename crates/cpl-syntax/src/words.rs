//! The contextual words the old parser reads as grammar, a copy of
//! `cpl_next_syntax::grammar`'s, so the old parser doesn't depend on the new
//! one. It goes with the old parser at the switch.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Word {
    Loop,
    Lazy,
}

const WORDS: &[(&str, Word)] = &[("loop", Word::Loop), ("lazy", Word::Lazy)];

/// The contextual word `text` spells, if any.
pub(crate) fn word(text: &str) -> Option<Word> {
    WORDS.iter().find(|(w, _)| *w == text).map(|(_, c)| *c)
}
