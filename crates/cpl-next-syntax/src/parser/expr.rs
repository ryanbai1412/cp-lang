//! Expressions: precedence climbing over `ops`'s table.

use super::*;
use crate::grammar::{contextual, Contextual};
use crate::ops::{self, Assoc, Level};

/// Tokens that can begin an operand.
fn begins_operand(kind: TokenKind) -> bool {
    matches!(
        kind,
        T::Ident
            | T::Int
            | T::Float
            | T::Str
            | T::RawStr
            | T::Char
            | T::True
            | T::False
            | T::SelfKw
            | T::LParen
            | T::LBracket
            | T::LBrace
            | T::At
    ) || ops::prefix(kind).is_some()
        || ops::RANGE.contains(&kind)
}

impl Parser<'_> {
    pub(super) fn expr(&mut self) {
        match self.peek() {
            T::If => self.if_clauses(NodeKind::IfExpr, |p| {
                if p.want(T::Colon, "`:` and the value") {
                    p.value_body();
                }
            }),
            T::Match => self.match_expr(),
            _ if self.lambda_ahead() => self.lambda(),
            _ => self.expr_at(Level::Expr),
        }
    }

    /// `x =>` or `(...) =>` / `(...) ->`.
    fn lambda_ahead(&self) -> bool {
        match self.peek() {
            T::Ident => self.nth(1) == T::FatArrow,
            T::LParen => {
                let mut depth = 0usize;
                for n in 0.. {
                    match self.nth(n) {
                        T::LParen | T::LBracket | T::LBrace => depth += 1,
                        T::RParen | T::RBracket | T::RBrace => {
                            depth -= 1;
                            if depth == 0 {
                                return matches!(self.nth(n + 1), T::FatArrow | T::Arrow);
                            }
                        }
                        T::Eof | T::Newline => return false,
                        _ => {}
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn lambda(&mut self) {
        self.start(NodeKind::LambdaExpr);
        if self.at(T::LParen) {
            self.list(
                NodeKind::ParamList,
                T::RParen,
                "the lambda's parameters",
                |p| {
                    p.eat(T::Mut);
                    p.pattern();
                    if p.eat(T::Colon) {
                        p.ty();
                    }
                },
            );
            if self.eat(T::Arrow) {
                self.ty();
            }
        } else {
            self.start(NodeKind::ParamList);
            self.pattern();
            self.finish();
        }
        if self.want(T::FatArrow, "`=>` before the lambda's body") {
            self.arm_body();
        }
        self.finish();
    }

    /// An expression whose operators all bind at least as tight as `min`.
    pub(super) fn expr_at(&mut self, min: Level) {
        let at = self.checkpoint();
        self.operand(min);
        loop {
            let kind = self.peek();
            if ops::RANGE.contains(&kind) && min <= Level::Range {
                self.start_at(at, NodeKind::RangeExpr);
                self.range_rest();
                self.finish();
                continue;
            }
            if let Some(word) = self.contextual() {
                if matches!(word, Contextual::Max | Contextual::Min)
                    && min <= Level::Compare
                    && self.nth(1) == T::Eq
                    && self.glued(1)
                {
                    self.start_at(at, NodeKind::KeepExpr);
                    self.bump();
                    self.bump();
                    // `place max= expr` (01:506): an `if` or `match` value too.
                    if matches!(self.peek(), T::If | T::Match) {
                        self.expr();
                    } else {
                        self.expr_at(Level::Range);
                    }
                    self.finish();
                    continue;
                }
                if word == Contextual::By && min <= Level::Range {
                    self.start_at(at, NodeKind::RangeExpr);
                    self.bump();
                    self.expr_at(Level::BitOr);
                    self.finish();
                    continue;
                }
            }
            let Some(op) = self.binary_here() else { break };
            if op.level < min {
                break;
            }
            match op.assoc {
                Assoc::Chain => {
                    self.start_at(at, NodeKind::CompareExpr);
                    while self.binary_here().is_some_and(|b| b.assoc == Assoc::Chain) {
                        self.bump_compare_op();
                        self.expr_at(Level::Range);
                    }
                    self.finish();
                }
                Assoc::Left | Assoc::Right => {
                    self.start_at(at, NodeKind::BinExpr);
                    self.bump();
                    let rhs = match (op.assoc, op.level) {
                        (_, Level::Pow) => Level::Unary,
                        (Assoc::Right, l) => l,
                        (_, l) => l.tighter(),
                    };
                    self.expr_at(rhs);
                    self.finish();
                }
            }
        }
    }

    /// The binary operator here: `ops::BINARY`'s, or `not in`.
    fn binary_here(&self) -> Option<ops::Binary> {
        if self.no_gt && matches!(self.peek(), T::Gt | T::Shr | T::GtEq) {
            return None;
        }
        if self.at(T::Not) && self.nth(1) == T::In {
            return ops::binary(T::In);
        }
        // `a || b` and `a && b` are not two operators: the doubled forms
        // are left to the foreign-syntax diagnostics.
        if matches!(self.peek(), T::Pipe | T::Amp) && self.nth(1) == self.peek() && self.glued(1) {
            return None;
        }
        ops::binary(self.peek())
    }

    fn bump_compare_op(&mut self) {
        if self.at(T::Not) {
            self.bump();
        }
        self.bump();
    }

    fn contextual(&self) -> Option<Contextual> {
        if self.at(T::Ident) {
            contextual(self.text())
        } else {
            None
        }
    }

    /// After a range's low end: `..` or `..=`, the high end if one is
    /// written, and `by step`.
    fn range_rest(&mut self) {
        let inclusive = self.at(T::DotDotEq);
        self.bump();
        if begins_operand(self.peek()) {
            self.expr_at(Level::BitOr);
        } else if inclusive {
            self.recover(codes::EXPECTED_EXPR, "the end of the range after `..=`");
        }
        if self.contextual() == Some(Contextual::By) {
            self.bump();
            self.expr_at(Level::BitOr);
        }
    }

    /// A prefix operator and its operand, a range with no low end, or a
    /// postfix expression.
    fn operand(&mut self, min: Level) {
        if self.op_value_ahead() {
            return self.postfix();
        }
        if self.at(T::Mut) {
            self.start(NodeKind::MutExpr);
            self.bump();
            self.postfix();
            return self.finish();
        }
        // `else: return None`: a branch that leaves.
        if matches!(self.peek(), T::Return | T::Break | T::Continue) {
            return self.simple();
        }
        if let Some(level) =
            ops::prefix(self.peek()).filter(|_| !(self.at(T::Not) && self.nth(1) == T::In))
        {
            self.start(NodeKind::PrefixExpr);
            self.bump();
            self.expr_at(level.max(min.min(Level::Unary)));
            self.finish();
            return;
        }
        if ops::RANGE.contains(&self.peek()) && min <= Level::Range {
            self.start(NodeKind::RangeExpr);
            self.range_rest();
            self.finish();
            return;
        }
        self.postfix();
    }

    fn postfix(&mut self) {
        let at = self.checkpoint();
        self.primary();
        loop {
            let node = match self.peek() {
                T::LParen => {
                    self.start_at(at, NodeKind::CallExpr);
                    self.list(NodeKind::ArgList, T::RParen, "the argument list", Self::arg);
                    self.finish();
                    continue;
                }
                T::LBracket => {
                    self.start_at(at, NodeKind::IndexExpr);
                    self.list(NodeKind::IndexList, T::RBracket, "the index", |p| p.expr());
                    self.finish();
                    continue;
                }
                T::Dot => NodeKind::FieldExpr,
                T::Question => NodeKind::TryExpr,
                // `x !in xs` is a foreign `not in`, not an unwrap.
                T::Bang if !(self.nth(1) == T::In && self.glued(1)) => NodeKind::UnwrapExpr,
                T::Ident
                    if self.contextual() == Some(Contextual::With)
                        && self.nth(1) == T::Ident
                        && self.nth(2) == T::Colon =>
                {
                    self.start_at(at, NodeKind::WithExpr);
                    self.bump();
                    loop {
                        self.start(NodeKind::RecordField);
                        self.bump();
                        self.bump();
                        self.expr();
                        self.finish();
                        if !(self.at(T::Comma)
                            && self.nth(1) == T::Ident
                            && self.nth(2) == T::Colon)
                        {
                            break;
                        }
                        self.bump();
                    }
                    self.finish();
                    continue;
                }
                T::Ident if self.contextual() == Some(Contextual::Is) => {
                    self.start_at(at, NodeKind::IsExpr);
                    self.bump();
                    self.pattern();
                    self.finish();
                    continue;
                }
                _ => break,
            };
            self.start_at(at, node);
            self.bump();
            if node == NodeKind::FieldExpr {
                if matches!(
                    self.peek(),
                    T::Ident | T::Int | T::In | T::Type | T::Domain | T::Use
                ) {
                    self.bump();
                } else {
                    self.recover(codes::EXPECTED_TOKEN, "a field name after `.`");
                }
            }
            self.finish();
        }
    }

    pub(super) fn arg(&mut self) {
        self.start(NodeKind::Arg);
        if self.nth(1) == T::Colon && (self.at(T::Ident) || self.peek().spelling().is_some()) {
            self.bump();
            self.bump();
            // `f(by:)` passes the variable `by`.
            if matches!(self.peek(), T::Comma | T::RParen) {
                return self.finish();
            }
        }
        self.eat(T::Mut);
        let at = self.checkpoint();
        self.expr();
        if self.at(T::For) {
            self.start_at(at, NodeKind::GeneratorExpr);
            self.clauses();
            self.finish();
        }
        self.finish();
    }

    /// `for p in xs if c`, one or more: a comprehension's or generator's.
    fn clauses(&mut self) {
        while self.at(T::For) {
            self.start(NodeKind::CompClause);
            self.bump();
            self.pattern();
            if self.want(T::In, "`in` and what to walk") {
                self.expr_at(Level::Expr);
            }
            while self.eat(T::If) {
                self.expr_at(Level::Expr);
            }
            self.finish();
        }
    }

    /// An operator alone where a value goes: `fold(+, xs)`, `(+)`.
    fn op_value_ahead(&self) -> bool {
        let k = self.peek();
        (ops::binary(k).is_some() || ops::prefix(k).is_some())
            && !matches!(k, T::Not)
            && matches!(self.nth(1), T::Comma | T::RParen)
    }

    /// `Name<..>` reads as type arguments when they close and what follows
    /// can follow a value. A lowercase fn takes them only before a call,
    /// and only when no argument is a bare lowercase name, so `a < b > (c)`
    /// stays a comparison.
    fn type_args_ahead(&self, upper: bool) -> bool {
        let (mut depth, mut nest) = (0i32, 0i32);
        let (mut seg, mut lower) = (0usize, false);
        for n in 0.. {
            let k = self.nth(n);
            let at_top = depth == 1 && nest == 0;
            match k {
                T::Lt => depth += 1,
                T::Gt => depth -= 1,
                T::Shr => depth -= 2,
                T::LParen | T::LBracket => nest += 1,
                T::RParen | T::RBracket => nest -= 1,
                T::Newline | T::Indent | T::Dedent | T::Eof | T::Semi | T::FatArrow => {
                    return false
                }
                T::Eq | T::EqEq | T::NotEq | T::LtEq | T::GtEq | T::And | T::Or | T::OpEq
                    if nest == 0 =>
                {
                    return false
                }
                T::Colon if nest == 0 => return false,
                _ => {}
            }
            if nest < 0 {
                return false;
            }
            if at_top && matches!(k, T::Comma | T::Gt | T::Shr) {
                if seg == 1 && lower {
                    return false;
                }
                seg = 0;
            } else if n > 0 && at_top {
                seg += 1;
                lower = k == T::Ident && self.nth_lower(n);
            }
            if depth < 0 {
                return false;
            }
            if depth == 0 && n > 0 {
                let next = self.nth(n + 1);
                return if upper {
                    matches!(
                        next,
                        T::LParen
                            | T::Dot
                            | T::ColonColon
                            | T::Comma
                            | T::RParen
                            | T::RBracket
                            | T::RBrace
                            | T::Colon
                            | T::If
                            | T::Newline
                            | T::Eof
                            | T::Dedent
                            | T::Semi
                            | T::Eq
                    )
                } else {
                    next == T::LParen
                };
            }
        }
        false
    }

    fn nth_lower(&self, n: usize) -> bool {
        self.nth_index(n).is_some_and(|i| {
            self.tokens[i]
                .span
                .text(self.src)
                .starts_with(|c: char| c.is_lowercase())
        })
    }

    fn primary(&mut self) {
        match self.peek() {
            T::Int | T::Float | T::Str | T::RawStr | T::Char | T::True | T::False => {
                self.start(NodeKind::Literal);
                self.bump();
                self.finish();
            }
            T::Ident | T::SelfKw => {
                let upper = self.text().starts_with(|c: char| c.is_uppercase());
                let at = self.checkpoint();
                self.start(NodeKind::NameRef);
                self.bump();
                self.finish();
                if self.at(T::Lt) && self.type_args_ahead(upper) {
                    self.start_at(at, NodeKind::GenericExpr);
                    self.generic_args();
                    self.finish();
                }
            }
            k if self.op_value_ahead() => {
                debug_assert!(ops::binary(k).is_some() || ops::prefix(k).is_some());
                self.start(NodeKind::OpValue);
                self.bump();
                self.finish();
            }
            T::At
                if matches!(
                    self.nth_word(1),
                    Some(Contextual::Intrinsic | Contextual::Cpp)
                ) =>
            {
                self.start(NodeKind::IntrinsicExpr);
                self.bump();
                self.bump();
                if self.want(T::LParen, "`(` and a string") {
                    self.want(T::Str, "a string");
                    self.want(T::RParen, "`)`");
                }
                self.finish();
            }
            T::LBrace => self.block_expr(),
            T::Dot if matches!(self.nth(1), T::Int | T::Ident) => {
                self.start(NodeKind::FieldFn);
                self.bump();
                self.bump();
                self.finish();
            }
            T::LParen => self.paren(),
            T::LBracket => self.bracket(),
            _ => self.recover(codes::EXPECTED_EXPR, "an expression"),
        }
    }

    /// `{ s; s; value }`: statements, then the block's value.
    fn block_expr(&mut self) {
        self.start(NodeKind::BlockExpr);
        self.bump();
        while !matches!(self.peek(), T::RBrace | T::Newline | T::Eof | T::Dedent) {
            if self.at(T::Domain) {
                let at = self.checkpoint();
                self.domain_decl(at);
            } else {
                self.simple();
            }
            if !self.eat(T::Semi) {
                break;
            }
        }
        self.want(T::RBrace, "`}` to close the block");
        self.finish();
    }

    /// `[a, b]`, `[k: v]`, `[:]`, `[body for p in xs]`, `[n] value`, or a
    /// table `[D] d => body`.
    fn bracket(&mut self) {
        if self.table_ahead() {
            return self.table();
        }
        let at = self.checkpoint();
        self.bump();
        if self.at(T::Colon) && self.nth(1) == T::RBracket {
            self.bump();
            self.bump();
            return self.wrap(at, NodeKind::MapExpr);
        }
        if self.eat(T::RBracket) {
            return self.wrap(at, NodeKind::ListExpr);
        }
        let first = self.checkpoint();
        self.expr();
        if self.at(T::For) {
            self.clauses();
            self.want(T::RBracket, "`]` to close the comprehension");
            return self.wrap(at, NodeKind::CompExpr);
        }
        if self.eat(T::Colon) {
            self.expr();
            self.wrap(first, NodeKind::MapEntry);
            while self.eat(T::Comma) && !self.at(T::RBracket) {
                let entry = self.checkpoint();
                self.expr();
                if self.want(T::Colon, "`:` between a key and its value") {
                    self.expr();
                }
                self.wrap(entry, NodeKind::MapEntry);
            }
            self.want(T::RBracket, "`]` to close the map");
            return self.wrap(at, NodeKind::MapExpr);
        }
        let comma = self.at(T::Comma);
        while self.eat(T::Comma) && !self.at(T::RBracket) {
            self.expr();
        }
        let closed = self.want(T::RBracket, "`]` to close the list");
        self.wrap(at, NodeKind::ListExpr);
        let fill = matches!(
            self.peek(),
            T::Int | T::Float | T::Str | T::Char | T::True | T::False | T::Minus | T::LParen
        ) || (self.at(T::Ident) && self.word() != Some(Contextual::Where));
        if closed && !comma && fill {
            self.expr();
            self.wrap(at, NodeKind::TableFill);
        }
    }

    fn wrap(&mut self, at: Checkpoint, kind: NodeKind) {
        self.start_at(at, kind);
        self.finish();
    }

    /// `[..]` then a name or `(..)`, then `=>`.
    fn table_ahead(&self) -> bool {
        let mut depth = 0usize;
        for n in 0.. {
            match self.nth(n) {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return match self.nth(n + 1) {
                            T::Ident => self.nth(n + 2) == T::FatArrow,
                            T::LParen => self.paren_then_arrow(n + 1),
                            _ => false,
                        };
                    }
                }
                T::Newline | T::Eof => return false,
                _ => {}
            }
        }
        false
    }

    fn paren_then_arrow(&self, from: usize) -> bool {
        let mut depth = 0usize;
        for n in from.. {
            match self.nth(n) {
                T::LParen | T::LBracket => depth += 1,
                T::RParen | T::RBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return self.nth(n + 1) == T::FatArrow;
                    }
                }
                T::Newline | T::Eof => return false,
                _ => {}
            }
        }
        false
    }

    fn table(&mut self) {
        self.start(NodeKind::TableExpr);
        self.list(NodeKind::Dims, T::RBracket, "the table's domains", |p| {
            p.targ()
        });
        if self.at(T::LParen) {
            self.list(
                NodeKind::ParamList,
                T::RParen,
                "the table's parameters",
                |p| p.pattern(),
            );
        } else {
            self.start(NodeKind::ParamList);
            self.pattern();
            self.finish();
        }
        if self.want(T::FatArrow, "`=>` before the table's body") {
            self.arm_body();
        }
        self.finish();
    }

    /// Whether the parentheses at the cursor hold a record: a `name:` or a
    /// `without` at their top level. One pass to the closing `)`.
    fn record_ahead(&self) -> bool {
        let mut toks = (self.tokens[self.pos..].iter())
            .filter(|t| !t.kind.is_trivia())
            .skip(1)
            .peekable();
        let (mut depth, mut start) = (0usize, true);
        while let Some(t) = toks.next() {
            match t.kind {
                T::Eof => return false,
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace if depth == 0 => return false,
                T::RParen | T::RBracket | T::RBrace => depth -= 1,
                T::Comma if depth == 0 => {
                    start = true;
                    continue;
                }
                T::Ident if depth == 0 => {
                    let word = crate::grammar::contextual(t.span.text(self.src));
                    let named = start && toks.peek().is_some_and(|n| n.kind == T::Colon);
                    if named || word == Some(Contextual::Without) {
                        return true;
                    }
                }
                _ => {}
            }
            start = false;
        }
        false
    }

    /// `(e)`, a tuple `()`, `(a,)`, `(a, b)`, or a record `(x: 1, y:)`,
    /// `(..a, to: u)`, `(to: e.v, ..e without (u, v))` (R12.11.4).
    fn paren(&mut self) {
        let mut elems = 0;
        let mut comma = false;
        let mut record = self.record_ahead();
        let rest = record;
        let outer = std::mem::replace(&mut self.no_gt, false);
        self.list(NodeKind::TupleExpr, T::RParen, "the parentheses", |p| {
            elems += 1;
            if rest && p.at(T::DotDot) {
                p.start(NodeKind::RecordRest);
                p.bump();
                p.expr_at(Level::BitOr);
                if p.word() == Some(Contextual::Without) {
                    p.row_without();
                }
                p.finish();
            } else if p.at(T::Ident) && p.nth(1) == T::Colon {
                record = true;
                p.start(NodeKind::RecordField);
                p.bump();
                p.bump();
                if !matches!(p.peek(), T::Comma | T::RParen) {
                    p.expr();
                }
                p.finish();
            } else {
                p.expr();
            }
            comma |= p.at(T::Comma);
        });
        self.no_gt = outer;
        if record {
            self.b.relabel_last(NodeKind::RecordExpr);
        } else if elems == 1 && !comma {
            // Re-label: a single element with no comma is a parenthesized
            // expression. The tuple node is replaced in place.
            self.b.relabel_last(NodeKind::ParenExpr);
        }
    }
}

pub(super) fn begins_pattern(kind: TokenKind) -> bool {
    matches!(
        kind,
        T::Ident
            | T::LParen
            | T::Int
            | T::Float
            | T::Str
            | T::RawStr
            | T::Char
            | T::True
            | T::False
            | T::Minus
    )
}
