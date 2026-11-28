//! Tokens.

use cpl_diag::Span;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    /// An interpolating string literal, quotes included.
    Str,
    /// `r"..."`.
    RawStr,
    /// Text of an interpolating string around its holes, quotes included,
    /// and a hole's format after its `:`. Only the parser makes these, when
    /// it splits a `Str` into a `Literal` with `Interp` children.
    StrText,
    Char,
    /// A loop label such as `'outer`.
    Label,

    // Keywords.
    Let,
    Var,
    Fn,
    Struct,
    Enum,
    Domain,
    Type,
    Const,
    Trait,
    Impl,
    Use,
    Pub,
    If,
    Elif,
    Else,
    For,
    While,
    In,
    Match,
    Return,
    Break,
    Continue,
    And,
    Or,
    Not,
    Assert,
    Mut,
    SelfKw,
    True,
    False,
    Lends,
    Yield,
    Unchecked,

    // Delimiters.
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Punctuation and operators.
    Comma,
    Colon,
    ColonColon,
    Semi,
    Dot,
    DotDot,
    DotDotEq,
    Arrow,
    FatArrow,
    At,
    Question,
    QuestionQuestion,
    Eq,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    /// `<=>`: three-way comparison.
    Spaceship,
    Gt,
    GtEq,
    Plus,
    Minus,
    Star,
    StarStar,
    Slash,
    Percent,
    Caret,
    Amp,
    Pipe,
    Tilde,
    /// Postfix `!`: unwrap an optional.
    Bang,
    Shl,
    Shr,
    /// `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `^=`, `&=`, `|=`, `<<=`, `>>=`.
    OpEq,

    // Layout.
    Newline,
    Indent,
    Dedent,
    Eof,

    /// A character the lexer could not read. Already reported.
    Error,

    // Trivia: the bytes between tokens, so the token list is the whole file.
    /// Spaces, tabs, `\r` and `\n`.
    Whitespace,
    /// A `//` comment, up to its line's end.
    Comment,
    /// A line the lexer skipped after reporting it: another language's
    /// header, such as `#include <bits/stdc++.h>`.
    Skipped,
}

/// The 33 reserved words (R01.3): 29 for contestants (two of them the
/// literals `true` and `false`) and 4 for library authors (`trait`, `impl`,
/// `lends`, `yield`).
pub const KEYWORDS: [&str; 33] = [
    "let",
    "var",
    "const",
    "fn",
    "struct",
    "enum",
    "domain",
    "type",
    "use",
    "pub",
    "if",
    "elif",
    "else",
    "for",
    "while",
    "in",
    "match",
    "return",
    "break",
    "continue",
    "and",
    "or",
    "not",
    "assert",
    "mut",
    "self",
    "unchecked",
    "true",
    "false",
    "trait",
    "impl",
    "lends",
    "yield",
];

/// The words that are literal values, as editors highlight them: the
/// booleans and `None` (a name the parser reads as the empty optional).
/// Lowercase `none` is an ordinary name.
pub const LITERAL_WORDS: &[&str] = &["true", "false", "None"];

/// What a literal word is, in [`LITERAL_WORDS`]' order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiteralWord {
    True,
    False,
    None,
}

impl LiteralWord {
    /// The literal word `text` spells, if it is one.
    pub fn of(text: &str) -> Option<LiteralWord> {
        let i = LITERAL_WORDS.iter().position(|w| *w == text)?;
        Some([LiteralWord::True, LiteralWord::False, LiteralWord::None][i])
    }
}

/// What to write instead of a word that v2.1 reserved and v2.2 does not.
/// The word itself is an ordinary name now.
pub fn removed_keyword(text: &str) -> Option<&'static str> {
    crate::grammar::REMOVED_KEYWORDS
        .iter()
        .find(|(word, _)| *word == text)
        .map(|(_, instead)| *instead)
}

impl TokenKind {
    pub fn keyword(text: &str) -> Option<TokenKind> {
        crate::grammar::KEYWORD_KINDS
            .iter()
            .find(|(word, _)| *word == text)
            .map(|(_, kind)| *kind)
    }

    /// How a keyword is written: `Fn` is `fn`.
    pub fn spelling(self) -> Option<&'static str> {
        crate::grammar::KEYWORD_KINDS
            .iter()
            .find(|(_, kind)| *kind == self)
            .map(|(word, _)| *word)
    }

    /// True for tokens that can only begin a statement or declaration, never
    /// continue an expression. Seeing one at the start of a line while a
    /// bracket is still open means the bracket was left unclosed.
    pub fn starts_statement(self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Let | Var
                | Fn
                | Struct
                | Enum
                | Domain
                | Type
                | Const
                | Trait
                | Impl
                | Use
                | Pub
                | For
                | While
                | Return
                | Break
                | Continue
                | Assert
                | At
        )
    }

    /// True for the kinds that carry no syntax: whitespace, comments and
    /// skipped lines.
    pub fn is_trivia(self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace | TokenKind::Comment | TokenKind::Skipped
        )
    }

    pub fn is_closer(self) -> bool {
        matches!(
            self,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
        )
    }

    /// True for binary operators that cannot end a statement, so a newline
    /// right after them continues the same logical line outside brackets.
    /// `>` and `>>` are excluded because they close generic type arguments.
    pub fn continues_line(self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            And | Or
                | Plus
                | Minus
                | Star
                | StarStar
                | Slash
                | Percent
                | Pipe
                | Caret
                | Amp
                | Shl
                | QuestionQuestion
                | EqEq
                | NotEq
                | Lt
                | LtEq
                | GtEq
                | Spaceship
        )
    }

    /// How the token is named in a diagnostic.
    pub fn describe(self) -> &'static str {
        use TokenKind::*;
        match self {
            Ident => "a name",
            Int | Float => "a number",
            Str | RawStr | StrText => "a string",
            Char => "a character",
            Label => "a label",
            Let => "`let`",
            Var => "`var`",
            Fn => "`fn`",
            Struct => "`struct`",
            Enum => "`enum`",
            Domain => "`domain`",
            Type => "`type`",
            Const => "`const`",
            Trait => "`trait`",
            Impl => "`impl`",
            Use => "`use`",
            Pub => "`pub`",
            If => "`if`",
            Elif => "`elif`",
            Else => "`else`",
            For => "`for`",
            While => "`while`",
            In => "`in`",
            Match => "`match`",
            Return => "`return`",
            Break => "`break`",
            Continue => "`continue`",
            And => "`and`",
            Or => "`or`",
            Not => "`not`",
            Assert => "`assert`",
            Mut => "`mut`",
            SelfKw => "`self`",
            True => "`true`",
            False => "`false`",
            Yield => "`yield`",
            Lends => "`lends`",
            Unchecked => "`unchecked`",
            LParen => "`(`",
            RParen => "`)`",
            LBracket => "`[`",
            RBracket => "`]`",
            LBrace => "`{`",
            RBrace => "`}`",
            Comma => "`,`",
            Colon => "`:`",
            ColonColon => "`::`",
            Semi => "`;`",
            Dot => "`.`",
            DotDot => "`..`",
            DotDotEq => "`..=`",
            Arrow => "`->`",
            FatArrow => "`=>`",
            At => "`@`",
            Question => "`?`",
            QuestionQuestion => "`??`",
            Eq => "`=`",
            EqEq => "`==`",
            NotEq => "`!=`",
            Lt => "`<`",
            LtEq => "`<=`",
            Spaceship => "`<=>`",
            Gt => "`>`",
            GtEq => "`>=`",
            Plus => "`+`",
            Minus => "`-`",
            Star => "`*`",
            StarStar => "`**`",
            Slash => "`/`",
            Percent => "`%`",
            Caret => "`^`",
            Amp => "`&`",
            Pipe => "`|`",
            Tilde => "`~`",
            Bang => "`!`",
            Shl => "`<<`",
            Shr => "`>>`",
            OpEq => "an assignment operator",
            Newline => "the end of the line",
            Indent => "an indented block",
            Dedent => "the end of the block",
            Eof => "the end of the file",
            Error => "an unreadable character",
            Whitespace => "a space",
            Comment => "a comment",
            Skipped => "a skipped line",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Per token, whether it is a named argument's label (`from` in
/// `f(from: x)`): it names the parameter, so it is never a hole. A `fn`'s
/// own parameters (`fn go(u: Int)`) are names, not labels.
pub fn arg_labels(kinds: &[TokenKind]) -> Vec<bool> {
    use TokenKind as K;
    let mut open = Vec::new();
    (0..kinds.len())
        .map(|i| {
            let label = kinds[i] == K::Ident
                && open.last() == Some(&Some(K::LParen))
                && kinds.get(i + 1) == Some(&K::Colon)
                && matches!(kinds[i - 1], K::LParen | K::Comma);
            let params = i >= 2 && kinds[i - 1] == K::Ident && kinds[i - 2] == K::Fn;
            match kinds[i] {
                K::LParen if params => open.push(None),
                K::LParen | K::LBracket | K::LBrace => open.push(Some(kinds[i])),
                K::RParen | K::RBracket | K::RBrace => {
                    open.pop();
                }
                _ => {}
            }
            label
        })
        .collect()
}
