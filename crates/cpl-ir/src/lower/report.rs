//! Where a lowering error is shown, and what it says when it is inside
//! library code.

use super::*;

impl<'a> Lowerer<'a> {
    /// Where an error at `span` is shown. One inside library code is
    /// shown at the user's expression that asked for that code, since the
    /// file on screen is the user's.
    pub(super) fn span(&self, span: Span) -> Span {
        if self.cur_mod == 0 {
            span.shift(self.offset)
        } else {
            self.asked_at
        }
    }

    pub(super) fn error(&mut self, code: cpl_diag::Code, span: Span, message: impl Into<String>) {
        let at = span.shift(self.offset);
        let span = self.span(span);
        let mut message = message.into();
        let mut note = None;
        if self.cur_mod != 0 {
            if let Some((module, line, inside)) = self.library_place(at) {
                message = format!(
                    "`{inside}` (`{module}`) does not work on the types it was given here: {message}"
                );
                note = Some(format!(
                    "{}.cp:{line}: inside `{inside}`; the gap is in `{module}`, not in your code",
                    module.replace('.', "/")
                ));
            }
            self.in_library.insert(message.clone());
        }
        let diag = Diagnostic::error(code, span, message);
        self.diags.push(match note {
            Some(note) => diag.with_note(note),
            None => diag,
        });
    }

    /// `if let`/`!` on a value that is not optional. For `if let r = i - 1`
    /// with `i: D` the offset is an `Int`, and the fix is `D.at(i - 1)`,
    /// the `?D` that is `None` off the end.
    pub(super) fn not_optional(&mut self, what: &str, value: &Expr, span: Span) {
        let message = format!(
            "`{what}` takes the value out of an optional, but this is a `{}`",
            self.show(&value.ty)
        );
        self.error(codes::TYPE_MISMATCH, span, message);
        let ExprKind::Binary(_, lhs, _) = &value.kind else {
            return;
        };
        let ExprKind::Cast(Cast::Label(d), _) = lhs.kind else {
            return;
        };
        if self.cur_mod != 0 {
            return;
        }
        let domain = self.show(&Ty::Pos(d));
        let at = format!("{domain}.at({})", self.source(span));
        self.help(format!(
            "write `{at}`: a position plus an `Int` is an `Int`, and `{domain}.at` gives back a `?{domain}` that is `None` outside `{domain}`"
        ));
        let edit = (self.span(span), at.clone());
        if let Some(last) = self.diags.last_mut() {
            last.fixes.push(cpl_diag::Fix {
                title: format!("write `{at}`"),
                edits: vec![edit],
            });
        }
    }

    /// `type V = 0..n` then `V.first`: a range type has no positions, a
    /// `domain` does. `range` is the alias's right side, absolute in the
    /// user file; the fix rewrites its `type` keyword.
    pub(super) fn range_alias_as_domain(&mut self, name: &str, range: Span) {
        let text = &self.mods[0].text;
        let shown = text
            .get(range.start as usize..range.end as usize)
            .unwrap_or("")
            .to_string();
        let head = text.get(..range.start as usize).unwrap_or("");
        let keyword = head.rfind("type").filter(|&at| {
            let between = head[at + 4..].split_whitespace().collect::<Vec<_>>();
            between == [name, "="]
        });
        let help = format!(
            "`type {name} = {shown}` names a range of integers; `domain {name} = {shown}` declares positions, which have `{name}.first`, `{name}.last` and index `[{name}]T`"
        );
        let Some(last) = self.diags.last_mut() else {
            return;
        };
        last.help = Some(help);
        if let Some(at) = keyword {
            let at = at as u32;
            last.fixes.push(cpl_diag::Fix {
                title: format!("declare `domain {name}`"),
                edits: vec![(Span::new(at, at + 4), "domain".into())],
            });
        }
    }

    /// The module, one-based line and innermost declaration holding `at`,
    /// a position in the current library module.
    fn library_place(&self, at: Span) -> Option<(String, u32, String)> {
        let analysis = *self.mods.get(self.cur_mod)?;
        let module = &self
            .mods
            .iter()
            .flat_map(|m| m.modules())
            .find(|m| std::ptr::eq(&*m.analysis, analysis))?
            .name;
        let item = analysis
            .items
            .iter()
            .find(|item| item.span.touches(at.start))?;
        let inside = item
            .syntax
            .decls
            .iter()
            .filter(|d| d.span.shift(item.offset).touches(at.start))
            .min_by_key(|d| (d.kind != cpl_check::DeclKind::Fn, d.span.end - d.span.start))
            .or(item.syntax.decls.first())?;
        let line = analysis.lines.line_of(at.start) + 1;
        Some((module.clone(), line, inside.name.clone()))
    }
}
