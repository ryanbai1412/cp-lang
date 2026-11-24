//! Python habits lowering recovers: `xs.join(" ")`, `a.pop(i)`, `m.get(k, d)`,
//! `key=` and `line.split()`.

use super::*;

impl<'a> Lowerer<'a> {
    /// `xs.join(" ")`, Python's order: `join` is the separator's, `" ".join(xs)`.
    pub(in crate::lower) fn join_receiver(
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
            d.help = Some(habits::JOIN_RECEIVER.say(&[&new]));
            d.fix(habits::WRITE.say(&[&new]), edit, new);
        }
    }

    /// Python's `a.pop(i)` and `m.get(k, d)`, C++'s `a.resize(n)`: `a.remove(i)`,
    /// `m.get(k) ?? d`, `a.resize(n, 0)`.
    pub(in crate::lower) fn foreign_method_args(
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
            ("pop", [i]) if matches!(ty, Ty::List(_)) => {
                (format!("{b}.remove({i})"), &habits::LIST_POP_INDEX)
            }
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
                (format!("{b}.resize({n}, {zero})"), &habits::LIST_RESIZE)
            }
            ("get", [k, d]) => {
                let new = self.parenthesized(whole.span, format!("{b}.get({k}) ?? {d}"));
                (new, &habits::MAP_GET_DEFAULT)
            }
            _ => return self.err_expr(span),
        };
        let at = self.span(whole.span);
        if let Some(last) = self
            .diags
            .last_mut()
            .filter(|d| d.code == codes::WRONG_ARG_COUNT)
        {
            last.help = Some(help.say(&[&new]));
            last.fixes.clear();
            last.fix(habits::WRITE.say(&[&new]), at, new);
        }
        self.err_expr(span)
    }

    /// A named argument the callee lacks; suggests the label meant, including
    /// Python's names for the same option (`key=` and `cmp=` are `by:`).
    pub(in crate::lower) fn unknown_arg_name(
        &mut self,
        said: msgs::Said,
        name: &ast::Name,
        labels: &[&str],
    ) {
        self.report_said(name.span, said);
        let foreign = match name.text.as_str() {
            "key" | "cmp" => Some("by"),
            _ => None,
        };
        if let Some(label) = foreign.filter(|l| labels.contains(l)) {
            let at = self.span(name.span);
            if let Some(last) = self.diags.last_mut() {
                last.help = Some(habits::ARG_NAME.say(&[&label]));
                last.fix(habits::WRITE.say(&[&label]), at, label);
            }
        } else if let Some(label) = cpl_check::closest(&name.text, labels.iter().copied()) {
            self.suggest_last(name.span, label);
        }
    }

    /// Python's `line.split()`: CP Lang splits on one `Char`, and a line's
    /// tokens are split on `' '`.
    pub(in crate::lower) fn split_on_space(&mut self, name: &str, param: &str, call: Span) {
        if (name, param) != ("split", "sep") || !self.source(call).ends_with("()") {
            return;
        }
        let at = self.span(call);
        if let Some(last) = self.diags.last_mut() {
            last.help = Some(habits::SPLIT_ON_SPACE.say(&[]));
            last.fix(
                habits::WRITE.say(&[&"split(' ')"]),
                Span::point(at.end - 1),
                "' '",
            );
        }
    }
}
