//! The grammar's spellings: every keyword and foreign keyword the lexer
//! knows by its text, in one table, so no other code compares source text
//! against a fixed word.

use crate::token::TokenKind::{self, *};

/// Each keyword and the token it lexes to.
pub const KEYWORD_KINDS: &[(&str, TokenKind)] = &[
    ("let", Let),
    ("var", Var),
    ("fn", Fn),
    ("struct", Struct),
    ("enum", Enum),
    ("domain", Domain),
    ("type", Type),
    ("const", Const),
    ("trait", Trait),
    ("impl", Impl),
    ("use", Use),
    ("pub", Pub),
    ("if", If),
    ("elif", Elif),
    ("else", Else),
    ("for", For),
    ("while", While),
    ("in", In),
    ("match", Match),
    ("return", Return),
    ("break", Break),
    ("continue", Continue),
    ("and", And),
    ("or", Or),
    ("not", Not),
    ("assert", Assert),
    ("mut", Mut),
    ("self", SelfKw),
    ("true", True),
    ("false", False),
    ("lends", Lends),
    ("yield", Yield),
    ("unchecked", Unchecked),
];

/// What to write instead of a word that v2.1 reserved and v2.2 does not.
/// The word itself is an ordinary name now.
pub const REMOVED_KEYWORDS: &[(&str, &str)] = &[
    (
        "defer",
        "undo by hand after the call: `cur.push(x); dfs(); cur.pop()`",
    ),
    ("assume", "write `assert`, which is always checked"),
    (
        "where",
        "write the bound in the angle brackets: `fn f<T: Ord>(...)`",
    ),
    (
        "inline",
        "drop it: closures and `lends` bodies are always inlined",
    ),
    (
        "lend",
        "end the `lends` body with the place itself: `mut self.data[i]`",
    ),
    ("law", "write `@law name(args) = condition`"),
];

/// Another language's keyword starting a line, and the keyword it is read
/// as so the uses parse with no cascade: `def f(` is `fn f(`.
pub const FOREIGN_KEYWORDS: &[(&str, TokenKind)] = &[
    ("def", Fn),
    ("func", Fn),
    ("function", Fn),
    ("import", Use),
    ("switch", Match),
];

/// A word that is an ordinary name except in one place, where the parser
/// reads it as grammar: `lo..hi by 2`, `x is Some(v)`, `best max= x`, `_`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Contextual {
    By,
    Is,
    Max,
    Min,
    /// `_`, the pattern that binds nothing.
    Wildcard,
    /// `repeat k:` runs its block `k` times.
    Repeat,
    /// `with ctx(..):` runs its block in a context.
    With,
    Mod,
    Extern,
    Where,
    /// `-> yields T`: a generator.
    Yields,
    /// A trait's or impl's `law`; elsewhere `law` is a removed keyword.
    Law,
}

/// Each contextual word and how it is spelled.
pub const CONTEXTUAL: &[(&str, Contextual)] = &[
    ("by", Contextual::By),
    ("is", Contextual::Is),
    ("max", Contextual::Max),
    ("min", Contextual::Min),
    ("_", Contextual::Wildcard),
    ("repeat", Contextual::Repeat),
    ("with", Contextual::With),
    ("mod", Contextual::Mod),
    ("extern", Contextual::Extern),
    ("where", Contextual::Where),
    ("yields", Contextual::Yields),
    ("law", Contextual::Law),
];

/// The contextual word `text` spells, if any.
pub fn contextual(text: &str) -> Option<Contextual> {
    CONTEXTUAL.iter().find(|(w, _)| *w == text).map(|(_, c)| *c)
}

/// The facts a doc comment states, each on its own `label: text` line, in the
/// order std/README.md lists them.
pub const DOC_FACTS: [&str; 5] = ["cost", "traps", "empty", "ties", "order"];
