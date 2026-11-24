//! Recovery for habits from other languages: one module per language,
//! and the helpers they share. Each recovery reports one error with the
//! CP Lang form and reads the line as that form, so nothing cascades.

mod c;
mod c_decl;
mod other;
mod python;
mod rust;

use super::expr::unescape;
use super::*;
use crate::literal::{self, StrPart};
use c_decl::*;
use cpl_diag::habits;

impl<'a> Parser<'a> {
    /// The help for a token where a statement should have ended: `;`
    /// from C, or the comma of Python's swap.
    pub(super) fn statement_end_help(&self, found: T, span: Span, diag: Diagnostic) -> Diagnostic {
        if let Some(decl) = self.c_decl_error() {
            return decl;
        }
        if found == T::Semi {
            return diag
                .with_help(habits::STATEMENT_END_HELP.say(&[]))
                .with_fix(habits::STATEMENT_END_HELP_2.say(&[]), span, "");
        }
        match (found == T::Comma).then(|| self.tuple_assign()).flatten() {
            Some((at, both, help)) => {
                diag.with_help(help)
                    .with_fix(habits::WRITE.say(&[&both]), at, both.clone())
            }
            None => diag,
        }
    }

    /// `if c: x += 1;`: C's `;` after a one-line body is its one error, so
    /// an `elif` or `else` on the next line still has its `if`.
    pub(in crate::parser) fn trailing_semi(&mut self) {
        if self.brace_depth == 0 && self.at(T::Semi) {
            let span = self.span();
            let diag = msgs::stmt::EXPECTED_STMT_END.error(span, &[&T::Semi.describe()]);
            let diag = self.statement_end_help(T::Semi, span, diag);
            self.error(diag);
            self.bump();
        }
    }

    /// Where the current token's line starts past its indent, and its
    /// text from there.
    fn this_line(&self) -> Option<(usize, &'a str)> {
        let at = self.tokens.get(self.pos)?.span.start as usize;
        let begin = self.src[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = self.src[at..].find('\n').map_or(self.src.len(), |i| at + i);
        let line = self.src[begin..end].trim_end();
        let indent = line.len() - line.trim_start().len();
        Some((begin + indent, &line[indent..]))
    }

    /// The source text of a span relative to the item.
    fn item_text(&self, span: Span) -> String {
        let at = |p: u32| (p as i64 - self.delta) as usize;
        self.src[at(span.start)..at(span.end)].to_string()
    }

    /// `if c {` with the `}` alone on a later line: the edits that make it
    /// `if c:` over the same indented body.
    pub(super) fn brace_header_edits(&self) -> Option<Vec<(Span, String)>> {
        let blank_after = |at: u32| {
            let rest = self.src.get(at as usize..).unwrap_or("");
            rest.split('\n').next().is_some_and(|l| l.trim().is_empty())
        };
        let open = self.tokens.get(self.pos)?.span;
        if !(self.at(T::LBrace) && blank_after(open.end)) {
            return None;
        }
        let mut depth = 0;
        for (i, t) in self.tokens.iter().enumerate().skip(self.pos) {
            match t.kind {
                T::LBrace => depth += 1,
                T::RBrace if depth == 1 => {
                    let line = self.src.get(..t.span.start as usize)?.rfind('\n')?;
                    let indent = &self.src[line + 1..t.span.start as usize];
                    let next = self.tokens.get(i + 1).filter(|n| n.kind == T::Else);
                    let close = match next {
                        // `} else {`: the `else` keeps the `}`'s indentation.
                        Some(e) => Span::new(t.span.start, e.span.start),
                        None if blank_after(t.span.end) => {
                            // A CRLF line ends in `\r\n`: drop both with the `}` line.
                            let cr = self.src[..line].ends_with('\r') as usize;
                            Span::new((line - cr) as u32, t.span.end)
                        }
                        None => return None,
                    };
                    return indent.trim().is_empty().then(|| {
                        vec![
                            (Span::new(self.prev_end(), self.span().end), ":".into()),
                            (self.rel(close), String::new()),
                        ]
                    });
                }
                T::RBrace => depth -= 1,
                _ => {}
            }
        }
        None
    }

    /// Whether the last token is a binary operator, or a character the
    /// lexer rejected: `a + * b` is an operator typed twice, and the `*` in
    /// `a + λ * b` multiplies; neither is a dereference.
    pub(super) fn after_operator(&self) -> bool {
        self.pos.checked_sub(1).is_some_and(|i| {
            matches!(
                self.tokens[i].kind,
                T::Error
                    | T::Plus
                    | T::Minus
                    | T::Star
                    | T::StarStar
                    | T::Slash
                    | T::Percent
                    | T::Caret
                    | T::Amp
                    | T::Pipe
                    | T::Shl
                    | T::Shr
                    | T::EqEq
                    | T::NotEq
                    | T::Lt
                    | T::LtEq
                    | T::Gt
                    | T::GtEq
                    | T::Spaceship
                    | T::And
                    | T::Or
                    | T::QuestionQuestion
                    | T::DotDot
                    | T::DotDotEq
            )
        })
    }

    /// The name called by the innermost open `(` before this token.
    fn callee(&self) -> Option<&'a str> {
        let mut depth = 0;
        let mut open = None;
        for i in (0..self.pos).rev() {
            match self.tokens[i].kind {
                T::RParen => depth += 1,
                T::LParen if depth == 0 => {
                    open = Some(i);
                    break;
                }
                T::LParen => depth -= 1,
                _ => {}
            }
        }
        let callee = open
            .and_then(|i| i.checked_sub(1))
            .map(|i| &self.tokens[i])?;
        (callee.kind == T::Ident).then(|| callee.span.text(self.src))
    }

    /// `a <=> b`: C++'s three-way comparison. One error with the `cmp(a, b)`
    /// fix, read as that call.
    pub(super) fn three_way(&mut self, left: Expr) -> Expr {
        let op = self.span();
        self.bump();
        let right = self.range();
        let span = Span::new(left.span.start, right.span.end.max(op.end));
        let text = |s: Span| {
            let at = |p: u32| (p as i64 - self.delta) as usize;
            &self.src[at(s.start)..at(s.end)]
        };
        let (l, r) = (text(left.span), text(right.span));
        let ours = format!("cmp({l}, {r})");
        self.recovered(
            cpl_diag::msgs::foreign::THREE_WAY
                .error(op, &[])
                .with_help(habits::THREE_WAY.say(&[]))
                .with_fix(habits::THREE_WAY_2.say(&[&l, &r]), span, &ours),
        );
        let arg = |value: Expr| Arg {
            name: None,
            mutable: false,
            value,
        };
        let callee = Expr {
            kind: ExprKind::Name("cmp".to_string()),
            span: op,
        };
        Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args: vec![arg(left), arg(right)],
            },
            span,
        }
    }

    /// Steps over a comparison operator, reading `!in` as `not in` and
    /// Python's `x is None` and `x is not None` as `==` and `!=`, each one
    /// error with a fix.
    pub(super) fn cmp_token(&mut self) {
        if self.at(T::Bang) {
            let span = self.span();
            self.error(
                cpl_diag::msgs::foreign::BANG_NOT
                    .error(span, &[])
                    .with_help(habits::CMP_TOKEN.say(&[]))
                    .with_fix(habits::CMP_TOKEN_2.say(&[]), span, "not "),
            );
        }
        if !self.at_ident("is") {
            self.bump();
            return;
        }
        let start = self.span();
        self.bump();
        let (foreign, ours) = if self.eat(T::Not) {
            ("is not", "!=")
        } else {
            ("is", "==")
        };
        let span = Span::new(start.start, self.rel(self.tokens[self.pos - 1].span).end);
        self.recovered(
            cpl_diag::msgs::foreign::CMP_TOKEN
                .error(span, &[&ours, &foreign])
                .with_help(habits::CMP_TOKEN_3.say(&[&ours]))
                .with_fix(habits::WRITE.say(&[&ours]), span, ours),
        );
    }

    /// `if c: a else: b`, as the ternaries of other languages read.
    pub(super) fn inline_if(cond: Expr, yes: Expr, no: Expr, span: Span) -> Expr {
        let block = |e: Expr| Block {
            stmts: vec![Stmt {
                public: false,
                decorators: Vec::new(),
                span: e.span,
                kind: StmtKind::Expr(e),
            }],
            span,
        };
        Expr {
            kind: ExprKind::If {
                branches: vec![(Cond::Expr(cond), block(yes))],
                otherwise: Some(block(no)),
            },
            span,
        }
    }

    /// `lambda x: x * 2`: Python's lambda. One error with the `x => x * 2`
    /// fix, read as that lambda.
    /// Python's `lambda x: e`, Rust's `|x| e` or Java's `x -> e`, read as
    /// `x => e`.
    pub(super) fn foreign_lambda(&mut self) -> Option<Expr> {
        if self.at_ident("lambda") && matches!(self.peek_n(1), T::Ident | T::Colon) {
            return Some(self.python_lambda());
        }
        if self.at(T::Ident) && self.peek_n(1) == T::Arrow {
            return Some(self.arrow_lambda());
        }
        self.rust_closure()
    }

    /// Where a loop body starting at token `from` ends: a `{ .. }`
    /// group, an indented block, or the rest of the line.
    fn body_end(&self, from: usize) -> usize {
        let toks = self.tokens;
        match toks.get(from).map(|t| t.kind) {
            Some(T::LBrace) => after_group(toks, from),
            Some(T::Newline) if toks.get(from + 1).map(|t| t.kind) == Some(T::Indent) => {
                let mut depth = 0;
                (from + 1..toks.len())
                    .find(|&i| {
                        match toks[i].kind {
                            T::Indent => depth += 1,
                            T::Dedent => depth -= 1,
                            _ => {}
                        }
                        depth == 0
                    })
                    .unwrap_or(toks.len())
            }
            _ => (from..toks.len())
                .find(|&i| toks[i].kind == T::Newline)
                .unwrap_or(toks.len()),
        }
    }

    /// `{1, 2}`, a set written the other languages' way: `Set([1, 2])`.
    /// `f({a, b})`, C++'s braced argument, is a pair: `f((a, b))`.
    pub(in crate::parser) fn brace_set(&mut self, span: Span, open: Span, close: Span, pair: bool) {
        if self.c_decl().is_some() {
            return;
        }
        let (message, help, l, r) = match pair {
            true => (
                &msgs::syntax::BRACE_PAIR,
                "braces make a block; parentheses make a tuple",
                "(",
                ")",
            ),
            false => (
                &msgs::syntax::BRACE_SET,
                "braces make a block; brackets make lists and maps",
                "Set([",
                "])",
            ),
        };
        let mut diag = message.error(span, &[]).with_help(help);
        diag.fixes.push(cpl_diag::Fix {
            title: habits::WRITE.say(&[&format!("{l}..{r}")]),
            edits: vec![(open, l.into()), (close, r.into())],
        });
        self.error(diag);
    }

    /// The built-in type a foreign type name here stands for. Rust's
    /// `Vec<T>` counts only with `<`: `std.geometry`'s `Vec` is a point.
    pub(in crate::parser) fn foreign_type_here(&self) -> Option<&'static str> {
        match self.text() {
            "Vec" | "ArrayList" | "Vector" if self.peek_n(1) == T::Lt => Some("List"),
            name => foreign_type(name),
        }
    }
}

/// Where the line holding `at` starts.
fn line_start(src: &str, at: usize) -> usize {
    src[..at].rfind('\n').map_or(0, |i| i + 1)
}

/// The built-in type another language's lowercase name stands for.
fn foreign_type(name: &str) -> Option<&'static str> {
    Some(match name {
        "int" | "long" | "ll" | "i64" | "i32" | "int64" | "integer" | "usize" => "Int",
        "string" | "str" => "Str",
        "bool" | "boolean" => "Bool",
        "float" | "double" | "f64" => "F64",
        "char" => "Char",
        "vector" | "vec" | "list" => "List",
        "map" | "dict" | "unordered_map" => "Map",
        "set" | "unordered_set" => "Set",
        _ => return None,
    })
}
