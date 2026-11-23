//! `map_contains_then_index` (L0594): `if k in m:` whose branch reads `m[k]`
//! looks the key up twice and leaves an index that traps once an edit moves
//! it out from under the test; `if let v = m.get(k):` looks it up once and
//! binds the value the branch reads. The guard form, `if k not in m: return`
//! then `m[k]` below it, becomes `let v = m.get(k) else: return`.

use crate::walk::{walk_stmt, Bind, Visitor};
use crate::{File, Finding, MAP_CONTAINS_THEN_INDEX};
use cpl_diag::{Diagnostic, Fix, Span};
use cpl_syntax::ast::{Block, CmpOp, Cond, Expr, ExprKind, PatternKind, Stmt, StmtKind};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    struct Tests<'a> {
        src: &'a str,
        base: u32,
        fns: Vec<(String, &'a str)>,
        /// Names earlier fixes bind, so nested fixes never shadow each other.
        taken: Vec<String>,
        out: Vec<Finding>,
    }
    impl Visitor for Tests<'_> {
        fn expr(&mut self, expr: &Expr) {
            if let ExprKind::If { branches, .. } = &expr.kind {
                for (cond, block) in branches {
                    self.branch(cond, block);
                }
            }
        }
        fn stmts(&mut self, stmts: &[Stmt]) {
            for (i, stmt) in stmts.iter().enumerate() {
                self.guard(stmt, &stmts[i + 1..]);
            }
        }
    }
    impl<'a> Tests<'a> {
        fn text(&self, span: Span) -> &'a str {
            span.shift(self.base).text(self.src)
        }
        fn branch(&mut self, cond: &Cond, block: &Block) {
            let Cond::Expr(test) = cond else {
                return;
            };
            let Some((key, map)) = self.membership(test, CmpOp::In) else {
                return;
            };
            self.lookup(test.span, key, map, &block.stmts, None);
        }
        /// `if k not in m: return` (or `break`, `continue`): the statements
        /// after it may read `m[k]`.
        fn guard(&mut self, stmt: &Stmt, after: &[Stmt]) {
            let StmtKind::Expr(e) = &stmt.kind else {
                return;
            };
            let ExprKind::If {
                branches,
                otherwise: None,
            } = &e.kind
            else {
                return;
            };
            let [(Cond::Expr(test), body)] = branches.as_slice() else {
                return;
            };
            let [exit] = body.stmts.as_slice() else {
                return;
            };
            if !matches!(
                exit.kind,
                StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_)
            ) {
                return;
            }
            let Some((key, map)) = self.membership(test, CmpOp::NotIn) else {
                return;
            };
            // The block's span runs on to the next line's indent; the guard
            // ends with its exit.
            let guard = Span::new(stmt.span.start, exit.span.end);
            let exit = self.text(exit.span).to_string();
            self.lookup(guard, key, map, after, Some(&exit));
        }
        /// `k in m` (or `k not in m`) with `k` a path and `m` a map name.
        fn membership(&self, test: &Expr, want: CmpOp) -> Option<(&'a str, &'a str)> {
            let ExprKind::Compare { first: lhs, rest } = &test.kind else {
                return None;
            };
            let [(op, rhs)] = rest.as_slice() else {
                return None;
            };
            let (key, map) = (self.text(lhs.span), self.text(rhs.span));
            (*op == want && is_path(key) && is_name(map) && declares_map(self.src, map))
                .then_some((key, map))
        }
        /// The `m[k]` reads in `stmts` become one `m.get(k)` lookup that
        /// replaces `test`: an `if let` test, or with `exit` a `let .. else:`.
        fn lookup(&mut self, test: Span, key: &str, map: &str, stmts: &[Stmt], exit: Option<&str>) {
            let mut uses = Uses {
                src: self.src,
                base: self.base,
                key,
                map,
                fns: &self.fns,
                found: Vec::new(),
                spoiled: false,
            };
            for stmt in stmts {
                walk_stmt(&mut uses, stmt);
            }
            if uses.spoiled || uses.found.is_empty() {
                return;
            }
            let found = uses.found;
            let (src, base) = (self.src, self.base);
            let bound = stmts
                .first()
                .filter(|_| found.len() == 1)
                .and_then(|first| own_line_let(src, base, first, found[0]));
            let name = match &bound {
                Some((name, _)) => name.clone(),
                None => match ["v", "got", "found", "entry", "stored"]
                    .into_iter()
                    .find(|n| !has_word(src, n) && !self.taken.iter().any(|t| t == n))
                {
                    Some(n) => n.to_string(),
                    None => return,
                },
            };
            self.taken.push(name.clone());
            let lookup = format!("let {name} = {map}.get({key})");
            let (written, message) = match exit {
                Some(exit) => (
                    format!("{lookup} else: {exit}"),
                    format!("`{key} not in {map}` exits, then `{map}[{key}]` looks it up again"),
                ),
                None => (
                    format!("if {lookup}:"),
                    format!("`{key} in {map}` is tested, then `{map}[{key}]` looks it up again"),
                ),
            };
            let replaced = match exit {
                Some(_) => written.clone(),
                None => lookup.clone(),
            };
            let mut edits = vec![(test.shift(base), replaced)];
            match bound {
                Some((_, line)) => edits.push((line, String::new())),
                None => edits.extend(found.iter().map(|s| (s.shift(base), name.clone()))),
            }
            let mut diag = Diagnostic::warning(MAP_CONTAINS_THEN_INDEX.code, test.shift(base), message)
                .with_label(found[0].shift(base), "the second lookup")
                .with_help(format!(
                    "write `{written}` and read `{name}`: one lookup, and no `{map}[{key}]` left to trap when an edit moves it past the test"
                ));
            diag.fixes.push(Fix {
                title: format!("use `{written}`"),
                edits,
            });
            self.out.push(Finding {
                lint: &MAP_CONTAINS_THEN_INDEX,
                diag,
            });
        }
    }
    struct Fns<'a> {
        src: &'a str,
        base: u32,
        out: Vec<(String, &'a str)>,
    }
    impl Visitor for Fns<'_> {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Fn(decl) = &stmt.kind {
                let body = stmt.span.shift(self.base).text(self.src);
                self.out.push((decl.name.text.clone(), body));
            }
        }
    }
    let mut fns = Fns {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut fns, |v, base| v.base = base);
    let fns = fns.out;
    let mut v = Tests {
        src: file.src,
        base: 0,
        fns,
        taken: Vec::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

/// The `m[k]` reads in a branch. A write to `m`, a method call on `m` or on
/// `m[k]` (which may change the value in place), a call to a function of
/// this file whose body names `m` (it may change `m`) or a new binding of `m` or `k` spoils the
/// rewrite: the bound value would no longer be the one each `m[k]` reads.
struct Uses<'a> {
    src: &'a str,
    base: u32,
    key: &'a str,
    map: &'a str,
    fns: &'a [(String, &'a str)],
    found: Vec<Span>,
    spoiled: bool,
}

impl Uses<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }
    fn is_read(&self, expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Index { base, indices }
            if self.text(base.span) == self.map
                && matches!(indices.as_slice(), [i] if self.text(i.span) == self.key))
    }
    fn root<'e>(&self, mut expr: &'e Expr) -> &'e Expr {
        loop {
            match &expr.kind {
                ExprKind::Index { base, .. } | ExprKind::Field { base, .. } => expr = base,
                _ => return expr,
            }
        }
    }
}

impl Visitor for Uses<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            let root = self.text(self.root(target).span);
            if root == self.map || root == self.key.split('.').next().unwrap_or("") {
                self.spoiled = true;
            }
        }
    }
    fn expr(&mut self, expr: &Expr) {
        if self.is_read(expr) {
            self.found.push(expr.span);
            return;
        }
        let ExprKind::Call { callee, .. } = &expr.kind else {
            return;
        };
        match &callee.kind {
            ExprKind::Field { base, .. } => {
                let mut on = base.as_ref();
                while let ExprKind::Field { base, .. } = &on.kind {
                    on = base;
                }
                if self.text(on.span) == self.map || self.is_read(on) {
                    self.spoiled = true;
                }
            }
            ExprKind::Name(_) => {
                let called = self.text(callee.span);
                if self
                    .fns
                    .iter()
                    .any(|(name, body)| name == called && has_word(body, self.map))
                {
                    self.spoiled = true;
                }
            }
            _ => {}
        }
    }
    fn bind(&mut self, name: &str, _span: Span, _how: Bind) {
        if name == self.map || self.key.split('.').next() == Some(name) {
            self.spoiled = true;
        }
    }
}

/// `let x = m[k]` alone on its line as the branch's first statement: its
/// name and the whole line, which the fix deletes.
fn own_line_let(src: &str, base: u32, first: &Stmt, read: Span) -> Option<(String, Span)> {
    let StmtKind::Let {
        mutable: false,
        pat,
        ty: None,
        init: Some(init),
        otherwise: None,
        guard: None,
    } = &first.kind
    else {
        return None;
    };
    let PatternKind::Name(name) = &pat.kind else {
        return None;
    };
    if init.span != read {
        return None;
    }
    let (start, end) = (
        (base + first.span.start) as usize,
        (base + first.span.end) as usize,
    );
    let line = src[..start].rfind('\n').map_or(0, |i| i + 1);
    let own = src[line..start].trim().is_empty() && src[end..].starts_with('\n');
    own.then(|| (name.to_string(), Span::new(line as u32, end as u32 + 1)))
}

fn is_name(s: &str) -> bool {
    s.chars()
        .next()
        .is_some_and(|c| c.is_lowercase() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn is_path(s: &str) -> bool {
    !s.is_empty() && s.split('.').all(is_name)
}

fn word_at(src: &str, at: usize, len: usize) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    !src[..at].chars().next_back().is_some_and(ident)
        && !src[at + len..].chars().next().is_some_and(ident)
}

fn has_word(src: &str, word: &str) -> bool {
    src.match_indices(word)
        .any(|(at, _)| word_at(src, at, word.len()))
}

/// Whether `name` is declared or constructed as a `Map` or `SortedMap`, the
/// maps whose `get` returns `?V` (a `Counter`'s `get` returns its default).
fn declares_map(src: &str, name: &str) -> bool {
    src.match_indices(name).any(|(at, _)| {
        if !word_at(src, at, name.len()) {
            return false;
        }
        let rest = src[at + name.len()..].trim_start();
        let Some(rest) = rest.strip_prefix('=').or_else(|| rest.strip_prefix(':')) else {
            return false;
        };
        if rest.starts_with('=') {
            return false;
        }
        let rest = rest.trim_start();
        ["Map<", "Map(", "SortedMap<", "SortedMap("]
            .iter()
            .any(|p| rest.starts_with(p))
    })
}
