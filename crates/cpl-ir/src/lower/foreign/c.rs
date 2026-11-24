//! C++ habits lowering recovers: `vector(5)` as a size, `m.count(k)`,
//! `m.insert({k, v})` and `a.resize(n)`.

use super::*;

impl<'a> Lowerer<'a> {
    /// R04b.3.2: a built-in collection's one argument is the contents it
    /// copies, what a `for` walks, and a map's are `(key, value)` pairs;
    /// never a size, as C++'s `vector(5)` or a bucket count would read.
    pub(in crate::lower) fn not_contents(
        &mut self,
        map: bool,
        src: &ast::Expr,
        ty: &Ty,
        span: Span,
    ) -> Expr {
        if matches!(ty, Ty::Error) {
            return self.err_expr(span);
        }
        let text = self.source(src.span).trim().to_string();
        let call = self.source(span);
        let name = call
            .split('(')
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        let shown = self.show(ty);
        let article = if shown.starts_with(['A', 'E', 'I', 'O', 'U']) {
            "an"
        } else {
            "a"
        };
        let (message, help) = match ty.seq_elem() {
            Some(elem) if map => (
                cpl_diag::msgs::ir::COPIES_PAIRS_HOLDS.at(&[
                    &name,
                    &text,
                    &text,
                    &self.show(&elem),
                ]),
                format!("pair each with its value: `{name}((x, v) for x in {text})`"),
            ),
            _ if map => (
                cpl_diag::msgs::ir::COPIES_PAIRS_NOT.at(&[&name, &text, &article, &shown]),
                format!("write `{name}()` for an empty map"),
            ),
            _ => (
                cpl_diag::msgs::ir::COPIES_SEQUENCE_NOT.at(&[&name, &text, &article, &shown]),
                format!("write `[0 for _ in 0..{text}]` for {text} zeros, or `[{text}]`"),
            ),
        };
        self.report_said(span, message);
        self.help(help);
        self.err_expr(span)
    }

    /// C++'s `m.count(k)`: a key is in a map or a set or not, `k in m`.
    pub(in crate::lower) fn keyed_count(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        key: &ast::Arg,
        ty: &Ty,
    ) -> Expr {
        let new = format!(
            "{} in {}",
            self.source(key.value.span),
            self.source(base.span)
        );
        let new = self.parenthesized(whole.span, new);
        self.report(
            whole.span,
            msgs::ir::UNKNOWN_FIELD_A_HAS_NO_METHOD,
            &[&self.show(ty)],
        );
        let at = self.span(whole.span);
        let ends = self.stands_alone(whole.span).1;
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("test a key with `in`: `{new}`"));
            if ends {
                last.fix(format!("write `{new}`"), at, new);
            }
        }
        self.err_expr(whole.span)
    }

    /// C++'s `m.insert({k, v})` or `m.emplace(k, v)`: a map sets a key by
    /// index. `insert` keeps a key already there, so the fix tests it first.
    pub(in crate::lower) fn map_insert(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        ty: &Ty,
    ) -> Expr {
        let a: Vec<String> = args.iter().map(|a| self.source(a.value.span)).collect();
        let pair = match a.as_slice() {
            [k, v] => Some((k.clone(), v.clone())),
            [p] => p
                .strip_prefix(['(', '{'])
                .and_then(|p| p.strip_suffix([')', '}']))
                .and_then(split_pair),
            _ => None,
        };
        let m = self.source(base.span);
        self.report(
            whole.span,
            msgs::ir::UNKNOWN_FIELD_A_HAS_NO_METHOD_2,
            &[&self.show(ty), &name.text, &m],
        );
        let at = self.span(whole.span);
        let text = &self.mods[self.cur_mod].text;
        let line = text[..at.start as usize].rsplit('\n').next().unwrap_or("");
        let rest = text[at.end as usize..].split('\n').next().unwrap_or("");
        let statement = line.trim().is_empty() && rest.trim().is_empty();
        if let (Some(last), Some((k, v))) = (self.diags.last_mut(), pair) {
            let new = format!("if {k} not in {m}: {m}[{k}] = {v}");
            last.help = Some(format!(
                "`{m}[{k}] = {v}` replaces a key already there, which C++'s `{}` keeps: `{new}`",
                name.text
            ));
            if statement {
                last.fix(format!("write `{new}`"), at, new);
            }
        }
        self.err_expr(whole.span)
    }
}

/// C++'s names for putting a key in a map.
pub(in crate::lower) fn foreign_insert(method: &str) -> bool {
    matches!(method, "insert" | "emplace")
}

/// `k, v` split at its one top-level comma.
fn split_pair(inner: &str) -> Option<(String, String)> {
    let mut depth = 0i32;
    let mut cuts = Vec::new();
    for (i, c) in inner.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => cuts.push(i),
            _ => {}
        }
    }
    let [cut] = cuts[..] else { return None };
    let (k, v) = (inner[..cut].trim(), inner[cut + 1..].trim());
    (!k.is_empty() && !v.is_empty()).then(|| (k.to_string(), v.to_string()))
}
