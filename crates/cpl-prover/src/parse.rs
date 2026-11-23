//! A parser for the expressions laws and invariants are written in.
//!
//! It reads the expression subset of `.cp` that a `law`, an `invariant`,
//! an `assert` or an index can contain, so the text of
//! `std/algebra.cp` can be handed to the prover as it stands:
//!
//! ```
//! use cpl_prover::parse;
//! let law = parse::law("law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))").unwrap();
//! assert_eq!(law.name, "assoc");
//! assert_eq!(parse::term("s.hi <= t.lo or t.hi <= s.lo").unwrap().to_string(), "s.hi <= t.lo or t.hi <= s.lo");
//! ```

use crate::laws::Law;
use crate::term::{BinOp, Error, Sort, Term, UnOp, INT_MAX, INT_MIN};

#[derive(Clone, PartialEq, Debug)]
enum Tok {
    Int(i128),
    Ident(String),
    Sym(&'static str),
    Eof,
}

const SYMBOLS: &[&str] = &[
    "..=", "=>", "==", "!=", "<=", ">=", "<<", ">>", "**", "??", "..", "(", ")", "[", "]", ",",
    ":", ".", "+", "-", "*", "/", "%", "<", ">", "=", "!", "?", "^", "&", "|",
];

fn lex(src: &str) -> Result<Vec<Tok>, Error> {
    let bytes = src.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            let (value, next) = lex_number(src, i)?;
            toks.push(Tok::Int(value));
            i = next;
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            toks.push(Tok::Ident(src[start..i].to_string()));
        } else if let Some(sym) = SYMBOLS.iter().find(|s| src[i..].starts_with(**s)) {
            toks.push(Tok::Sym(sym));
            i += sym.len();
        } else {
            let ch = src[i..].chars().next().unwrap();
            return Err(Error::new(format!("unexpected character `{ch}`")));
        }
    }
    toks.push(Tok::Eof);
    Ok(toks)
}

/// `42`, `1_000`, `0xff`, `0b101`, `1e9`, `2e5`.
fn lex_number(src: &str, start: usize) -> Result<(i128, usize), Error> {
    let bytes = src.as_bytes();
    let too_big = || Error::new("integer literal is too large");
    let radix = match src[start..].get(..2) {
        Some("0x") => 16,
        Some("0b") => 2,
        _ => 10,
    };
    let mut i = if radix == 10 { start } else { start + 2 };
    let mut value = 0i128;
    let digits_from = i;
    while i < bytes.len() && (bytes[i] == b'_' || (bytes[i] as char).is_digit(radix)) {
        if bytes[i] != b'_' {
            let digit = (bytes[i] as char).to_digit(radix).unwrap() as i128;
            value = value
                .checked_mul(radix as i128)
                .and_then(|v| v.checked_add(digit))
                .ok_or_else(too_big)?;
        }
        i += 1;
    }
    if i == digits_from {
        return Err(Error::new("a number needs digits"));
    }
    let exponent_follows =
        radix == 10 && i + 1 < bytes.len() && bytes[i] == b'e' && bytes[i + 1].is_ascii_digit();
    if exponent_follows {
        i += 1;
        let mut exp = 0u32;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            exp = exp
                .saturating_mul(10)
                .saturating_add((bytes[i] - b'0') as u32);
            i += 1;
        }
        value = 10i128
            .checked_pow(exp)
            .and_then(|p| value.checked_mul(p))
            .ok_or_else(too_big)?;
    }
    Ok((value, i))
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

type Res<T> = Result<T, Error>;

impl Parser {
    fn new(src: &str) -> Res<Parser> {
        Ok(Parser {
            toks: lex(src)?,
            pos: 0,
        })
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos]
    }

    fn peek_at(&self, ahead: usize) -> &Tok {
        &self.toks[(self.pos + ahead).min(self.toks.len() - 1)]
    }

    fn next(&mut self) -> Tok {
        let tok = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        tok
    }

    fn at_sym(&self, sym: &str) -> bool {
        matches!(self.peek(), Tok::Sym(s) if *s == sym)
    }

    fn at_word(&self, word: &str) -> bool {
        matches!(self.peek(), Tok::Ident(s) if s == word)
    }

    fn eat_sym(&mut self, sym: &str) -> bool {
        let hit = self.at_sym(sym);
        if hit {
            self.next();
        }
        hit
    }

    fn eat_word(&mut self, word: &str) -> bool {
        let hit = self.at_word(word);
        if hit {
            self.next();
        }
        hit
    }

    fn describe(&self) -> String {
        match self.peek() {
            Tok::Int(v) => format!("`{v}`"),
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Sym(s) => format!("`{s}`"),
            Tok::Eof => "the end of the text".to_string(),
        }
    }

    fn expect_sym(&mut self, sym: &str) -> Res<()> {
        if self.eat_sym(sym) {
            Ok(())
        } else {
            Err(Error::new(format!(
                "expected `{sym}`, found {}",
                self.describe()
            )))
        }
    }

    fn expect_word(&mut self, word: &str) -> Res<()> {
        if self.eat_word(word) {
            Ok(())
        } else {
            Err(Error::new(format!(
                "expected `{word}`, found {}",
                self.describe()
            )))
        }
    }

    fn ident(&mut self) -> Res<String> {
        match self.peek().clone() {
            Tok::Ident(name) => {
                self.next();
                Ok(name)
            }
            _ => Err(Error::new(format!(
                "expected a name, found {}",
                self.describe()
            ))),
        }
    }

    fn end(&self) -> Res<()> {
        if *self.peek() == Tok::Eof {
            Ok(())
        } else {
            Err(Error::new(format!("unexpected {}", self.describe())))
        }
    }

    /// Whether the `(` at the cursor opens the parameter list of a lambda.
    fn lambda_ahead(&self) -> bool {
        let mut depth = 0usize;
        let mut i = self.pos;
        while i < self.toks.len() {
            match &self.toks[i] {
                Tok::Sym("(") => depth += 1,
                Tok::Sym(")") => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(self.toks.get(i + 1), Some(Tok::Sym("=>")));
                    }
                }
                Tok::Ident(_) | Tok::Sym(",") => {}
                _ => return false,
            }
            i += 1;
        }
        false
    }

    fn expr(&mut self) -> Res<Term> {
        if matches!(self.peek(), Tok::Ident(_)) && matches!(self.peek_at(1), Tok::Sym("=>")) {
            let param = self.ident()?;
            self.next();
            return Ok(Term::Lambda(vec![param], Box::new(self.expr()?)));
        }
        if self.at_sym("(") && self.lambda_ahead() {
            self.next();
            let mut params = Vec::new();
            while !self.eat_sym(")") {
                params.push(self.ident()?);
                if !self.at_sym(")") {
                    self.expect_sym(",")?;
                }
            }
            self.expect_sym("=>")?;
            return Ok(Term::Lambda(params, Box::new(self.expr()?)));
        }
        if self.eat_word("if") {
            let cond = self.expr()?;
            self.expect_sym(":")?;
            let then = self.expr()?;
            self.expect_word("else")?;
            self.expect_sym(":")?;
            let other = self.expr()?;
            return Ok(Term::If(Box::new(cond), Box::new(then), Box::new(other)));
        }
        let lhs = self.or()?;
        if self.eat_sym("??") {
            let rhs = self.expr()?;
            return Ok(Term::binary(BinOp::Coalesce, lhs, rhs));
        }
        Ok(lhs)
    }

    fn or(&mut self) -> Res<Term> {
        let mut lhs = self.and()?;
        while self.eat_word("or") {
            lhs = Term::binary(BinOp::Or, lhs, self.and()?);
        }
        Ok(lhs)
    }

    fn and(&mut self) -> Res<Term> {
        let mut lhs = self.not()?;
        while self.eat_word("and") {
            lhs = Term::binary(BinOp::And, lhs, self.not()?);
        }
        Ok(lhs)
    }

    fn not(&mut self) -> Res<Term> {
        if self.eat_word("not") {
            return Ok(self.not()?.negate());
        }
        self.comparison()
    }

    fn comparison_op(&self) -> Option<BinOp> {
        match self.peek() {
            Tok::Sym("==") => Some(BinOp::Eq),
            Tok::Sym("!=") => Some(BinOp::Ne),
            Tok::Sym("<") => Some(BinOp::Lt),
            Tok::Sym("<=") => Some(BinOp::Le),
            Tok::Sym(">") => Some(BinOp::Gt),
            Tok::Sym(">=") => Some(BinOp::Ge),
            _ => None,
        }
    }

    /// `a < b`, the chain `lo <= x < hi`, and `x in lo..hi`.
    fn comparison(&mut self) -> Res<Term> {
        let first = self.binary(6)?;
        if self.eat_word("in") {
            let (lo, hi, inclusive) = self.range()?;
            return Ok(Term::InRange(
                Box::new(first),
                Box::new(lo),
                Box::new(hi),
                inclusive,
            ));
        }
        let mut result: Option<Term> = None;
        let mut left = first;
        while let Some(op) = self.comparison_op() {
            self.next();
            let right = self.binary(6)?;
            let link = Term::binary(op, left, right.clone());
            result = Some(match result {
                Some(chain) => chain.and(link),
                None => link,
            });
            left = right;
        }
        Ok(result.unwrap_or(left))
    }

    fn range(&mut self) -> Res<(Term, Term, bool)> {
        let lo = self.binary(10)?;
        let inclusive = if self.eat_sym("..=") {
            true
        } else {
            self.expect_sym("..")?;
            false
        };
        Ok((lo, self.binary(10)?, inclusive))
    }

    fn binary_op(&self, level: u8) -> Option<BinOp> {
        let op = match self.peek() {
            Tok::Sym("|") => BinOp::BitOr,
            Tok::Sym("^") => BinOp::Xor,
            Tok::Ident(word) if word == "xor" => BinOp::Xor,
            Tok::Sym("&") => BinOp::BitAnd,
            Tok::Sym("<<") => BinOp::Shl,
            Tok::Sym(">>") => BinOp::Shr,
            Tok::Sym("+") => BinOp::Add,
            Tok::Sym("-") => BinOp::Sub,
            Tok::Sym("*") => BinOp::Mul,
            Tok::Sym("/") => BinOp::Div,
            Tok::Sym("%") => BinOp::Mod,
            _ => return None,
        };
        let op_level = match op {
            BinOp::BitOr => 6,
            BinOp::Xor => 7,
            BinOp::BitAnd => 8,
            BinOp::Shl | BinOp::Shr => 9,
            BinOp::Add | BinOp::Sub => 10,
            _ => 11,
        };
        (op_level == level).then_some(op)
    }

    /// The left-associative operators from `|` (level 6) to `*` (level 11).
    fn binary(&mut self, level: u8) -> Res<Term> {
        if level > 11 {
            return self.unary();
        }
        let mut lhs = self.binary(level + 1)?;
        while let Some(op) = self.binary_op(level) {
            self.next();
            lhs = Term::binary(op, lhs, self.binary(level + 1)?);
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Res<Term> {
        if self.eat_sym("-") {
            return Ok(match self.unary()? {
                Term::Int(v) => Term::Int(-v),
                other => Term::Unary(UnOp::Neg, Box::new(other)),
            });
        }
        let base = self.postfix()?;
        if self.eat_sym("**") {
            return Ok(Term::binary(BinOp::Pow, base, self.unary()?));
        }
        Ok(base)
    }

    fn args(&mut self, close: &str) -> Res<Vec<Term>> {
        let mut args = Vec::new();
        while !self.eat_sym(close) {
            args.push(self.expr()?);
            if !self.at_sym(close) {
                self.expect_sym(",")?;
            }
        }
        Ok(args)
    }

    /// The arguments of `a[...]`. A range `lo..hi` is the pair
    /// `(lo: lo, hi: hi)`, the shape a slice law talks about; `lo..=hi`
    /// ends one later.
    fn index_args(&mut self) -> Res<Vec<Term>> {
        let mut args = Vec::new();
        while !self.eat_sym("]") {
            let first = self.expr()?;
            let inclusive = self.at_sym("..=");
            if inclusive || self.at_sym("..") {
                self.next();
                let mut hi = self.binary(10)?;
                if inclusive {
                    hi = Term::binary(BinOp::Add, hi, Term::Int(1));
                }
                args.push(Term::Tuple(vec![
                    (Some("lo".to_string()), first),
                    (Some("hi".to_string()), hi),
                ]));
            } else {
                args.push(first);
            }
            if !self.at_sym("]") {
                self.expect_sym(",")?;
            }
        }
        Ok(args)
    }

    fn postfix(&mut self) -> Res<Term> {
        let mut term = self.primary()?;
        loop {
            if self.eat_sym("(") {
                term = Term::Call(Box::new(term), self.args(")")?);
            } else if self.eat_sym("[") {
                term = Term::Index(Box::new(term), self.index_args()?);
            } else if self.eat_sym(".") {
                let field = match self.next() {
                    Tok::Ident(name) => name,
                    Tok::Int(index) => index.to_string(),
                    _ => return Err(Error::new("expected a field name after `.`")),
                };
                term = match (&term, field.as_str()) {
                    (Term::Var(base), "MIN" | "min") if base == "Int" => Term::Int(INT_MIN),
                    (Term::Var(base), "MAX" | "max") if base == "Int" => Term::Int(INT_MAX),
                    _ => Term::Field(Box::new(term), field),
                };
            } else if self.eat_sym("!") {
                term = Term::Unary(UnOp::Unwrap, Box::new(term));
            } else {
                return Ok(term);
            }
        }
    }

    fn primary(&mut self) -> Res<Term> {
        if !matches!(self.peek(), Tok::Int(_) | Tok::Ident(_) | Tok::Sym("(")) {
            return Err(Error::new(format!(
                "expected an expression, found {}",
                self.describe()
            )));
        }
        match self.next() {
            Tok::Int(v) => Ok(Term::Int(v)),
            Tok::Ident(word) => Ok(match word.as_str() {
                "true" => Term::Bool(true),
                "false" => Term::Bool(false),
                "none" => Term::None,
                "if" | "else" | "and" | "or" | "not" | "in" | "xor" | "law" => {
                    return Err(Error::new(format!("`{word}` can't start an expression")));
                }
                _ => Term::Var(word),
            }),
            Tok::Sym("(") => {
                let mut items = Vec::new();
                let mut trailing_comma = false;
                while !self.eat_sym(")") {
                    let named = matches!(self.peek(), Tok::Ident(_))
                        && matches!(self.peek_at(1), Tok::Sym(":"));
                    let name = if named {
                        let name = self.ident()?;
                        self.next();
                        Some(name)
                    } else {
                        None
                    };
                    items.push((name, self.expr()?));
                    trailing_comma = self.at_sym(",");
                    if !self.at_sym(")") {
                        self.expect_sym(",")?;
                    }
                }
                let grouping = items.len() == 1 && items[0].0.is_none() && !trailing_comma;
                Ok(if grouping {
                    items.pop().unwrap().1
                } else {
                    Term::Tuple(items)
                })
            }
            _ => unreachable!(),
        }
    }

    fn sort(&mut self) -> Res<Sort> {
        if self.eat_sym("?") {
            return Ok(Sort::Opt(Box::new(self.sort()?)));
        }
        if self.at_sym("(") {
            // `(Int, Int)` is a tuple; `(n + 1)..m` is a range.
            let start = self.pos;
            if let Ok(tuple) = self.tuple_sort() {
                if !self.at_sym("..") && !self.at_sym("..=") {
                    return Ok(tuple);
                }
            }
            self.pos = start;
        }
        if let Tok::Ident(word) = self.peek() {
            let name = word.clone();
            let type_like = name.starts_with(|c: char| c.is_ascii_uppercase());
            let ends_here = !matches!(
                self.peek_at(1),
                Tok::Sym("." | ".." | "..=" | "+" | "-" | "*" | "/" | "(" | "[")
            );
            if type_like && ends_here {
                self.next();
                return Ok(match name.as_str() {
                    "Int" => Sort::Int,
                    "Bool" => Sort::Bool,
                    _ => Sort::Named(name),
                });
            }
        }
        let (lo, hi, inclusive) = self.range()?;
        Ok(Sort::Range {
            lo: Box::new(lo),
            hi: Box::new(hi),
            inclusive,
        })
    }

    fn tuple_sort(&mut self) -> Res<Sort> {
        self.expect_sym("(")?;
        let mut items = Vec::new();
        while !self.eat_sym(")") {
            let named =
                matches!(self.peek(), Tok::Ident(_)) && matches!(self.peek_at(1), Tok::Sym(":"));
            let name = if named {
                let name = self.ident()?;
                self.next();
                Some(name)
            } else {
                None
            };
            items.push((name, self.sort()?));
            if !self.at_sym(")") {
                self.expect_sym(",")?;
            }
        }
        Ok(Sort::Tuple(items))
    }
}

/// Parse an expression: `op(op(a, b), c) == op(a, op(b, c))`.
pub fn term(src: &str) -> Result<Term, Error> {
    let mut p = Parser::new(src)?;
    let term = p.expr().map_err(|e| in_source(e, src))?;
    p.end().map_err(|e| in_source(e, src))?;
    Ok(term)
}

/// Parse a type: `Int`, `0..n`, `1..=2e5`, `(Int, Int)`, `(lo: Int, hi: Int)`, `?Int`, `T`.
pub fn sort(src: &str) -> Result<Sort, Error> {
    let mut p = Parser::new(src)?;
    let sort = p.sort().map_err(|e| in_source(e, src))?;
    p.end().map_err(|e| in_source(e, src))?;
    Ok(sort)
}

/// Parse a law as it is written in a struct or a projection:
/// `law disjoint(i, j) = i != j`. The word `law` may be left out, and so
/// may the parameter types.
pub fn law(src: &str) -> Result<Law, Error> {
    let mut p = Parser::new(src)?;
    let parsed = (|| {
        p.eat_word("law");
        let name = p.ident()?;
        p.expect_sym("(")?;
        let mut params = Vec::new();
        while !p.eat_sym(")") {
            let param = p.ident()?;
            let sort = if p.eat_sym(":") {
                Some(p.sort()?)
            } else {
                None
            };
            params.push((param, sort));
            if !p.at_sym(")") {
                p.expect_sym(",")?;
            }
        }
        p.expect_sym("=")?;
        let body = p.expr()?;
        p.end()?;
        Ok(Law { name, params, body })
    })();
    parsed.map_err(|e| in_source(e, src))
}

fn in_source(e: Error, src: &str) -> Error {
    Error::new(format!("{} in `{}`", e.message, src.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(src: &str) {
        let parsed = term(src).unwrap();
        assert_eq!(parsed.to_string(), src);
        assert_eq!(term(&parsed.to_string()).unwrap(), parsed);
    }

    #[test]
    fn printing_gives_back_the_source() {
        for src in [
            "op(op(a, b), c) == op(a, op(b, c))",
            "s.hi <= t.lo or t.hi <= s.lo",
            "not (lt(a, b) and lt(b, c)) or lt(a, c)",
            "(x, y) => if x.0 > y.0: x else: if y.0 > x.0: y else: (x.0, x.1 + y.1)",
            "a => -a",
            "(sum: x.sum + y.sum, pre: max(x.pre, x.sum + y.pre))",
            "a - (b - c) - d",
            "(a + b) * c % m",
            "a[i] * b[i] + g[r, c]",
            "i in 0..n",
            "v in lo + 1..=hi",
            "a! + (b ?? 0)",
            "x ^ y & z | w << 2",
            "-(a + b) ** 2",
            "Int.MIN <= x and x <= Int.MAX",
        ] {
            round_trip(src);
        }
    }

    #[test]
    fn literals() {
        assert_eq!(term("1e9").unwrap(), Term::Int(1_000_000_000));
        assert_eq!(term("2e5").unwrap(), Term::Int(200_000));
        assert_eq!(term("1_000_000").unwrap(), Term::Int(1_000_000));
        assert_eq!(term("0xff").unwrap(), Term::Int(255));
        assert_eq!(term("0b1011").unwrap(), Term::Int(11));
        assert_eq!(term("-5").unwrap(), Term::Int(-5));
        assert_eq!(term("Int.MAX").unwrap(), Term::Int(INT_MAX));
        assert_eq!(term("Int.min").unwrap(), Term::Int(INT_MIN));
    }

    #[test]
    fn comparison_chains_are_conjunctions() {
        assert_eq!(
            term("0 <= i < n").unwrap(),
            term("0 <= i and i < n").unwrap()
        );
    }

    #[test]
    fn a_range_index_is_a_pair_of_ends() {
        assert_eq!(term("a[l..m]").unwrap(), term("a[(lo: l, hi: m)]").unwrap());
        assert_eq!(
            term("a[l..=m - 1]").unwrap(),
            term("a[(lo: l, hi: m - 1 + 1)]").unwrap()
        );
        assert_eq!(term("g[r, c]").unwrap().to_string(), "g[r, c]");
    }

    #[test]
    fn tuples_and_grouping() {
        assert_eq!(term("(a)").unwrap(), Term::var("a"));
        assert!(matches!(term("(a, b)").unwrap(), Term::Tuple(items) if items.len() == 2));
        assert!(matches!(term("(a,)").unwrap(), Term::Tuple(items) if items.len() == 1));
        assert!(matches!(term("()").unwrap(), Term::Tuple(items) if items.is_empty()));
    }

    #[test]
    fn sorts() {
        assert_eq!(sort("Int").unwrap(), Sort::Int);
        assert_eq!(sort("?Int").unwrap(), Sort::Opt(Box::new(Sort::Int)));
        assert_eq!(sort("T").unwrap(), Sort::Named("T".to_string()));
        assert_eq!(sort("(Int, Bool)").unwrap().to_string(), "(Int, Bool)");
        assert_eq!(
            sort("(lo: 0..n, hi: 0..=n)").unwrap().to_string(),
            "(lo: 0..n, hi: 0..=n)"
        );
        assert_eq!(sort("1..=2e5").unwrap().to_string(), "1..=200000");
        assert_eq!(sort("(n + 1)..m").unwrap().to_string(), "n + 1..m");
        assert_eq!(sort("K..2 * K").unwrap().to_string(), "K..2 * K");
    }

    #[test]
    fn laws_as_written_in_the_library() {
        let assoc =
            law("law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))").unwrap();
        assert_eq!(assoc.params.len(), 3);
        assert_eq!(
            assoc.params[0],
            ("a".to_string(), Some(Sort::Named("T".to_string())))
        );
        let disjoint = law("law disjoint(i, j) = i != j").unwrap();
        assert_eq!(
            disjoint.params,
            vec![("i".to_string(), None), ("j".to_string(), None)]
        );
        assert_eq!(disjoint.body.to_string(), "i != j");
    }

    #[test]
    fn errors_say_what_was_expected() {
        assert_eq!(
            term("a +").unwrap_err().message,
            "expected an expression, found the end of the text in `a +`"
        );
        assert_eq!(term("a b").unwrap_err().message, "unexpected `b` in `a b`");
        assert_eq!(
            term("if a: b").unwrap_err().message,
            "expected `else`, found the end of the text in `if a: b`"
        );
        assert!(term("a # b")
            .unwrap_err()
            .message
            .starts_with("unexpected character `#`"));
        assert!(term("99999999999999999999999999999999999999999").is_err());
    }
}
