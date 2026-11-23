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
        const POS_MEMBERS: [&str; 19] = [
            "after", "ahead", "back", "before", "count", "cyc", "first", "idx", "is_empty",
            "label", "len", "next", "offset", "ord", "prev", "rank", "to_idx", "with", "without",
        ];
        const BOUND_EXTRA: [&str; 4] = ["l", "left", "r", "right"];
        const RANGE_MEMBERS: [&str; 10] = [
            "end", "first", "halves", "is_empty", "last", "mid", "rev", "sorted", "start", "step",
        ];
        const DOMAIN_MEMBERS: [&str; 24] = [
            "after", "all", "any", "at", "before", "bounds", "count", "cuts", "first", "from_idx",
            "is_empty", "last", "len", "mask", "max", "min", "nth", "rev", "size", "sort",
            "sorted", "subsets", "sum", "windows",
        ];
        const SEQ_EXTRA: [&str; 6] = [
            "adjacent", "inverse", "items", "resized", "reverse", "reversed",
        ];
        const STR_EXTRA: [&str; 6] = [
            "adjacent", "count", "items", "reverse", "reversed", "to_str",
        ];
        const INT_MEMBERS: [&str; 6] = ["abs", "idx", "isqrt", "ord", "sqrt", "to_idx"];
        const FLOAT_MEMBERS: [&str; 6] = ["abs", "ceil", "floor", "round", "sqrt", "trunc"];
        let receiver = match ty {
            Ty::Str | Ty::StrView => Some(Receiver::Str),
            Ty::List(_) | Ty::MutView(_) | Ty::ListView(..) | Ty::Keyed(..) => Some(Receiver::List),
            Ty::Map(..) | Ty::HashMap(..) => Some(Receiver::Map),
            Ty::Set(_) | Ty::HashSet(_) => Some(Receiver::Set),
            Ty::Array { .. } => Some(Receiver::Array),
            _ => None,
        };
        if let Some(own) = receiver.and_then(|r| cpl_check::foreign_member(r, name)) {
            return self.suggest_last(span, own.into());
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
            Ty::Pos(d) => {
                names.extend(POS_MEMBERS);
                if self.subset_base(*d).is_some() {
                    names.push("subsets");
                }
            }
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
            Ty::List(_) | Ty::Array { .. } | Ty::MutView(_) | Ty::ListView(..) | Ty::Keyed(..) => {
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
                names.push(sym_name.strip_suffix(" of std").unwrap_or(sym_name));
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
        let Some(s) = cpl_check::closest(name, names.into_iter()) else {
            return;
        };
        self.suggest_last(span, s);
    }

    /// `List.name(..)` with no such static fn: one error, and `filled` if
    /// it is a typo of that.
    pub(super) fn list_static_typo(&mut self, name: &ast::Name, span: Span) -> Expr {
        let message = format!(
            "a `List` has no method `{}` that `cpl run` knows",
            name.text
        );
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
        let last = self.diags.last_mut().filter(|d| {
            d.code == codes::TYPE_MISMATCH && d.span == at && d.message.ends_with("found `Str`")
        });
        if let Some(d) = last {
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
        let last = self.diags.last_mut().filter(|d| {
            d.code == codes::TYPE_MISMATCH
                && d.span == at
                && d.message.starts_with("expected `Str`")
        });
        if let Some(d) = last {
            d.help = Some(format!("`join` is called on the separator: `{new}`"));
            d.fix(format!("write `{new}`"), edit, new);
        }
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

    /// JavaScript's `x.toFixed(2)`: a float's digits are set in its
    /// interpolation, `"{x:.2}"` (R10).
    pub(super) fn to_fixed_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        let (Ty::Float, "toFixed", [d]) = (ty, method, args) else {
            return;
        };
        if !matches!(base.kind, ast::ExprKind::Name(_))
            || !matches!(d.value.kind, ast::ExprKind::Int(_))
        {
            return;
        }
        let new = format!(
            "\"{{{}:.{}}}\"",
            self.source(base.span),
            self.source(d.value.span)
        );
        let edit = self.span(span);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(format!(
                "a float's digits are set in its interpolation: `{new}`"
            ));
            last.fixes.clear();
            last.fix(format!("write `{new}`"), edit, new);
        }
    }

    /// Python's `sorted(xs, reverse=True)`: CP Lang's sort says `desc: true`.
    pub(super) fn sort_reverse_arg(&mut self, args: &[ast::Arg]) -> bool {
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

    /// JavaScript's `xs.slice(a, b)` or `s.substring(a, b)`: a slice indexes a range, `xs[a..b]`.
    pub(super) fn slice_call_fix(
        &mut self,
        ty: &Ty,
        base: &ast::Expr,
        method: &str,
        args: &[ast::Arg],
        span: Span,
    ) {
        self.to_fixed_fix(ty, base, method, args, span);
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
}
