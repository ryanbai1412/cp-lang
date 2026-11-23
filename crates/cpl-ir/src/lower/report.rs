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
