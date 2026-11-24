//! Comprehension and generator clauses: `for pat in iter`, then filters.

use super::*;

impl<'a> Parser<'a> {
    /// Parses one or more `for pat in iter [if cond]` (or comma-separated
    /// `for a in A, b in B if cond`, mirroring `for` statements).
    pub(super) fn clauses(&mut self) -> Vec<Clause> {
        let mut clauses = Vec::new();
        while self.eat(T::For) {
            loop {
                let pat = self.pattern();
                self.expect(T::In, "`in` after the comprehension variable");
                // Stop before `if`, which here is the filter.
                let iter = if self.at(T::Question) {
                    self.optional_type_iter()
                } else {
                    self.coalesce()
                };
                // Comprehension and generator filters use `if` only.
                let mut conds = Vec::new();
                while self.eat(T::If)
                    || (self.at_ident("where") && {
                        let span = self.span();
                        self.error(
                            Diagnostic::error(
                                codes::REMOVED_SYNTAX,
                                span,
                                "comprehension filters use `if`, not `where`",
                            )
                            .with_help("use `if` to filter a comprehension or generator")
                            .with_fix(
                                "replace `where` with `if`",
                                span,
                                "if",
                            ),
                        );
                        self.bump();
                        true
                    })
                {
                    conds.push(self.filter_cond());
                }
                clauses.push(Clause { pat, iter, conds });
                if self.at(T::Comma) && self.comma_starts_clause() {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        clauses
    }

    /// True if a `,` inside a comprehension is followed by `pat in ...`.
    pub(super) fn comma_starts_clause(&self) -> bool {
        let mut i = 1;
        let mut depth = 0u32;
        loop {
            match self.peek_n(i) {
                T::LParen | T::LBracket => depth += 1,
                T::RParen | T::RBracket => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                T::In if depth == 0 => return true,
                T::Comma | T::Colon | T::Eq | T::For | T::If | T::Newline | T::Eof
                    if depth == 0 =>
                {
                    return false;
                }
                T::Eof => return false,
                _ => {}
            }
            i += 1;
        }
    }

    /// `let pat = value`, then `and`, or a `,` that does not start the
    /// next clause, with a test or another `let`, as in an `if let` header.
    pub(super) fn let_guard(&mut self) -> Cond {
        self.bump();
        let pat = self.pattern();
        self.expect(T::Eq, "`=` after the pattern in `if let`");
        let value = self.not();
        let and = if self.eat(T::And) || self.c_and() {
            Some(Box::new(if self.at(T::Let) {
                self.let_guard()
            } else {
                Cond::Expr(self.and())
            }))
        } else if self.eat_guard_comma() {
            Some(Box::new(self.more_clauses()))
        } else if self.at_ident("where") {
            let span = self.span();
            self.error(
                Diagnostic::error(
                    codes::REMOVED_SYNTAX,
                    span,
                    "use `and`, not `where`, after `if let`",
                )
                .with_help("write `if let pat = value and cond`")
                .with_fix("replace `where` with `and`", span, "and"),
            );
            self.bump();
            Some(Box::new(Cond::Expr(self.coalesce())))
        } else {
            None
        };
        Cond::Let { pat, value, and }
    }

    /// A comprehension's `if`: what an `if` header takes, `let` binders and
    /// commas included (R01.8.1); `, y in ys` starts the next clause.
    pub(super) fn filter_cond(&mut self) -> Cond {
        let outer = std::mem::replace(&mut self.for_filter, true);
        let cond = if self.at(T::Let) {
            self.let_guard()
        } else {
            let test = self.coalesce();
            if self.eat_guard_comma() {
                Cond::Guard {
                    test,
                    and: Box::new(self.more_clauses()),
                }
            } else {
                Cond::Expr(test)
            }
        };
        self.for_filter = outer;
        cond
    }
}
