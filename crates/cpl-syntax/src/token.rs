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

/// What to write instead of a word that v2.1 reserved and v2.2 does not.
/// The word itself is an ordinary name now.
pub fn removed_keyword(text: &str) -> Option<&'static str> {
    Some(match text {
        "defer" => "undo by hand after the call: `cur.push(x); dfs(); cur.pop()`",
        "assume" => "write `assert`, which is always checked",
        "where" => "write the bound in the angle brackets: `fn f<T: Ord>(...)`",
        "inline" => "drop it: closures and `lends` bodies are always inlined",
        "lend" => "end the `lends` body with the place itself: `mut self.data[i]`",
        "law" => "write `@law name(args) = condition`",
        _ => return None,
    })
}

impl TokenKind {
    pub fn keyword(text: &str) -> Option<TokenKind> {
        use TokenKind::*;
        Some(match text {
            "let" => Let,
            "var" => Var,
            "fn" => Fn,
            "struct" => Struct,
            "enum" => Enum,
            "domain" => Domain,
            "type" => Type,
            "const" => Const,
            "trait" => Trait,
            "impl" => Impl,
            "use" => Use,
            "pub" => Pub,
            "if" => If,
            "elif" => Elif,
            "else" => Else,
            "for" => For,
            "while" => While,
            "in" => In,
            "match" => Match,
            "return" => Return,
            "break" => Break,
            "continue" => Continue,
            "and" => And,
            "or" => Or,
            "not" => Not,
            "assert" => Assert,
            "mut" => Mut,
            "self" => SelfKw,
            "true" => True,
            "false" => False,
            "lends" => Lends,
            "yield" => Yield,
            "unchecked" => Unchecked,
            _ => return Option::None,
        })
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
            Str | RawStr => "a string",
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
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
