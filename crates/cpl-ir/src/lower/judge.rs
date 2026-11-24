//! The fns that cross to the judge's grader, and the types they cross in
//! (R10.1.3, R10.1.4).

use super::*;

impl<'a> Lowerer<'a> {
    /// The fns that cross to the judge's grader: those it supplies (no
    /// body) and those it calls (`@grader fn` with a body), each checked
    /// to take and give only what a grader can.
    pub(super) fn grader_fns(&mut self, analysis: &'a Analysis) -> (Vec<FnId>, Vec<FnId>) {
        let entries: Vec<&ast::FnDecl> = analysis
            .items
            .iter()
            .flat_map(|item| &item.syntax.parsed.stmts)
            .filter_map(|stmt| match &stmt.kind {
                ast::StmtKind::Fn(decl)
                    if stmt.decorators.iter().any(|d| d.name.text == "grader") =>
                {
                    Some(decl)
                }
                _ => None,
            })
            .collect();
        let (mut judge_fns, mut entry_fns) = (Vec::new(), Vec::new());
        for id in 0..self.sigs.len() {
            let Some(f) = &self.bodies[id] else { continue };
            let decl = self.sigs[id].decl;
            let top_level = self.sigs[id].mod_id == 0 && self.sigs[id].owner.is_none();
            if matches!(decl.body, ast::FnBody::None) {
                judge_fns.push(id);
            } else if top_level && entries.iter().any(|entry| std::ptr::eq(*entry, decl)) {
                entry_fns.push(id);
            } else {
                continue;
            }
            let name = &decl.name.text;
            let changed = f.params.iter().position(|p| p.mutable);
            let tys: Vec<Ty> = f
                .locals
                .iter()
                .take(f.params.len())
                .map(|l| l.ty.clone())
                .collect();
            let ret = f.ret.clone();
            let saved = (self.cur_mod, self.offset);
            (self.cur_mod, self.offset) = (self.sigs[id].mod_id, self.sigs[id].offset);
            if let Some(at) = changed {
                let param = decl.params[at].name.text.clone();
                self.error(
                    codes::UNSUPPORTED,
                    decl.params[at].name.span,
                    format!("the judge's grader can't lend `{param}` to `{name}` to change"),
                );
                self.help(format!("take `{param}` by value and return what changed"));
            } else {
                let (params, result) = self.sig_ranges.get(&id).cloned().unwrap_or_default();
                let at = |i: usize| params.get(i).and_then(Option::as_ref);
                let sides: Result<Vec<JudgeTy>, Uncrossable> = (tys.iter().enumerate())
                    .map(|(i, ty)| judge_ty(ty, at(i)))
                    .collect();
                match sides.and_then(|sides| Ok((sides, judge_ty(&ret, result.as_ref())?))) {
                    Ok((sides, ret)) => self.judge_tys.push((id, sides, ret)),
                    Err(Uncrossable::Type(ty)) => {
                        let shown = self.show(&ty);
                        self.error(
                            codes::UNSUPPORTED,
                            decl.name.span,
                            format!("a `{shown}` can't cross to the judge's grader in `{name}`"),
                        );
                        self.help("a `@grader fn` takes and gives `Int`, `F64`, `Bool`, `Char`, `Str`, and lists and tuples of them");
                    }
                    Err(Uncrossable::Open(open)) => {
                        self.error(
                            codes::UNSUPPORTED,
                            decl.name.span,
                            format!("`{open}` can't cross to the judge's grader in `{name}`: no C++ integer holds a range with an open end"),
                        );
                        self.help("close the range, as in `0..=1e18`, or write `Int` for the grader's `int`");
                    }
                }
            }
            (self.cur_mod, self.offset) = saved;
        }
        (judge_fns, entry_fns)
    }
}
