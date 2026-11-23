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
        const DOMAIN_MEMBERS: [&str; 23] = [
            "after", "all", "any", "at", "before", "bounds", "count", "cuts", "first", "is_empty",
            "last", "len", "mask", "max", "min", "nth", "rev", "size", "sort", "sorted", "subsets",
            "sum", "windows",
        ];
        const SEQ_EXTRA: [&str; 6] = [
            "adjacent", "inverse", "items", "resized", "reverse", "reversed",
        ];
        const STR_EXTRA: [&str; 6] = [
            "adjacent", "count", "items", "reverse", "reversed", "to_str",
        ];
        const INT_MEMBERS: [&str; 5] = ["abs", "idx", "isqrt", "ord", "sqrt"];
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
        let message = "expected `Str`, found `List<Char>`: a `Str` method takes a list of characters as `Str(..)`";
        let mut d = Diagnostic::error(codes::TYPE_MISMATCH, at, message.to_string());
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
        let sym = self
            .get_sym(method)
            .or_else(|| self.get_sym(&cpl_check::std_method(method)));
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
