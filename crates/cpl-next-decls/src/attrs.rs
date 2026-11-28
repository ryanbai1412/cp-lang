//! What each `@name` an item carries means to the declarations (R11.6). The vocabulary is
//! `cpl_next_syntax::grammar::Decorator`, the one table the parsers share; a role a declaration
//! fills is `@role(..)` (`Role`). The name is read once, at the `Decorator` node; past it decls
//! keeps the `Decorator`, never the text.

pub use cpl_next_syntax::grammar::Decorator;

/// What a decorator sits on, for the rules of where each may go and what it takes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeclKind {
    Fn { params: usize },
    Struct,
    Enum,
    Variant,
    Field,
    Trait,
    Impl,
    Alias,
    Const,
    Domain,
    Context,
    Use,
}
