//! `unproven_list_index`: `xs[i]` on a `List` is bounds-checked on every
//! access unless the compiler can see `i` is in range. Inside two or more
//! nested loops that check is paid millions of times.
//!
//! An index is proven when it is the variable of an enclosing
//! `for i in 0..xs.len` or `for i in xs.indices` over the same list, or when
//! an enclosing `while` or `if` test, or the left side of an `and`, tests it:
//! `i < xs.len and xs[i] > 0`, `if not xs.is_empty: xs[xs.len - 1]`.
//! An access in the `Proven*` fact table ([`crate::ProvenIndex`], which the
//! editor shows too) is proven as well: `ys[mid]` in a binary search.

use super::shapes::flip;
use crate::walk::{unparen, Bind, Scope, Visitor};
use crate::{File, Finding, UNPROVEN_LIST_INDEX};
use cpl_check::Ty;
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{
    BinOp, CmpOp, Cond, Expr, ExprKind, Pattern, PatternKind, Stmt, StmtKind, UnOp,
};
use std::collections::{HashMap, HashSet};

/// How many loops deep an index has to be before it is worth a warning.
const HOT_DEPTH: usize = 2;

/// The index `xs.len - 1`, which a guard proves when it shows `xs` is not empty.
const LAST: &str = "len - 1";

fn is_list_ty(ty: &Ty) -> bool {
    matches!(ty, Ty::Named { name, .. } if name == "List")
}

/// `[]`, `[1, 2]`, `List<Int>()`.
pub(crate) fn is_list_value(expr: &Expr) -> bool {
    match &unparen(expr).kind {
        ExprKind::List(_) => true,
        ExprKind::Call { callee, .. } => match &callee.kind {
            ExprKind::Generic { base, .. } => {
                matches!(&base.kind, ExprKind::Name(n) if n == "List")
            }
            ExprKind::Name(name) => name == "List",
            _ => false,
        },
        _ => false,
    }
}

/// An open scope.
struct Open {
    scope: Scope,
    /// The `(index variable, list)` pairs it proves in range.
    proves: Vec<(String, String)>,
    /// The `if` or `while` test its proofs come from, where they do not hold yet.
    test: Option<Span>,
    /// The names it binds, and whether each is a `List` (`None`: only the
    /// checker's facts can say).
    names: HashMap<String, Option<bool>>,
}

struct Indexes<'a> {
    src: &'a str,
    facts: Option<&'a crate::Facts>,
    base: u32,
    /// The names bound at the top level, and whether each is a `List`.
    globals: HashMap<String, Option<bool>>,
    scopes: Vec<Open>,
    /// Whether the `let` being walked declares a `List`.
    binding_list: Option<bool>,
    /// Where in the file the `@test` fn being read ends: nothing before it is hot.
    test_ends: u32,
    /// The parameters of the `fn` about to be entered, and whether each is a `List`.
    list_params: HashMap<String, Option<bool>>,
    /// The right sides of `and`s, with the `(index, list)` pairs their left side proves.
    guards: Vec<(Span, Vec<(String, String)>)>,
    findings: Vec<Finding>,
}

impl Indexes<'_> {
    /// Whether the checker types the name declared at `span` as a `List`;
    /// `None` when it has no answer.
    fn declares_list(&self, span: Span) -> Option<bool> {
        let ty = self.facts?.place_type(span.shift(self.base))?;
        Some(is_list_ty(ty))
    }

    /// Loops open since the enclosing fn began.
    fn loop_depth(&self) -> usize {
        self.scopes
            .iter()
            .rev()
            .take_while(|open| open.scope != Scope::Fn)
            .filter(|open| open.scope == Scope::Loop)
            .count()
    }

    fn proven(&self, at: Span, index: &str, list: &str) -> bool {
        self.scopes
            .iter()
            .filter(|open| !open.test.is_some_and(|test| test.contains(at)))
            .any(|open| open.proves.iter().any(|(i, xs)| i == index && xs == list))
    }

    /// Whether an `and` whose right side holds `at` tests `index` against `list`.
    fn guarded(&self, at: Span, index: &str, list: &str) -> bool {
        self.guards.iter().any(|(span, facts)| {
            span.contains(at) && facts.iter().any(|(i, xs)| i == index && xs == list)
        })
    }

    /// Whether `name`, as seen here, is a `List`: the nearest binding
    /// decides, and `None` leaves it to the checker's type for the name.
    fn is_list(&self, name: &str) -> Option<bool> {
        self.scopes
            .iter()
            .rev()
            .find_map(|open| open.names.get(name))
            .or_else(|| self.globals.get(name))
            .copied()
            .unwrap_or(Some(false))
    }
}

/// `xs` in `xs.field`.
fn receiver<'a>(expr: &'a Expr, field: &str) -> Option<&'a str> {
    match &unparen(expr).kind {
        ExprKind::Field { base, name } if name.text == field => match &base.kind {
            ExprKind::Name(list) => Some(list),
            _ => None,
        },
        _ => None,
    }
}

/// `xs` in `xs.len`.
fn list_of_len(len: &Expr) -> Option<&str> {
    receiver(len, "len")
}

/// The list `iter` walks the indices of: `0..xs.len`, `xs.indices`, and
/// either one `.rev`.
fn indexed_list(iter: &Expr) -> Option<&str> {
    match &unparen(iter).kind {
        ExprKind::Field { base, name } if name.text == "rev" => indexed_list(base),
        ExprKind::Range {
            lo: Some(lo),
            hi: Some(hi),
            inclusive: false,
            step: None,
        } if matches!(lo.kind, ExprKind::Int(_)) => receiver(hi, "len"),
        _ => receiver(iter, "indices"),
    }
}

/// The `(index, list)` pairs `test` being true proves in range: `i < xs.len`
/// and `xs.len > i` prove `(i, xs)`; `xs.len > 0`, `xs.len >= 1`,
/// `xs.len != 0` and `not xs.is_empty` prove `(LAST, xs)`. Both sides of an
/// `and` count.
fn guard_facts(test: &Expr, out: &mut Vec<(String, String)>) {
    match &unparen(test).kind {
        ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            guard_facts(lhs, out);
            guard_facts(rhs, out);
        }
        ExprKind::Unary {
            op: UnOp::Not,
            expr,
        } => {
            if let Some(list) = receiver(expr, "is_empty") {
                out.push((LAST.to_string(), list.to_string()));
            }
        }
        ExprKind::Compare { first, rest } => {
            let mut lhs = &**first;
            for (op, rhs) in rest {
                let (op, small, len) = match (list_of_len(lhs), list_of_len(rhs)) {
                    (None, Some(list)) => (*op, lhs, list),
                    (Some(list), None) => (flip(*op), rhs, list),
                    _ => {
                        lhs = rhs;
                        continue;
                    }
                };
                match (op, &unparen(small).kind) {
                    (CmpOp::Lt, ExprKind::Name(index)) => {
                        out.push((index.clone(), len.to_string()))
                    }
                    (CmpOp::Lt, ExprKind::Int(0))
                    | (CmpOp::Le, ExprKind::Int(1))
                    | (CmpOp::Ne, ExprKind::Int(0)) => {
                        out.push((LAST.to_string(), len.to_string()))
                    }
                    _ => {}
                }
                lhs = rhs;
            }
        }
        _ => {}
    }
}

/// `xs.len - 1`.
fn is_last(index: &Expr, list: &str) -> bool {
    matches!(&unparen(index).kind, ExprKind::Binary { op: BinOp::Sub, lhs, rhs, .. }
        if list_of_len(lhs) == Some(list) && matches!(unparen(rhs).kind, ExprKind::Int(1)))
}

impl Visitor for Indexes<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.binding_list = None;
        match &stmt.kind {
            StmtKind::Let { pat, ty, init, .. } => {
                self.binding_list = match ty {
                    _ if !matches!(pat.kind, PatternKind::Name(_)) => None,
                    Some(_) => self.declares_list(pat.span),
                    None => init.as_ref().filter(|e| is_list_value(e)).map(|_| true),
                };
            }
            StmtKind::Fn(decl) => {
                // A `@test` is not what a judge times.
                if stmt.decorators.iter().any(|d| d.name.text == "test") {
                    self.test_ends = self.base + stmt.span.end;
                }
                self.list_params = decl
                    .params
                    .iter()
                    .map(|param| {
                        let list = param
                            .ty
                            .as_ref()
                            .and_then(|_| self.declares_list(param.name.span));
                        (param.name.text.clone(), list)
                    })
                    .collect();
            }
            _ => {}
        }
    }

    fn bind(&mut self, name: &str, _span: Span, how: Bind) {
        let list = match how {
            Bind::Let | Bind::Var => self.binding_list.take(),
            Bind::Param => self.list_params.get(name).copied().flatten(),
            Bind::Pattern => None,
            Bind::Const => Some(false),
        };
        let names = match self.scopes.last_mut() {
            Some(open) => &mut open.names,
            None => &mut self.globals,
        };
        names.insert(name.to_string(), list);
    }

    fn enter(&mut self, scope: Scope) {
        self.scopes.push(Open {
            scope,
            proves: Vec::new(),
            test: None,
            names: HashMap::new(),
        });
    }

    fn exit(&mut self, _scope: Scope) {
        self.scopes.pop();
    }

    fn for_loop(&mut self, pat: &Pattern, iter: &Expr) {
        if let (PatternKind::Name(index), Some(list)) = (&pat.kind, indexed_list(iter)) {
            if let Some(top) = self.scopes.last_mut() {
                top.proves.push((index.clone(), list.to_string()));
            }
        }
    }

    /// `while head < q.len:` walks a list that grows as it is read: the
    /// test is the bounds check.
    fn while_loop(&mut self, cond: &Cond) {
        self.if_branch(cond);
    }

    fn if_branch(&mut self, cond: &Cond) {
        if let (Cond::Expr(test), Some(top)) = (cond, self.scopes.last_mut()) {
            guard_facts(test, &mut top.proves);
            top.test = Some(test.span.shift(self.base));
        }
    }

    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } = &expr.kind
        {
            let mut facts = Vec::new();
            guard_facts(lhs, &mut facts);
            if !facts.is_empty() {
                self.guards.push((rhs.span.shift(self.base), facts));
            }
        }
        let ExprKind::Index { base, indices } = &expr.kind else {
            return;
        };
        let (ExprKind::Name(list), [index]) = (&base.kind, indices.as_slice()) else {
            return;
        };
        let known = self.is_list(list);
        if self.loop_depth() < HOT_DEPTH
            || known == Some(false)
            || self.base + expr.span.start < self.test_ends
        {
            return;
        }
        let at = index.span.shift(self.base);
        if is_last(index, list) && (self.proven(at, LAST, list) || self.guarded(at, LAST, list)) {
            return;
        }
        match &unparen(index).kind {
            ExprKind::Name(name) if self.proven(at, name, list) || self.guarded(at, name, list) => {
                return
            }
            // A slice, not an element.
            ExprKind::Range { .. } => return,
            _ => {}
        }
        let written = index.span.shift(self.base).text(self.src);
        let mut diag = Diagnostic::warning(
            UNPROVEN_LIST_INDEX.code,
            expr.span.shift(self.base),
            format!("`{list}[{written}]` is bounds-checked on every pass of a nested loop"),
        );
        if known.is_none() {
            diag = diag.with_label(base.span.shift(self.base), format!("`{list}` is a `List`"));
        }
        let diag = diag.with_help(format!(
            "loop over the values (`for x in {list}`) or the indices (`for i in 0..{list}.len`), or index an array by a domain: `[D]T` needs no check"
        ));
        self.findings.push(Finding {
            lint: &UNPROVEN_LIST_INDEX,
            diag,
        });
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut indexes = Indexes {
        src: file.src,
        facts: file.facts,
        base: 0,
        globals: HashMap::new(),
        scopes: Vec::new(),
        binding_list: None,
        test_ends: 0,
        list_params: HashMap::new(),
        guards: Vec::new(),
        findings: Vec::new(),
    };
    file.walk(&mut indexes, |indexes, base| indexes.base = base);
    if indexes.findings.is_empty() {
        return indexes.findings;
    }
    let proven: HashSet<(u32, u32)> = super::proven_indexes(file)
        .into_iter()
        .map(|p| p.access)
        .collect();
    indexes
        .findings
        .retain(|f| !proven.contains(&(f.diag.span.start, f.diag.span.end)));
    indexes.findings
}

#[cfg(test)]
mod tests {
    use crate::tests::{codes, only};

    #[test]
    fn unproven_list_index_wants_two_loops_and_an_unproven_index() {
        let nest = |index: &str| {
            format!("var ws: List<Int> = []\nws.push(read())\nvar total = 0\nfor i in 0..ws.len:\n    for j in ws.indices:\n        total += {index}\nprint(total)\n")
        };
        let found = only(&nest("ws[j + 1]"));
        assert_eq!(found.code.0, "L0414");
        assert_eq!(
            found.message,
            "`ws[j + 1]` is bounds-checked on every pass of a nested loop"
        );
        // Loop variables over the list's own indices are proven.
        assert_eq!(codes(&nest("ws[i] * ws[j]")), Vec::<&str>::new());
        // One loop is not hot, and an array indexed by a domain is not a list.
        let single =
            "var ws: List<Int> = []\nws.push(read())\nfor i in 0..3:\n    print(ws[i + 1])\n";
        assert_eq!(codes(single), Vec::<&str>::new());
        assert_eq!(codes(&nest("ws[i..j].len")), Vec::<&str>::new());
        // A name is judged by its nearest binding: a list in one fn does not make
        // an array of the same name in another a list.
        let shadowed = "fn f(seen: List<Bool>) -> Bool = seen[0]\ndomain V = 0..9\nvar seen: [V]Bool = false\nfor u in V:\n    for v in V:\n        seen[v] = seen[u]\nprint(f([true]))\n";
        assert!(!codes(shadowed).contains(&"L0414"));
        let inner = "domain V = 0..9\nvar ws: [V]Int = 0\nfn g():\n    var ws: List<Int> = []\n    ws.push(read())\n    for i in 0..3:\n        for j in 0..3:\n            print(ws[i + j])\ng()\nprint(ws)\n";
        assert!(codes(inner).contains(&"L0414"));
        // A fn starts its own count.
        let in_fn = "var ws: List<Int> = []\nws.push(read())\nfor i in 0..3:\n    for j in 0..3:\n        let f = k => ws[k]\n        print(f(i + j))\n";
        assert_eq!(codes(in_fn), ["L0414"]);
        // `while head < q.len` is the check, for a list that grows as it is read.
        let worklist = "var q: List<Int> = [0]\nfor root in 0..3:\n    var head = 0\n    while head < q.len:\n        let u = q[head]\n        head += 1\n        if u < 3: q.push(u + 1)\n";
        assert!(!codes(worklist).contains(&"L0414"));
        // A `@test` is not what a judge times; the code after it is.
        let test = "@test\nfn t():\n    let ws = [1, 2, 3]\n    for i in 0..2:\n        for j in 0..2:\n            assert ws[i + j] > 0\n";
        assert_eq!(codes(test), Vec::<&str>::new());
        let after = format!("{test}fn f(ws: List<Int>):\n    for i in 0..2:\n        for j in 0..2:\n            print(ws[i + j])\nf([1, 2, 3])\n");
        assert_eq!(codes(&after), ["L0414"]);
        // The left side of an `and` is the check for its right side.
        let guard = |test: &str| {
            format!("var ws: List<Int> = []\nws.push(read())\nvar total = 0\nfor i in 0..3:\n    for j in 0..3:\n        if {test}: total += 1\nprint(total)\n")
        };
        assert_eq!(codes(&guard("ws[j] > 0")), ["L0414"]);
        assert_eq!(
            codes(&guard("j < ws.len and ws[j] > 0")),
            Vec::<&str>::new()
        );
        assert_eq!(
            codes(&guard("ws.len > j and ws[j] > 0")),
            Vec::<&str>::new()
        );
        assert_eq!(
            codes(&guard("0 <= j < ws.len and ws[j] > 0")),
            Vec::<&str>::new()
        );
        let last = |test: &str| {
            format!("var ws: List<Int> = []\nws.push(read())\nfor i in 0..3:\n    for j in 0..3:\n        while {test} and ws[ws.len - 1] == 0: ws.pop()\n")
        };
        for test in [
            "not ws.is_empty",
            "ws.len > 0",
            "0 < ws.len",
            "ws.len >= 1",
            "ws.len != 0",
        ] {
            assert!(!codes(&last(test)).contains(&"L0414"), "{test}");
        }
        // Reversed index loops prove their index too.
        let rev = |iter: &str| {
            format!("var ws: List<Int> = []\nws.push(read())\nvar total = 0\nfor i in 0..3:\n    for j in {iter}:\n        total += ws[j]\nprint(total)\n")
        };
        assert_eq!(codes(&rev("(0..ws.len).rev")), Vec::<&str>::new());
        assert_eq!(codes(&rev("ws.indices.rev")), Vec::<&str>::new());
        // Any literal start is at least 0, and so is the outer `i in 0..3`;
        // `i - 5` may be negative.
        assert_eq!(codes(&rev("(1..ws.len).rev")), Vec::<&str>::new());
        assert_eq!(codes(&rev("2..ws.len")), Vec::<&str>::new());
        assert_eq!(codes(&rev("i..ws.len")), Vec::<&str>::new());
        assert_eq!(codes(&rev("(i - 5)..ws.len")), ["L0414"]);
        // An access the fact table proves: a binary search over a fixed list.
        let search = "let ys: List<Int> = read()\nvar hits = 0\nfor q in 0..3:\n    for t in 0..3:\n        var (lo, hi) = (0, ys.len)\n        while lo < hi:\n            let mid = (lo + hi) / 2\n            if ys[mid] < t: lo = mid + 1\n            else: hi = mid\n        hits += lo\nprint(hits)\n";
        assert!(!codes(search).contains(&"L0414"), "{:?}", codes(search));
        // So is an enclosing `if` or `while` test.
        let body = |test: &str, index: &str| {
            format!("var ws: List<Int> = []\nws.push(read())\nvar total = 0\nfor i in 0..3:\n    for j in 0..3:\n        if {test}:\n            total += {index}\nprint(total)\n")
        };
        assert_eq!(codes(&body("j < ws.len", "ws[j]")), Vec::<&str>::new());
        assert!(!codes(&body("not ws.is_empty", "ws[ws.len - 1]")).contains(&"L0414"));
        assert_eq!(codes(&body("j < ws.len", "ws[i]")), ["L0414"]);
        assert_eq!(codes(&body("j < ws.len or i > 0", "ws[j]")), ["L0414"]);
        let stack = "var st: List<Int> = [0]\nfor i in 0..3:\n    for j in 0..3:\n        while st.len > 0 and st[st.len - 1] > j: st.pop()\n        st.push(j)\n";
        assert!(!codes(stack).contains(&"L0414"));
        // Near misses: the guard must test the same index and list, before the index.
        assert_eq!(codes(&guard("ws[j] > 0 and j < ws.len")), ["L0414"]);
        assert_eq!(codes(&guard("i < ws.len and ws[j] > 0")), ["L0414"]);
        assert!(codes(&last("ws.len > 1 or true")).contains(&"L0414"));
        let other = "var ws: List<Int> = []\nvar vs: List<Int> = []\nws.push(read())\nfor i in 0..3:\n    for j in 0..3:\n        if j < vs.len and ws[j] > 0: print(j)\n";
        assert!(codes(other).contains(&"L0414"));
    }
}
