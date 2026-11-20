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
    With,
    By,
    Mut,
    SelfKw,
    True,
    False,
    None,
    Yield,
    Where,
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
            "with" => With,
            "by" => By,
            "mut" => Mut,
            "self" => SelfKw,
            "true" => True,
            "false" => False,
            "none" => None,
            "yield" => Yield,
            "where" => Where,
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
        matches!(self, TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace)
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
            With => "`with`",
            By => "`by`",
            Mut => "`mut`",
            SelfKw => "`self`",
            True => "`true`",
            False => "`false`",
            None => "`none`",
            Yield => "`yield`",
            Where => "`where`",
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
