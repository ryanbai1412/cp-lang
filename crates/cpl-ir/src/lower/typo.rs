//! The member a mistyped `.name` most likely meant, offered as a fix.

use super::*;
use cpl_diag::msgs;

impl<'a> Lowerer<'a> {
    /// On the error just reported for `.name` of a domain position: an arc
    /// of a graph built from a list of pairs is the neighbour itself, with
    /// no edge record around it to read `.e` from.
    pub(super) fn position_member_hint(&mut self, ty: &Ty, name: &str) {
        if matches!(ty, Ty::Pos(_)) && name == "e" {
            self.help(
                "this is the neighbour itself, and a graph built from a list of pairs numbers no edges; \
                 for `.e`, give the edges a domain: `let edges: [E](u: V, v: V) = ...`, then `Graph.directed(edges)`",
            );
        }
    }

    /// "a `List`", "an `Int`": the type with its article, for messages.
    pub(super) fn a_ty(&self, ty: &Ty) -> String {
        let shown = self.show(ty);
        let an = shown.starts_with(['A', 'E', 'I', 'O', 'U']);
        format!("{} `{shown}`", if an { "an" } else { "a" })
    }

    /// On the error just reported for `.name` of a `ty`: the member it is
    /// a typo of, with the fix. Candidates are the built-in members (a
    /// range has a list's too), the record's fields and every fn in reach that takes `self`.
    pub(super) fn member_typo(&mut self, ty: &Ty, name: &str, span: Span) {
        use cpl_check::Receiver;
        const POS_MEMBERS: [&str; 18] = [
            "after", "ahead", "back", "before", "count", "cyc", "first", "idx", "is_empty",
            "label", "len", "next", "offset", "ord", "prev", "rank", "with", "without",
        ];
        const BOUND_EXTRA: [&str; 4] = ["l", "left", "r", "right"];
        const RANGE_MEMBERS: [&str; 10] = [
            "end", "first", "halves", "is_empty", "last", "mid", "rev", "sorted", "start", "step",
        ];
        const DOMAIN_MEMBERS: [&str; 21] = [
            "after", "all", "any", "at", "before", "count", "cuts", "first", "is_empty", "last",
            "len", "mask", "max", "min", "nth", "rev", "size", "sort", "sorted", "subsets", "sum",
        ];
        const SEQ_EXTRA: [&str; 6] = [
            "adjacent", "inverse", "items", "resized", "reverse", "reversed",
        ];
        const STR_EXTRA: [&str; 6] = [
            "adjacent", "count", "items", "reverse", "reversed", "to_str",
        ];
        const INT_MEMBERS: [&str; 5] = ["abs", "idx", "isqrt", "ord", "sqrt"];
        const FLOAT_MEMBERS: [&str; 6] = ["abs", "ceil", "floor", "round", "sqrt", "trunc"];
        let receiver = receiver_of(ty);
        if let Some(own) = receiver.and_then(|r| cpl_check::foreign_member(r, name)) {
            return self.suggest_last(span, own.into());
        }
        let on_str = matches!(ty, Ty::Str | Ty::StrView);
        let use_module = |s: &str| {
            receiver
                .and_then(|_| cpl_check::method_module(s, on_str))
                .filter(|_| self.cur_mod == 0)
        };
        if let Some(module) = use_module(name) {
            return self.use_fix(&module, name, None);
        }
        let mut names: Vec<&str> = receiver
            .into_iter()
            .flat_map(cpl_check::builtin_members)
            .map(|m| m.name)
            .collect();
        match ty {
            Ty::Record(fields) => {
                names.extend(fields.iter().map(|(f, _)| f.as_str()));
            }
            Ty::Struct(id) => {
                let info = &self.structs[*id];
                names.extend(info.def.fields.iter().map(|(f, _)| f.as_str()));
                names.extend(info.methods.keys().map(String::as_str));
                names.extend(info.gen_methods.keys().map(String::as_str));
                names.extend(info.projections.keys().map(String::as_str));
                names.extend(info.consts.keys().map(String::as_str));
                names.extend(info.owned_domains.keys().map(String::as_str));
                if info.methods.contains_key("range_first") {
                    names.push("range");
                }
            }
            Ty::Pos(_) => names.extend(POS_MEMBERS),
            Ty::Bound(_) => {
                names.extend(POS_MEMBERS);
                names.extend(BOUND_EXTRA);
            }
            Ty::Range(_) => {
                names.extend(RANGE_MEMBERS);
                names.extend(cpl_check::builtin_members(Receiver::List).map(|m| m.name));
                names.extend(SEQ_EXTRA);
            }
            Ty::DomainVal => {
                names.extend(DOMAIN_MEMBERS);
            }
            Ty::List(_) | Ty::Array { .. } | Ty::MutView(..) | Ty::ListView(..) | Ty::Keyed(..) => {
                names.extend(SEQ_EXTRA);
            }
            Ty::Str | Ty::StrView => {
                names.extend(STR_EXTRA);
            }
            Ty::Int | Ty::I128 | Ty::U64 => {
                names.extend(INT_MEMBERS);
            }
            Ty::Float => {
                names.extend(FLOAT_MEMBERS);
            }
            _ => {}
        }
        for (sym_name, sym) in &self.mod_syms[self.cur_mod] {
            let decl = match *sym {
                Sym::Fn(fid) => self.sigs[fid].decl,
                Sym::GenFn(gfid) => self.gen_fns[gfid].decl,
                _ => continue,
            };
            if decl.params.first().is_some_and(|p| p.name.text == "self") {
                names.push(sym_name.split(" of ").next().unwrap_or(sym_name));
            }
        }
        // Auto-imported stdlib UFCS methods are only pulled into `mod_syms`
        // when spelled right, so include them as typo candidates too.
        if !matches!(ty, Ty::Struct(_) | Ty::Enum(_) | Ty::Record(_)) {
            names.extend(
                cpl_check::std_methods()
                    .iter()
                    .map(|(_, m)| m.text.as_str()),
            );
        }
        // A std `fn f(self: T)` whose `T` this receiver isn't would only
        // trade this error for a mismatch (errors/R7).
        let heads: &[&str] = match ty {
            Ty::List(_) | Ty::ListView(..) | Ty::MutView(..) | Ty::Keyed(..) => &["List"],
            Ty::Array { .. } => &["["],
            Ty::Str | Ty::StrView => &["Str"],
            Ty::Int | Ty::I128 | Ty::U64 => &["Int"],
            Ty::Float => &["Float"],
            Ty::Char => &["Char"],
            Ty::Range(_) => &["Range"],
            _ => &[],
        };
        let fits = |m: &str| {
            heads.is_empty()
                || cpl_check::std_self_heads().get(m).is_none_or(|all| {
                    all.iter()
                        .any(|h| h.as_deref().is_none_or(|h| heads.contains(&h)))
                })
        };
        let Some(s) = cpl_check::closest(name, names.into_iter().filter(|m| fits(m))) else {
            return;
        };
        let auto = cpl_check::std_methods().iter().any(|(_, m)| m.text == s);
        match use_module(&s).filter(|_| !auto) {
            Some(module) => self.use_fix(&module, &s, Some(span)),
            None => self.suggest_last(span, s),
        }
    }

    /// Help and fix for a std method that needs its `use`, renaming the
    /// misspelled name at `rename` too.
    fn use_fix(&mut self, module: &str, s: &str, rename: Option<Span>) {
        let (at, import) = cpl_check::import_edit(&self.mods[0].text, module, s);
        let mut edits = vec![(at, import)];
        let use_line = format!("use std.{module}.{{{s}}}");
        let (help, title) = match rename {
            Some(span) => {
                edits.push((self.span(span), s.to_string()));
                let help = format!("did you mean `{s}`, from `std.{module}`?");
                (help, format!("change to `{s}` and add `{use_line}`"))
            }
            None => {
                let help = format!("`{s}` is in the standard library: add `{use_line}`");
                (help, format!("add `{use_line}`"))
            }
        };
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(help);
            last.fixes.push(cpl_diag::Fix { title, edits });
        }
    }

    /// `List.name(..)` with no such static fn: one error, and `filled` if
    /// it is a typo of that.
    pub(super) fn list_static_typo(&mut self, name: &ast::Name, span: Span) -> Expr {
        self.report(
            name.span,
            msgs::ir::UNKNOWN_FIELD_A_LIST_HAS_NO,
            &[&name.text],
        );
        if let Some(s) = cpl_check::closest(&name.text, ["filled"].into_iter()) {
            self.suggest_last(name.span, s);
        }
        self.err_expr(span)
    }

    /// Offers `s` for the source at `span` on the error just reported.
    /// `s.upper()`: `upper` is a `Char` getter, so the `Str` receiver is a
    /// mismatch; the string's own function is `to_upper`.
    pub(super) fn char_fn_on_str(&mut self, base: &ast::Expr, name: &ast::Name) {
        let Some(own) = cpl_check::foreign_str_fn(&name.text) else {
            return;
        };
        let at = self.span(base.span);
        let edit = self.span(name.span);
        let whole = self.source(base.span);
        let member = self.span(name.span);
        let last = self.diags.last_mut().filter(|d| {
            (d.code == codes::TYPE_MISMATCH && d.span == at && d.message.ends_with("found `Str`"))
                || (d.code == codes::UNKNOWN_FIELD
                    && d.span == member
                    && d.message.starts_with("a `Str` has no method"))
        });
        if let Some(d) = last {
            d.fixes.clear();
            d.help = Some(format!(
                "`{}` is a `Char` getter; a whole `Str` is `{whole}.{own}()`",
                name.text
            ));
            d.fix(format!("write `{own}`"), edit, own);
        }
    }

    /// `['R', 'W'].join("")`: a `List<Char>` receiver is not made a `Str` for a `self: Str` fn
    /// (R04.4.22 converts arguments, not receivers), or it would join `""` with `"RW"` between.
    pub(super) fn char_list_as_str(&mut self, sym: Sym, base: &ast::Expr) -> bool {
        let first = match sym {
            Sym::Fn(fid) => self.sigs[fid].decl.params.first(),
            Sym::GenFn(gfid) => self.gen_fns[gfid].decl.params.first(),
            _ => None,
        };
        let takes_str = first.and_then(|p| p.ty.as_ref()).is_some_and(
            |t| matches!(&t.kind, ast::TypeKind::Named { name, .. } if name.text == "Str"),
        );
        if !takes_str || !matches!(self.probe_ty_silent(base), Some(Ty::List(e)) if *e == Ty::Char)
        {
            return false;
        }
        let (text, at) = (self.source(base.span).to_string(), self.span(base.span));
        let mut d = cpl_diag::msgs::check::STR_METHOD_ON_CHARS.error(at, &[&text]);
        d.help = Some(format!("write `Str({text})`"));
        d.fix(format!("write `Str({text})`"), at, format!("Str({text})"));
        self.diags.push(d);
        true
    }

    /// `t[0]` on a tuple: its positional fields are read `t.0`, `t.1`.
    pub(super) fn tuple_index_fix(&mut self, ty: &Ty, index: &ast::Expr, span: Span) {
        let (Ty::Tuple(_), ast::ExprKind::Int(n)) = (ty, &index.kind) else {
            return;
        };
        let edit = self.span(Span::new(index.span.start - 1, span.end));
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!(
                "a tuple's fields are read `.0`, `.1`: write `.{n}`"
            ));
            last.fix(format!("write `.{n}`"), edit, format!(".{n}"));
        }
    }

    /// `x.unwrap()` on an optional is `x!`, and on a value is just `x`;
    /// `xs.iter()` and `s.chars()`
    /// are left out, since a list or string is iterated as it is.
    pub(super) fn foreign_call_fix(&mut self, ty: &Ty, base: &ast::Expr, method: &str, span: Span) {
        let listed = matches!(ty, Ty::List(_) | Ty::Array { .. });
        if let Some(help) = cpl_check::list_end_help(method).filter(|_| listed) {
            if let Some(last) = self.diags.last_mut() {
                last.help = Some(help.to_string());
            }
            return;
        }
        if matches!(ty, Ty::Str | Ty::List(_)) && method == "end" {
            let new = format!("{}.len", self.source(base.span));
            let edit = self.span(Span::new(base.span.start, span.end));
            if let Some(last) = self.diags.last_mut() {
                last.help = Some(cpl_check::ITERATOR_POSITION_HELP.to_string());
                last.fixes.clear();
                last.fix(format!("write `{new}`"), edit, new);
            }
            return;
        }
        let (help, new) = match (ty, method) {
            (Ty::Opt(_), "unwrap" | "expect") => (cpl_check::OPTIONAL_UNWRAP_HELP, "!"),
            (_, "unwrap") => ("this is not an optional, so there is nothing to unwrap", ""),
            (Ty::Str | Ty::List(_), _) if cpl_check::foreign_iter_call(method) => {
                ("a list or string is iterated as it is: `for x in xs`", "")
            }
            _ => return,
        };
        let edit = self.span(Span::new(base.span.end, span.end));
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(help.to_string());
            let title = if new.is_empty() {
                format!("remove `.{method}()`")
            } else {
                "write `!`".to_string()
            };
            last.fixes.clear();
            last.fix(title, edit, new);
        }
    }

    /// A one-character string where a `Char` or a `Char` test is wanted:
    /// `'a'`, or `c => c == 'a'`; a statement where a value is wanted.
    pub(super) fn type_mismatch(&mut self, span: Span, want: &Ty, found: &Ty, said: msgs::Said) {
        // A position or cut of one domain where another's is wanted is the wrong domain.
        let code = match (want, found) {
            (Ty::Pos(a), Ty::Pos(b)) | (Ty::Bound(a), Ty::Bound(b)) if a != b => {
                codes::WRONG_DOMAIN
            }
            _ => codes::TYPE_MISMATCH,
        };
        self.report_said(span, said.with_code(code));
        let text = self.source(span);
        if matches!(found, Ty::Unit) && !text.contains('\n') {
            if let Some(last) = self
                .diags
                .last_mut()
                .filter(|d| d.code == codes::TYPE_MISMATCH)
            {
                last.help = Some(format!(
                    "`{text}` gives no value: a fn's result is its last line, or a `return` with a value"
                ));
            }
            return;
        }
        if matches!(found, Ty::Opt(inner) if **inner == *want) && !text.contains('\n') {
            let end = self.span(span).end;
            if let Some(last) = self
                .diags
                .last_mut()
                .filter(|d| d.code == codes::TYPE_MISMATCH)
            {
                last.help = Some(format!(
                    "`{text}` may be `None`: write `{text}!` to unwrap it, or `{text} ?? d` with a default `d`"
                ));
                last.fixes.clear();
                last.fix("unwrap it with `!`", Span::point(end), "!");
            }
            return;
        }
        // Python's `sep=' '`: its quotes make a string either way.
        let quoted = text.strip_prefix('\'').and_then(|t| t.strip_suffix('\''));
        if let (Ty::Char, Ty::Str, Some(inner)) = (found, want, quoted.filter(|t| !t.contains('"')))
        {
            let (new, edit) = (format!("\"{inner}\""), self.span(span));
            if let Some(last) = self
                .diags
                .last_mut()
                .filter(|d| d.code == codes::TYPE_MISMATCH)
            {
                last.help = Some("a string is written in double quotes".into());
                last.fix(format!("write `{new}`"), edit, new);
            }
            return;
        }
        let Some(c) = cpl_check::char_literal(&text) else {
            return;
        };
        let (new, help) = match (found, want) {
            (Ty::Str, Ty::Char) => (c.clone(), "a character is written in single quotes"),
            (Ty::Str, Ty::Fn(args, ret))
                if matches!(args.as_slice(), [Ty::Char]) && matches!(**ret, Ty::Bool) =>
            {
                (
                    format!("c => c == {c}"),
                    "a test on each character is a lambda",
                )
            }
            _ => return,
        };
        let edit = self.span(span);
        if let Some(last) = self
            .diags
            .last_mut()
            .filter(|d| d.code == codes::TYPE_MISMATCH)
        {
            last.help = Some(format!("{help}: `{new}`"));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), edit, new);
        }
    }

    /// `l == r` with types that don't meet; `s[0] == "a"` gets `'a'`.
    pub(super) fn compare_mismatch(&mut self, span: Span, l: &Expr, r: &Expr) {
        let said = msgs::ir::CANT_COMPARE.at(&[&self.show(&l.ty), &self.show(&r.ty)]);
        let code = match (&l.ty, &r.ty) {
            (Ty::Pos(_), Ty::Pos(_)) => codes::WRONG_DOMAIN,
            _ => codes::TYPE_MISMATCH,
        };
        self.report_said(span, said.with_code(code));
        // Lowered spans are already absolute in the module's text.
        let lit = match (&l.ty, &r.ty) {
            (Ty::Char, Ty::Str) => r,
            (Ty::Str, Ty::Char) => l,
            _ => return,
        };
        let text = self.mods[self.cur_mod]
            .text
            .get(lit.span.start as usize..lit.span.end as usize)
            .unwrap_or("");
        let Some(c) = cpl_check::char_literal(text) else {
            return;
        };
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("a character is written in single quotes: `{c}`"));
            last.fix(format!("write `{c}`"), lit.span, c);
        }
    }

    /// Whether the code at `span` starts an operand (after `(`, `,`, `=`,
    /// `[`, `:`, `if` or `while`) and ends one (before `)`, `,`, `]`, `:` or the line's end).
    pub(in crate::lower) fn stands_alone(&self, span: Span) -> (bool, bool) {
        let text = &self.mods[self.cur_mod].text;
        let abs = self.span(span);
        let before = text[..abs.start as usize].trim_end();
        let after = text[abs.end as usize..]
            .trim_start_matches(' ')
            .chars()
            .next();
        let word = before.rsplit(|c: char| !c.is_alphanumeric()).next();
        let opens = matches!(before.chars().last(), Some('(' | ',' | '=' | '[' | ':'))
            || matches!(word, Some("if" | "while" | "elif"));
        (
            opens,
            matches!(after, Some(')' | ',' | ']' | ':' | '\n') | None),
        )
    }

    /// `new`, which replaces the code at `span`, in parentheses unless it
    /// stands alone.
    pub(in crate::lower) fn parenthesized(&self, span: Span, new: String) -> String {
        match self.stands_alone(span) {
            (true, true) => new,
            _ => format!("({new})"),
        }
    }

    /// `x.m<T>()` where `x`'s type has no generic method `m`.
    pub(super) fn no_generic_method(&mut self, ty: &Ty, name: &ast::Name, span: Span) -> Expr {
        if !matches!(ty, Ty::Error) {
            self.report(
                span,
                msgs::ir::UNKNOWN_FIELD_NO_METHOD_ON,
                &[&name.text, &self.show(ty)],
            );
            if let (Some(last), "parse") = (self.diags.last_mut(), name.text.as_str()) {
                last.help = Some("`parse` reads a value from a `Str`".to_string());
            }
        }
        self.err_expr(span)
    }

    /// `read<List<Int>>(n)`: the type says how many, `read<[n]Int>()`.
    pub(super) fn read_count_arg(&mut self, count: &ast::Expr, elem: &Ty, span: Span) {
        let new = format!("read<[{}]{}>()", self.source(count.span), self.show(elem));
        self.report(span, msgs::ir::WRONG_ARG_COUNT_READ_TAKES_NO_COUNT, &[]);
        let at = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("`n` values read as `[n]T`: `{new}`"));
            last.fix(format!("write `{new}`"), at, new);
        }
    }

    /// `print(xs.sort())`: an in-place change gives nothing to print; its
    /// copying form does.
    pub(super) fn no_value_to_print(&mut self, span: Span) {
        self.report(span, msgs::ir::TYPE_MISMATCH_THIS_HAS_NO_VALUE_2, &[]);
        let text = self.source(span);
        let copy = [(".sort()", ".sorted()"), (".reverse()", ".rev")]
            .into_iter()
            .find_map(|(old, new)| Some(format!("{}{new}", text.strip_suffix(old)?)));
        let edit = self.span(span);
        if let (Some(last), Some(copy)) = (self.diags.last_mut(), copy) {
            last.help = Some(format!(
                "`{text}` changes it in place and gives nothing back; `{copy}` gives the result"
            ));
            last.fix(format!("write `{copy}`"), edit, copy);
        }
    }

    /// `a.last = 5`: `last` reads the end item and is no place, so the
    /// end item is assigned through its index.
    pub(super) fn last_place_fix(&mut self, ty: &Ty, base: Span, name: &ast::Name) {
        let b = self.source(base);
        let index = match ty {
            _ if name.text != "last" || b.contains('\n') => return,
            Ty::List(_) => format!("{b}.len - 1"),
            Ty::Array { dims, .. } if dims.len() == 1 => format!("{b}.keys.last!"),
            _ => return,
        };
        let new = format!("{b}[{index}]");
        let edit = self.span(Span::new(base.start, name.span.end));
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!(
                "`last` is read-only; assign the end item by its index, `{new}`"
            ));
            last.fix(format!("write `{new}`"), edit, new);
        }
    }

    /// `reverse: true` or the dropped `desc: true`: largest first is a
    /// descending key, `by: x => desc(x)` (decision 290).
    pub(super) fn sort_reverse_arg(&mut self, args: &[ast::Arg]) -> bool {
        let Some((name, value)) = args.iter().find_map(|a| {
            (a.name.as_ref())
                .filter(|n| matches!(n.text.as_str(), "reverse" | "desc"))
                .map(|n| (n, &a.value))
        }) else {
            return false;
        };
        self.report(
            name.span,
            msgs::ir::WRONG_ARG_COUNT_A_SORT_PUTS_THE,
            &[&name.text],
        );
        let by = (args.iter()).find(|a| {
            a.name
                .as_ref()
                .is_some_and(|n| matches!(n.text.as_str(), "by" | "key"))
        });
        let keyed = by.is_some();
        let edit = self.span(Span::new(name.span.start, value.span.end));
        let truthy = self.source(value.span) == "true";
        // `by: w => w.len, reverse: true` is `by: w => desc(w.len)`.
        let wrapped = by.filter(|_| truthy).and_then(|by| {
            let last = args.last().filter(|l| std::ptr::eq(&l.value, value))?;
            let before = args.get(args.len().checked_sub(2)?)?;
            let text = self.source(by.value.span).to_string();
            let (head, body) = text.split_once("=>")?;
            let key = format!("{}=> desc({})", head, body.trim());
            let cut = Span::new(before.value.span.end, last.value.span.end);
            Some(vec![
                (self.span(by.value.span), key),
                (self.span(cut), String::new()),
            ])
        });
        if let Some(last) = self.diags.last_mut() {
            if let Some(edits) = wrapped {
                last.help = Some("`desc(key)` orders largest first".into());
                last.fixes.push(cpl_diag::Fix {
                    title: "wrap the key in `desc`".into(),
                    edits,
                });
            } else if keyed {
                last.help = Some("wrap the key in `desc`: `by: x => desc(key)`".into());
            } else if truthy {
                last.help = Some("`desc(x)` orders largest first".into());
                last.fix("write `by: x => desc(x)`", edit, "by: x => desc(x)");
            }
        }
        true
    }

    pub(super) fn sort_unknown_arg(&mut self, args: &[ast::Arg]) -> bool {
        let Some(named) = args
            .iter()
            .find_map(|a| a.name.as_ref().filter(|n| n.text != "by"))
        else {
            return false;
        };
        let said = msgs::ir::SORT_HAS_NO_PARAMETER.at(&[&named.text]);
        self.unknown_arg_name(said, named, &["by"]);
        true
    }

    pub(super) fn suggest_last(&mut self, span: Span, s: String) {
        let span = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.suggest(span, s);
        }
    }

    /// `x.max(3)` on a number: the larger of two values is `max(x, 3)`.
    pub(super) fn number_max_call(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) -> bool {
        let (Ty::Int | Ty::I128 | Ty::U64 | Ty::Float, "max" | "min", [arg]) = (ty, method, args)
        else {
            return false;
        };
        let new = format!(
            "{method}({}, {})",
            self.source(base.span),
            self.source(arg.value.span)
        );
        self.report(
            span,
            msgs::ir::TYPE_MISMATCH_TAKES_A_SEQUENCE_S,
            &[&method, &method, &self.show(ty)],
        );
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("the {method} of two values is the fn `{new}`"));
            last.fix(format!("write `{new}`"), edit, new);
        }
        true
    }

    /// `a.zip(b)` where `zip` is a free function, not a method (its first
    /// parameter isn't `self`): the call is `zip(a, b)`.
    pub(super) fn free_call_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        let sym = (self.resolved_method(method, base, span))
            .or_else(|| self.method_syms(method).into_iter().next());
        let decl = match sym {
            Some(Sym::Fn(fid)) => self.sigs[fid].decl,
            Some(Sym::GenFn(gfid)) => self.gen_fns[gfid].decl,
            _ => return,
        };
        let Some(first) = decl.params.first() else {
            return;
        };
        let required = decl.params.iter().filter(|p| p.default.is_none()).count();
        if first.name.text == "self" || decl.params.len() <= args.len() || required > args.len() + 1
        {
            return;
        }
        let takes = first
            .ty
            .as_ref()
            .and_then(|t| self.written_shape(t, &decl.generics));
        if matches!((takes, self.shape(ty)), (Some(takes), Some(given)) if takes != given) {
            return;
        }
        let rest: String = args
            .iter()
            .map(|a| {
                let name = a
                    .name
                    .as_ref()
                    .map_or(String::new(), |n| format!("{}: ", n.text));
                let mutable = if a.mutable { "mut " } else { "" };
                format!(", {name}{mutable}{}", self.source(a.value.span))
            })
            .collect();
        let new = format!("{method}({}{rest})", self.source(base.span));
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!(
                "`{method}` is a free function, not a method: `{new}`"
            ));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), edit, new);
        }
    }

    /// `s.parse()` with no type: `parse<T>` reads a `T`, so it has to be
    /// named (R10.2.4). The fix names the type the result is used as, else
    /// `Int`.
    pub(super) fn parse_without_type(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        hint: Option<&Ty>,
    ) -> bool {
        if !matches!(self.probe_ty_silent(base), Some(Ty::Str | Ty::StrView)) {
            return false;
        }
        let ty = match hint {
            Some(Ty::Opt(t)) if !matches!(**t, Ty::Unit) => self.show(t),
            _ => "Int".to_string(),
        };
        let recv = self.source(base.span).to_string();
        let mut d =
            cpl_diag::msgs::check::PARSE_NEEDS_TYPE.error(self.span(name.span), &[&recv, &ty]);
        d.help = Some(format!(
            "`parse<T>()` reads a `T` by the rules `read` uses (R10.2.4), so name it: `{recv}.parse<{ty}>()`, or `{recv}.parse<(Int, Int)>()` for two numbers"
        ));
        let at = self.span(Span::point(name.span.end));
        d.fix(format!("write `parse<{ty}>()`"), at, format!("<{ty}>"));
        self.diags.push(d);
        true
    }
}

/// The error for `Name<..>` written with `given` type arguments too few.
pub(super) fn missing_type_args(name: &str, given: usize) -> msgs::Said {
    if given == 0 {
        return msgs::ir::NEEDS_ELEMENT_TYPE.at(&[&name, &name]);
    }
    let args = vec!["Int"; given + 1].join(", ");
    msgs::ir::MORE_TYPE_ARGS.at(&[&name, &name, &args])
}

/// The built-in type whose members `ty` has.
fn receiver_of(ty: &Ty) -> Option<cpl_check::Receiver> {
    use cpl_check::Receiver;
    match ty {
        Ty::Str | Ty::StrView => Some(Receiver::Str),
        Ty::List(_) | Ty::MutView(..) | Ty::ListView(..) | Ty::Keyed(..) => Some(Receiver::List),
        Ty::Map(..) => Some(Receiver::Map),
        Ty::HashMap(..) => Some(Receiver::HashMap),
        Ty::Set(_) => Some(Receiver::Set),
        Ty::HashSet(_) => Some(Receiver::HashSet),
        Ty::Array { .. } => Some(Receiver::Array),
        _ => None,
    }
}

/// The error for a built-in member called with arguments it doesn't take:
/// `xs.fold()` names what `fold` takes rather than saying it is missing.
pub(super) fn builtin_misuse(ty: &Ty, method: &str) -> Option<msgs::Said> {
    let row = cpl_check::builtin_members(receiver_of(ty)?).find(|m| m.name == method)?;
    Some(msgs::ir::MEMBER_TAKES.at(&[&method, &row.params, &row.example]))
}
