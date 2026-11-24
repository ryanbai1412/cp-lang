//! The member a mistyped `.name` most likely meant, offered as a fix.

use super::*;

impl<'a> Lowerer<'a> {
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
        const DOMAIN_MEMBERS: [&str; 22] = [
            "after", "all", "any", "at", "before", "count", "cuts", "first", "is_empty", "last",
            "len", "mask", "max", "min", "nth", "rev", "size", "sort", "sorted", "subsets", "sum",
            "windows",
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
        let message = format!("a `List` has no method `{}`", name.text);
        self.error(codes::UNKNOWN_FIELD, name.span, message);
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

    /// `xs.join(" ")`, Python's order: `join` is the separator's, `" ".join(xs)`.
    pub(super) fn join_receiver(
        &mut self,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        span: Span,
    ) {
        let ([sep], "join") = (args, name.text.as_str()) else {
            return;
        };
        let at = self.span(base.span);
        let edit = self.span(span);
        let new = format!(
            "{}.join({})",
            self.source(sep.value.span),
            self.source(base.span)
        );
        let member = self.span(name.span);
        let last = self.diags.last_mut().filter(|d| {
            (d.code == codes::TYPE_MISMATCH
                && d.span == at
                && (d.message.starts_with("expected `Str`")
                    || d.message.starts_with("this method is called on a `Str`")))
                || (d.code == codes::UNKNOWN_FIELD
                    && d.span == member
                    && d.message.starts_with("a `List<"))
        });
        if let Some(d) = last {
            d.fixes.clear();
            d.help = Some(format!("`join` is called on the separator: `{new}`"));
            d.fix(format!("write `{new}`"), edit, new);
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
        let message = format!("this method is called on a `Str`, and `{text}` is a `List<Char>`");
        let mut d = Diagnostic::error(codes::TYPE_MISMATCH, at, message);
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
            (Ty::Opt(_), "unwrap" | "expect") => (
                "`x!` unwraps an optional, trapping on `None` at this place (R02.4.2)",
                "!",
            ),
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
    pub(super) fn type_mismatch(&mut self, span: Span, want: &Ty, found: &Ty, message: String) {
        self.error(codes::TYPE_MISMATCH, span, message);
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
        let message = format!(
            "can't compare a `{}` with a `{}`",
            self.show(&l.ty),
            self.show(&r.ty)
        );
        let code = match (&l.ty, &r.ty) {
            (Ty::Pos(_), Ty::Pos(_)) => codes::WRONG_DOMAIN,
            _ => codes::TYPE_MISMATCH,
        };
        self.error(code, span, message);
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

    /// Python's `a.pop(i)` and `m.get(k, d)`, C++'s `a.resize(n)`: `a.remove(i)`,
    /// `m.get(k) ?? d`, `a.resize(n, 0)`.
    pub(super) fn foreign_method_args(
        &mut self,
        whole: &ast::Expr,
        base: &ast::Expr,
        name: &ast::Name,
        args: &[ast::Arg],
        ty: &Ty,
    ) -> Expr {
        let span = whole.span;
        if args.iter().any(|a| a.name.is_some()) {
            return self.err_expr(span);
        }
        let b = self.source(base.span);
        let a: Vec<String> = args.iter().map(|a| self.source(a.value.span)).collect();
        let (new, help) = match (name.text.as_str(), a.as_slice()) {
            ("pop", [i]) if matches!(ty, Ty::List(_)) => (
                format!("{b}.remove({i})"),
                "removing at an index is `remove`",
            ),
            // C++'s `a.resize(n)` fills with the element's zero.
            ("resize", [n]) => {
                let Ty::List(elem) = ty else {
                    return self.err_expr(span);
                };
                let zero = match &**elem {
                    Ty::Int | Ty::I128 | Ty::U64 => "0",
                    Ty::Float => "0.0",
                    Ty::Bool => "false",
                    Ty::Str => "\"\"",
                    _ => return self.err_expr(span),
                };
                (
                    format!("{b}.resize({n}, {zero})"),
                    "`resize` takes the value for new places",
                )
            }
            ("get", [k, d]) => {
                let new = self.parenthesized(whole.span, format!("{b}.get({k}) ?? {d}"));
                (new, "a default for a missing key is given with `??`")
            }
            _ => return self.err_expr(span),
        };
        let at = self.span(whole.span);
        if let Some(last) = self
            .diags
            .last_mut()
            .filter(|d| d.code == codes::WRONG_ARG_COUNT)
        {
            last.help = Some(format!("{help}: `{new}`"));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), at, new);
        }
        self.err_expr(span)
    }

    /// Whether the code at `span` starts an operand (after `(`, `,`, `=`,
    /// `[`, `:`, `if` or `while`) and ends one (before `)`, `,`, `]`, `:` or the line's end).
    fn stands_alone(&self, span: Span) -> (bool, bool) {
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

    /// C++'s `m.count(k)`: a key is in a map or a set or not, `k in m`.
    pub(super) fn keyed_count(
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
        let message = format!(
            "a `{}` has no method `count`: a key is in it or not",
            self.show(ty)
        );
        self.error(codes::UNKNOWN_FIELD, whole.span, message);
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
    pub(super) fn map_insert(
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
        let message = format!(
            "a `{}` has no method `{}`: a key is set by index, `{m}[k] = v`",
            self.show(ty),
            name.text
        );
        self.error(codes::UNKNOWN_FIELD, whole.span, message);
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

    /// `x.m<T>()` where `x`'s type has no generic method `m`.
    pub(super) fn no_generic_method(&mut self, ty: &Ty, name: &ast::Name, span: Span) -> Expr {
        if !matches!(ty, Ty::Error) {
            let message = format!("no method `{}` on `{}`", name.text, self.show(ty));
            self.error(codes::UNKNOWN_FIELD, span, message);
            if let (Some(last), "parse") = (self.diags.last_mut(), name.text.as_str()) {
                last.help = Some("`parse` reads a value from a `Str`".to_string());
            }
        }
        self.err_expr(span)
    }

    /// `read<List<Int>>(n)`: the type says how many, `read<[n]Int>()`.
    pub(super) fn read_count_arg(&mut self, count: &ast::Expr, elem: &Ty, span: Span) {
        let new = format!("read<[{}]{}>()", self.source(count.span), self.show(elem));
        self.error(
            codes::WRONG_ARG_COUNT,
            span,
            "`read` takes no count: its type says how many values to read",
        );
        let at = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("`n` values read as `[n]T`: `{new}`"));
            last.fix(format!("write `{new}`"), at, new);
        }
    }

    /// JavaScript's `x.toFixed(2)` and `x.to_string()` on a float: its
    /// text is an interpolation, `"{x:.2}"` or `"{x}"` (R10).
    pub(super) fn float_text_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        if !matches!(base.kind, ast::ExprKind::Name(_)) {
            return;
        }
        let x = self.source(base.span);
        let new = match (ty, method, args) {
            (Ty::Float, "toFixed", [d]) if matches!(d.value.kind, ast::ExprKind::Int(_)) => {
                format!("\"{{{x}:.{}}}\"", self.source(d.value.span))
            }
            (Ty::Float, "to_string" | "toString", []) => format!("\"{{{x}}}\""),
            _ => return,
        };
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("a float's text is an interpolation: `{new}`"));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), edit, new);
        }
    }

    /// Python's `sorted(xs, reverse=True)`: CP Lang's sort says `desc: true`.
    /// `print(xs.sort())`: an in-place change gives nothing to print; its
    /// copying form does.
    pub(super) fn no_value_to_print(&mut self, span: Span) {
        self.error(codes::TYPE_MISMATCH, span, "this has no value to print");
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

    /// `xs.sort(x => -x)` or `reverse: true`: the key goes by name, and
    /// largest first is `desc: true`.
    pub(super) fn sort_reverse_arg(&mut self, args: &[ast::Arg]) -> bool {
        if let [arg] = args {
            if arg.name.is_none() && matches!(arg.value.kind, ast::ExprKind::Lambda { .. }) {
                let message = "a sort takes its key by name: `by: x => ..`";
                self.error(codes::WRONG_ARG_COUNT, arg.value.span, message);
                let at = self.span(arg.value.span);
                let title = format!("write `by: {}`", self.source(arg.value.span));
                if let Some(last) = self.diags.last_mut() {
                    last.fix(title, Span::new(at.start, at.start), "by: ");
                }
                return true;
            }
        }
        let Some(name) = args
            .iter()
            .find_map(|a| a.name.as_ref().filter(|n| n.text == "reverse"))
        else {
            return false;
        };
        let message = "a sort takes `desc: true` for largest first, not `reverse:`";
        self.error(codes::WRONG_ARG_COUNT, name.span, message);
        let edit = self.span(name.span);
        if let Some(last) = self.diags.last_mut() {
            last.fix("write `desc`", edit, "desc");
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
        let message = format!("`sort` has no parameter named `{}`", named.text);
        self.unknown_arg_name(message, named, &["by", "desc"]);
        true
    }

    /// A named argument the callee lacks; suggests the label meant, including
    /// Python's names for the same option (`key=` and `cmp=` are `by:`).
    pub(super) fn unknown_arg_name(&mut self, message: String, name: &ast::Name, labels: &[&str]) {
        self.error(codes::UNKNOWN_ARG_NAME, name.span, message);
        let foreign = match name.text.as_str() {
            "key" | "cmp" => Some("by"),
            "reverse" => Some("desc"),
            _ => None,
        };
        if let Some(label) = foreign.filter(|l| labels.contains(l)) {
            let at = self.span(name.span);
            if let Some(last) = self.diags.last_mut() {
                last.help = Some(format!("CP Lang names this argument `{label}`"));
                last.fix(format!("write `{label}`"), at, label);
            }
        } else if let Some(label) = cpl_check::closest(&name.text, labels.iter().copied()) {
            self.suggest_last(name.span, label);
        }
    }

    /// Python's `line.split()`: CP Lang splits on one `Char`, and a line's
    /// tokens are split on `' '`.
    pub(super) fn split_on_space(&mut self, name: &str, param: &str, call: Span) {
        if (name, param) != ("split", "sep") || !self.source(call).ends_with("()") {
            return;
        }
        let at = self.span(call);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some("a string splits on one `Char`: `split(' ')`".to_string());
            last.fix("write `split(' ')`", Span::point(at.end - 1), "' '");
        }
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
        let message = format!(
            "`.{method}` takes a sequence's {method}, and `{}` is one number",
            self.show(ty)
        );
        self.error(codes::TYPE_MISMATCH, span, message);
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

    /// JavaScript's `xs.slice(a, b)` or `s.substring(a, b)`: a slice indexes a range, `xs[a..b]`.
    pub(super) fn slice_call_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        self.float_text_fix(ty, base, method, args, span);
        let (Ty::Str | Ty::List(_), "slice" | "substring", [a, b]) = (ty, method, args) else {
            return;
        };
        if !matches!(
            base.kind,
            ast::ExprKind::Name(_) | ast::ExprKind::Field { .. } | ast::ExprKind::Call { .. }
        ) {
            return;
        }
        let new = format!(
            "{}[{}..{}]",
            self.source(base.span),
            self.source(a.value.span),
            self.source(b.value.span)
        );
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!("a slice indexes a range of positions: `{new}`"));
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
        let message = format!("`parse` needs the type it reads: `{recv}.parse<{ty}>()`");
        let mut d = Diagnostic::error(codes::WRONG_ARG_COUNT, self.span(name.span), message);
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
/// C++'s names for putting a key in a map.
pub(super) fn foreign_insert(method: &str) -> bool {
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

pub(super) fn missing_type_args(name: &str, given: usize) -> String {
    if given == 0 {
        return format!("`{name}` needs its element type: `{name}<Int>`");
    }
    let args = vec!["Int"; given + 1].join(", ");
    format!("`{name}` takes more type arguments: `{name}<{args}>`")
}

/// The built-in type whose members `ty` has.
fn receiver_of(ty: &Ty) -> Option<cpl_check::Receiver> {
    use cpl_check::Receiver;
    match ty {
        Ty::Str | Ty::StrView => Some(Receiver::Str),
        Ty::List(_) | Ty::MutView(..) | Ty::ListView(..) | Ty::Keyed(..) => Some(Receiver::List),
        Ty::Map(..) | Ty::HashMap(..) => Some(Receiver::Map),
        Ty::Set(_) | Ty::HashSet(_) => Some(Receiver::Set),
        Ty::Array { .. } => Some(Receiver::Array),
        _ => None,
    }
}

/// The error for a built-in member called with arguments it doesn't take:
/// `xs.fold()` names what `fold` takes rather than saying it is missing.
pub(super) fn builtin_misuse(ty: &Ty, method: &str) -> Option<String> {
    let row = cpl_check::builtin_members(receiver_of(ty)?).find(|m| m.name == method)?;
    Some(format!(
        "`{method}` takes `{}`, as in `{}`",
        row.params, row.example
    ))
}
