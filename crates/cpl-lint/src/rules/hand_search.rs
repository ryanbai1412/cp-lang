//! Walks newcomers spell by hand where std names them: the submasks of a
//! mask (`submask_loop`), a ternary search over integers
//! (`ternary_search_by_hand`), a palindrome test (`palindrome_by_hand`),
//! nearest smaller or greater positions (`monotone_stack_by_hand`), every
//! subset by its mask (`subset_masks_by_hand`), neighbouring pairs by index
//! (`adjacent_by_index`), the sort order of positions by value
//! (`argsort_loop`), a factorial table (`fact_tables_by_hand`), the top
//! `k` of a heap (`manual_heap_drain`), lists grouped by key
//! (`group_into_map`) and a sliding window sum (`sliding_sum_by_hand`).

use crate::walk::Visitor;
use crate::{
    File, Finding, ADJACENT_BY_INDEX, ARGSORT_LOOP, FACT_TABLES_BY_HAND, GROUP_INTO_MAP,
    MANUAL_HEAP_DRAIN, MONOTONE_STACK_BY_HAND, PALINDROME_BY_HAND, SLIDING_SUM_BY_HAND,
    SUBMASK_LOOP, SUBSET_MASKS_BY_HAND, TERNARY_SEARCH_BY_HAND,
};
use cpl_diag::Diagnostic;
use cpl_syntax::ast::*;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandSearch {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandSearch<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

/// `text` without comments and without spaces, except one between two words
/// (`if a[i]`, `for i in 0..n`), so a keyword never fuses with a name.
fn squash_code(text: &str) -> String {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut out = String::new();
    for line in text.lines() {
        let code = line.split("//").next().unwrap_or("");
        for c in code.chars().chain([' ']) {
            if c.is_whitespace() {
                if out.ends_with(word) {
                    out.push(' ');
                }
            } else {
                if out.ends_with(' ') && !(word(c) && out.trim_end().ends_with(word)) {
                    out.pop();
                }
                out.push(c);
            }
        }
    }
    out.trim_end().to_string()
}

fn ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// The std call for `st.len > 0 and a[st.last!] < a[i]` (squashed), the pop
/// condition of a monotone stack of positions into `a`, with what it finds,
/// the stack and the current position.
fn monotone_pop(c: &str) -> Option<(String, &'static str, &str, &str)> {
    let (nonempty, cmp) = c.split_once(" and ")?;
    let st = nonempty
        .strip_suffix(".len>0")
        .or_else(|| nonempty.strip_suffix(".len!=0"))
        .or_else(|| nonempty.strip_prefix("not ")?.strip_suffix(".is_empty"))?;
    let (a, rest) = cmp.split_once(&format!("[{st}.last!]"))?;
    let (op, rhs) = ["<=", ">=", "<", ">"]
        .iter()
        .find_map(|op| Some((*op, rest.strip_prefix(op)?)))?;
    let i = rhs.strip_prefix(&format!("{a}["))?.strip_suffix(']')?;
    if !(ident(st) && ident(a) && ident(i)) {
        return None;
    }
    let (call, what) = match op {
        "<" => ("next_smaller(by: x => -x)", "greater"),
        "<=" => (
            "next_smaller(by: x => -x, strict: false)",
            "greater or equal",
        ),
        ">" => ("next_smaller()", "smaller"),
        _ => ("next_smaller(strict: false)", "smaller or equal"),
    };
    Some((format!("{a}.{call}"), what, st, i))
}

/// Whether a `fn` in `src` pushes into `h`: a loop that calls it refills the
/// heap it pops.
fn pushed_in_fn(src: &str, h: &str) -> bool {
    let lines: Vec<&str> = src.lines().collect();
    let indent = |l: &str| l.len() - l.trim_start().len();
    lines.iter().enumerate().any(|(n, line)| {
        if !line.contains(&format!("{h}.push(")) {
            return false;
        }
        let mut depth = indent(line);
        lines[..n].iter().rev().any(|above| {
            if above.trim().is_empty() || indent(above) >= depth || depth == 0 {
                return false;
            }
            depth = indent(above);
            let head = above.trim_start();
            head.starts_with("fn ") || head.starts_with("pub fn ")
        })
    })
}

/// Whether squashed `body` assigns an element of `xs` (`xs[..] = ..`,
/// `xs[..] += ..`, `xs[..] min= ..`): a loop that does is a DP over neighbours,
/// not a walk over pairs.
fn writes(body: &str, xs: &str) -> bool {
    body.match_indices(&format!("{xs}[")).any(|(at, _)| {
        let rest = &body[at + xs.len() + 1..];
        let mut depth = 1;
        let Some(close) = rest.find(|c| {
            depth += match c {
                '[' => 1,
                ']' => -1,
                _ => 0,
            };
            depth == 0
        }) else {
            return false;
        };
        let after = rest[close + 1..].trim_start();
        let op = after.trim_start_matches(|c: char| c.is_alphabetic() || "+-*/%&|^<>".contains(c));
        op.starts_with('=') && !op.starts_with("==")
    })
}

/// `(x[i],i)` at the start of `s`, squashed: `(x, i)`.
fn value_index(s: &str) -> Option<(&str, &str)> {
    let (x, rest) = s.strip_prefix('(')?.split_once('[')?;
    let (i, rest) = rest.split_once(']')?;
    let ok = ident(x) && ident(i) && rest.starts_with(&format!(",{i})"));
    ok.then_some((x, i))
}

impl HandSearch<'_> {
    /// The pairs sorted by `sorted`: `(x, i, range)` when `recv` is a list of
    /// `(x[i], i)` built by a comprehension or a push loop over `range`.
    fn argsort(&self, recv: &str) -> Option<(String, String, String)> {
        let src = squash_code(self.src);
        let comp = |c: &str| {
            let (x, i) = value_index(c.strip_prefix('[')?)?;
            let rest = c.split_once(&format!("for {i} in "))?.1;
            let range = rest.split([']', ':']).next()?;
            Some((x.to_string(), i.to_string(), range.to_string()))
        };
        if recv.starts_with("[(") {
            return comp(recv);
        }
        if !ident(recv) {
            return None;
        }
        if let Some(at) = src.find(&format!("{recv}=[(")) {
            return comp(&src[at + recv.len() + 1..]);
        }
        let at = src.find(&format!("{recv}.push(("))?;
        let (x, i) = value_index(&src[at + recv.len() + 6..])?;
        let head = &src[..at];
        let start = head.rfind(&format!("for {i} in "))? + i.len() + 8;
        let range = head[start..].split(':').next()?;
        Some((x.to_string(), i.to_string(), range.to_string()))
    }
}

impl HandSearch<'_> {
    /// `span` widened to a `var ok = true` (or `false`) on the line above: the
    /// flag the loop sets is part of the same test.
    fn with_flag(&self, span: cpl_diag::Span) -> cpl_diag::Span {
        let head = &self.src[..span.start as usize];
        let above = head.trim_end_matches([' ', '\t']);
        let above = above.strip_suffix('\n').unwrap_or(above);
        let line_start = above.rfind('\n').map_or(0, |i| i + 1);
        let line = above[line_start..].split("//").next().unwrap_or("").trim();
        let flag =
            line.starts_with("var ") && (line.ends_with("= true") || line.ends_with("= false"));
        if !flag {
            return span;
        }
        let indent = above[line_start..].len() - above[line_start..].trim_start().len();
        cpl_diag::Span::new((line_start + indent) as u32, span.end)
    }

    /// `for mask in 0..(1 << n):` testing `(mask >> i) & 1` to pick `a[i]`.
    fn subset_masks(&mut self, stmt: &Stmt, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let m = text(pat.span);
        let it = text(iter.span);
        let Some(n) = it
            .strip_prefix("0..(1<<")
            .and_then(|r| r.strip_suffix(')'))
            .or_else(|| it.strip_prefix("0..1<<"))
        else {
            return;
        };
        let whole = text(body.span);
        let bit = [format!("({m}>>"), format!("{m}&(1<<"), format!("{m}.bit(")]
            .iter()
            .find_map(|head| {
                let at = whole.find(head.as_str())?;
                let pre = whole[..at].trim_end_matches(['(', ' ']);
                let picks = pre.ends_with("if") || pre.ends_with("and");
                let rest = &whole[at + head.len()..];
                let i = rest
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next()?;
                let unset = rest[i.len()..]
                    .trim_start_matches(".ord")
                    .trim_start_matches([')', '&', '1', ' '])
                    .starts_with("==0");
                (picks && !unset && ident(i)).then(|| i.to_string())
            });
        let Some(i) = bit else {
            return;
        };
        let idx = format!("[{i}]");
        let Some(xs) = whole.match_indices(&idx).find_map(|(at, _)| {
            let xs = whole[..at]
                .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?;
            ident(xs).then(|| xs.to_string())
        }) else {
            return;
        };
        if !ident(&m) || !ident(n) {
            return;
        }
        let diag = Diagnostic::warning(
            SUBSET_MASKS_BY_HAND.code,
            stmt.span.shift(self.base),
            format!("every subset of `{xs}` walked as a mask, its elements picked bit by bit"),
        )
        .with_help(format!(
            "write `for sub in {xs}.subsequences():` (`std.seq`): the same subsets, `sub.items` the chosen elements, and `sub.prune()` skips every extension of one; keep the mask where it is a DP index"
        ));
        self.out.push(Finding {
            lint: &SUBSET_MASKS_BY_HAND,
            diag,
        });
    }

    /// `for i in 1..=n: fact[i] = fact[i - 1] * i % P` (or `.push(..)`, or
    /// `* Z(i)` over a modular type).
    fn fact_table(&mut self, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let [only] = body.stmts.as_slice() else {
            return;
        };
        let it = text(iter.span);
        let Some(n) = it
            .strip_prefix("1..=")
            .or_else(|| it.strip_prefix("1.."))
            .or_else(|| it.strip_prefix("2..="))
        else {
            return;
        };
        if !ident(&i) {
            return;
        }
        let t = text(only.span);
        let Some(product) = t
            .split_once('=')
            .and_then(|(x, rhs)| {
                let x = x.strip_suffix(&format!("[{i}]"))?;
                ident(x).then_some(())?;
                rhs.strip_prefix(&format!("{x}[{i}-1]*"))
            })
            .or_else(|| {
                let (x, rest) = t.split_once(".push(")?;
                ident(x).then_some(())?;
                let rest = rest.strip_suffix(')')?;
                rest.strip_prefix(&format!("{x}[{i}-1]*"))
                    .or_else(|| rest.strip_prefix(&format!("{x}.last!*")))
            })
        else {
            return;
        };
        let (factor, modulus) = match product.split_once('%') {
            Some((f, m)) => (f, Some(m)),
            None => (product, None),
        };
        let ty = if factor == i {
            None
        } else {
            let Some(ty) = factor
                .strip_suffix(&format!("({i})"))
                .filter(|ty| ident(ty))
            else {
                return;
            };
            Some(ty)
        };
        let z = match (modulus, ty) {
            (Some(p), None) if p.chars().all(|c| c.is_ascii_digit() || c == '_') => {
                format!("Mod<{p}>")
            }
            (None, Some(ty)) => ty.to_string(),
            _ => return,
        };
        let diag = Diagnostic::warning(
            FACT_TABLES_BY_HAND.code,
            only.span.shift(self.base),
            "a factorial table built by hand",
        )
        .with_help(format!(
            "write `let comb = Comb<{z}>({n})` (`std.mod`): `comb.fact(i)` is `i!`, `comb.inv_fact(i)` is `1 / i!` and `comb.choose(a, b)` the binomial, with no table or `%` to keep in step"
        ));
        self.out.push(Finding {
            lint: &FACT_TABLES_BY_HAND,
            diag,
        });
    }

    /// `for x in xs: (m[k] ??= []).push(v)` as the whole body.
    fn group_into_map(&mut self, pat: &Pattern, iter: &Expr, body: &Block) {
        // `(m[k] ??= []).push(v)` desugars to several statements; read the source.
        let span = body.span.shift(self.base);
        let raw = span.text(self.src);
        let lines: Vec<_> = raw
            .lines()
            .map(|l| l.split("//").next().unwrap_or("").trim())
            .filter(|l| !l.is_empty())
            .collect();
        let [raw] = lines.as_slice() else {
            return;
        };
        let Some((m, rest)) = raw.strip_prefix('(').and_then(|r| r.split_once('[')) else {
            return;
        };
        let Some((key, rest)) = rest.rsplit_once("??=") else {
            return;
        };
        let Some(key) = key.trim_end().strip_suffix(']') else {
            return;
        };
        let Some(v) = squash_code(rest)
            .strip_prefix("[]).push(")
            .and_then(|v| v.strip_suffix(')'))
            .map(|_| {
                let at = rest.find(".push(").map_or(0, |i| i + 6);
                rest[at..rest.len() - 1].trim().to_string()
            })
        else {
            return;
        };
        let x = pat.span.shift(self.base).text(self.src).trim();
        let xs = iter.span.shift(self.base).text(self.src).trim();
        if !ident(m) || !ident(x) {
            return;
        }
        let xs = if xs.contains("..") {
            format!("({xs})")
        } else {
            xs.to_string()
        };
        let of = if v == x {
            String::new()
        } else {
            format!(", of: {x} => {v}")
        };
        let diag = Diagnostic::warning(
            GROUP_INTO_MAP.code,
            span,
            format!("`{m}` filled with lists by key, one push per element"),
        )
        .with_help(format!(
            "write `let {m} = {xs}.group(by: {x} => {}{of})` (`std.seq`): the same lists, in input order, with no empty-list default; it is a `HashMap`",
            key.trim()
        ));
        self.out.push(Finding {
            lint: &GROUP_INTO_MAP,
            diag,
        });
    }

    /// `cur += a[i] - a[i - k]` in a loop over `k..n`.
    fn sliding_sum(&mut self, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let it = text(iter.span);
        let Some((k, _)) = it.split_once("..") else {
            return;
        };
        if !ident(&i) || k.is_empty() {
            return;
        }
        for st in &body.stmts {
            let t = text(st.span);
            let Some((cur, rhs)) = t.split_once("+=") else {
                continue;
            };
            let Some((add, sub)) = rhs.split_once('-') else {
                continue;
            };
            let Some(a) = add.strip_suffix(&format!("[{i}]")) else {
                continue;
            };
            if !ident(cur) || !ident(a) || sub != format!("{a}[{i}-{k}]") {
                continue;
            }
            let diag = Diagnostic::warning(
                SLIDING_SUM_BY_HAND.code,
                st.span.shift(self.base),
                format!("`{cur}` kept as the sum of the last `{k}` elements of `{a}` by hand"),
            )
            .with_help(format!(
                "write `let p = Prefix({a})` (`std.prefix`): `p.fold(j - {k}..j)` is the window ending before `j`, for `j` in `{k}..=n`, with no first window to build and no state between iterations"
            ));
            self.out.push(Finding {
                lint: &SLIDING_SUM_BY_HAND,
                diag,
            });
        }
    }

    /// `for _ in 0..k:` popping a heap with `h.pop()!` and never pushing it.
    fn heap_drain(&mut self, stmt: &Stmt, iter: &Expr, body: &Block) {
        let raw = iter.span.shift(self.base).text(self.src).trim();
        let Some(k) = raw.strip_prefix("0..").map(str::trim) else {
            return;
        };
        let whole = squash_code(body.span.shift(self.base).text(self.src));
        let file = squash_code(self.src);
        let pops: Vec<_> = whole.match_indices(".pop()").collect();
        let [(at, _)] = pops.as_slice() else {
            return;
        };
        if !whole[*at..].starts_with(".pop()!") {
            return;
        }
        let Some(h) = whole[..*at]
            .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
            .next()
            .filter(|h| ident(h))
        else {
            return;
        };
        let heap = ["=", ":"].iter().any(|sep| {
            ["MinHeap<", "MaxHeap<", "Heap<"]
                .iter()
                .any(|ty| file.contains(&format!("var {h}{sep}{ty}")))
        });
        if !heap || whole.contains(&format!("{h}.push(")) || pushed_in_fn(self.src, h) {
            return;
        }
        let head = stmt.span.shift(self.base);
        let head = cpl_diag::Span::new(head.start, iter.span.shift(self.base).end);
        let diag = Diagnostic::warning(
            MANUAL_HEAP_DRAIN.code,
            head,
            format!("the top `{k}` of `{h}` popped one by one"),
        )
        .with_help(format!(
            "write `for x in {h}.drain().take({k})` (`std.heap`): the same elements in the same order, each named, stopping early when `{h}` runs out instead of trapping on `pop()!`"
        ));
        self.out.push(Finding {
            lint: &MANUAL_HEAP_DRAIN,
            diag,
        });
    }

    /// `if v != V.last!:` or `if i + 1 < n and ..` guarding a read of the next
    /// position, in a loop over positions.
    fn adjacent(&mut self, stmt: &Stmt, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let v = text(pat.span);
        if !ident(&v) {
            return;
        }
        for (k, b) in body.stmts.iter().enumerate() {
            let t = text(b.span);
            let Some(cond) = t.strip_prefix("if ") else {
                continue;
            };
            let last = cond
                .strip_prefix(&format!("{v}!="))
                .and_then(|r| r.split_once(".last!:"))
                .is_some_and(|(d, _)| ident(d));
            let end = text(iter.span);
            let bound = cond
                .strip_prefix(&format!("{v}+1<"))
                .and_then(|r| r.split(':').next()?.split(" and ").next())
                .is_some_and(|n| end == format!("0..{n}"));
            let at = if bound {
                format!("[{v}+1]")
            } else {
                format!("[{v}]")
            };
            let Some(xs) = t.match_indices(&at).find_map(|(at, _)| {
                let xs = t[..at]
                    .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next()?;
                ident(xs).then_some(xs)
            }) else {
                continue;
            };
            let pair =
                t.contains(&format!("{xs}[{v}]")) && (last || t.contains(&format!("{xs}[{v}+1]")));
            if !(last || bound) || !pair || writes(&text(body.span), xs) {
                continue;
            }
            let span = if body.stmts.len() == 1 && k == 0 {
                stmt.span
            } else {
                b.span
            };
            let diag = Diagnostic::warning(
                ADJACENT_BY_INDEX.code,
                span.shift(self.base),
                format!("neighbouring pairs read by index, guarded so `{v}` is not the last position"),
            )
            .with_help(format!(
                "write `for (x, y) in {xs}.adjacent` (`std.seq`): every neighbouring pair of `{xs}`, with no last-position guard or `+ 1` index to get wrong{}",
                if last {
                    format!("; `{v}.next` is the position after `{v}` where the position itself is needed")
                } else {
                    String::new()
                }
            ));
            self.out.push(Finding {
                lint: &ADJACENT_BY_INDEX,
                diag,
            });
            return;
        }
    }

    /// `for i in 0..n / 2:` comparing `s[i]` with `s[n - 1 - i]`.
    fn palindrome(&mut self, stmt: &Stmt, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let Some(n) = text(iter.span)
            .strip_prefix("0..")
            .and_then(|r| r.strip_suffix("/2"))
            .map(str::to_string)
        else {
            return;
        };
        let whole = text(body.span);
        let mirror = format!("[{n}-1-{i}]");
        let Some(s) = whole.match_indices(&mirror).find_map(|(at, _)| {
            let before = &whole[..at];
            let s = before
                .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?;
            let before = &before[..before.len() - s.len()];
            ["!=", "=="]
                .iter()
                .any(|op| before.ends_with(&format!("{s}[{i}]{op}")))
                .then(|| s.to_string())
        }) else {
            return;
        };
        if !ident(&i) || !ident(&s) {
            return;
        }
        let diag = Diagnostic::warning(
            PALINDROME_BY_HAND.code,
            self.with_flag(stmt.span.shift(self.base)),
            format!("`{s}[{i}]` against `{s}[{n} - 1 - {i}]` tests whether `{s}` is a palindrome"),
        )
        .with_help(format!(
            "write `is_palindrome({s})` (`use std.strings.{{is_palindrome}}`): a `Bool`, with no mirrored index or half-length bound to get wrong"
        ));
        self.out.push(Finding {
            lint: &PALINDROME_BY_HAND,
            diag,
        });
    }
}

impl Visitor for HandSearch<'_> {
    fn expr(&mut self, expr: &Expr) {
        let span = expr.span.shift(self.base);
        let text = squash_code(span.text(self.src));
        let Some(recv) = text.strip_suffix(".sorted()") else {
            return;
        };
        let Some((x, i, range)) = self.argsort(recv) else {
            return;
        };
        let diag = Diagnostic::warning(
            ARGSORT_LOOP.code,
            span,
            format!("pairs `({x}[{i}], {i})` sorted to get the positions in order of `{x}`"),
        )
        .with_help(format!(
            "sort the positions by value: `({range}).sorted(by: {i} => {x}[{i}])`; the sort is stable, so equal values keep their position order as the pairs did, and `{x}[{i}]` is read where it is needed"
        ));
        self.out.push(Finding {
            lint: &ARGSORT_LOOP,
            diag,
        });
    }

    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        {
            self.palindrome(stmt, pat, iter, body);
            self.subset_masks(stmt, pat, iter, body);
            self.adjacent(stmt, pat, iter, body);
            self.fact_table(pat, iter, body);
            self.heap_drain(stmt, iter, body);
            self.group_into_map(pat, iter, body);
            self.sliding_sum(pat, iter, body);
        }
        let StmtKind::While {
            cond: Cond::Expr(cond),
            body,
            else_body: None,
        } = &stmt.kind
        else {
            return;
        };
        let span = stmt.span.shift(self.base);
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let c = text(cond.span);
        let Some(last) = body.stmts.last() else {
            return;
        };
        let records = |st: &str, i: &str| {
            let mut wrote = false;
            let all = body.stmts.iter().all(|b| {
                let t = text(b.span);
                let popped = t.strip_prefix("let ").and_then(|t| {
                    let (j, rhs) = t.split_once('=')?;
                    (ident(j) && (rhs == format!("{st}.last!") || rhs == format!("{st}.pop()!")))
                        .then_some(())
                });
                let answer = t.strip_suffix(&format!("={i}")).is_some_and(|lhs| {
                    lhs.ends_with(']') && lhs.split_once('[').is_some_and(|(ans, _)| ident(ans))
                });
                wrote |= answer;
                popped.is_some() || answer || t == format!("{st}.pop()")
            });
            all && wrote
        };
        if let Some((call, what)) = monotone_pop(&c)
            .filter(|(_, _, st, i)| records(st, i))
            .map(|(c, w, _, _)| (c, w))
        {
            let diag = Diagnostic::warning(
                MONOTONE_STACK_BY_HAND.code,
                span,
                "a monotone stack written by hand".to_string(),
            )
            .with_help(format!(
                "write `{call}` (`std.seq`): each position's nearest later position with a {what} value, as a `[D]?D` with `None` where there is none; `prev_smaller` gives the nearest earlier one"
            ));
            self.out.push(Finding {
                lint: &MONOTONE_STACK_BY_HAND,
                diag,
            });
            return;
        }
        if let Some(s) = c.strip_suffix(">0").or_else(|| c.strip_suffix("!=0")) {
            let step = text(last.span);
            let mask = step.strip_prefix(&format!("{s}=({s}-1)&"));
            if let Some(m) = mask.filter(|m| ident(s) && ident(m)) {
                let diag = Diagnostic::warning(
                    SUBMASK_LOOP.code,
                    span,
                    format!("`{s} = ({s} - 1) & {m}` walks the submasks of `{m}` by hand"),
                )
                .with_help(format!(
                    "write `for {s} in submasks({m}):` (`use std.math.{{submasks}}`): it yields every submask of `{m}`, largest first, and `0` last, so skip `0` there if the loop must not see it"
                ));
                self.out.push(Finding {
                    lint: &SUBMASK_LOOP,
                    diag,
                });
            }
            return;
        }
        let whole = text(body.span);
        if whole.matches(")/3").count() >= 2 && body.stmts.len() >= 3 {
            let diag = Diagnostic::warning(
                TERNARY_SEARCH_BY_HAND.code,
                span,
                "a ternary search over integers written by hand".to_string(),
            )
            .with_help(
                "write `ternary_min(lo..hi, f)!` or `ternary_max(lo..hi, f)!` (`std.search`): golden-section search, about log n calls of `f`, with no stopping rule or final scan to get wrong",
            );
            self.out.push(Finding {
                lint: &TERNARY_SEARCH_BY_HAND,
                diag,
            });
        }
    }
}
