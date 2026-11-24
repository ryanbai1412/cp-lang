//! Body checks of an imported module, on demand. A module read through
//! `use` has its signatures checked, not its bodies (0f3ca2a93: the bodies
//! were most of the front end's time); a consumer that needs what a body
//! says, IR lowering a std fn, asks for that one body here.

use super::*;

impl Database {
    /// What checking found in the item of `path` around `at`, in file
    /// positions, checking its body if `path` was only imported. Each item's
    /// facts are worked out once per revision; LSP and lowering share them.
    pub(crate) fn body_facts(&mut self, path: &str, at: Span) -> Option<Arc<crate::Facts>> {
        let file = self.files.get(path)?;
        let checked = file.program.is_some();
        let (id, analysis) = (
            file.id,
            file.program.clone().or_else(|| file.module.clone())?,
        );
        let index = (analysis.items).partition_point(|item| item.span.end < at.start);
        let item = analysis
            .items
            .get(index)
            .filter(|i| i.span.touches(at.start))?;
        if let Some(Some(facts)) = file.body_facts.get(index) {
            return Some(facts.clone());
        }
        let check = if checked {
            item.check.clone()
        } else {
            self.check_body(id, &analysis, index)
        };
        let facts = Arc::new(crate::Facts::from_checks(
            &analysis,
            &[(item.offset, &*check)],
        ));
        let slots = &mut self.files.get_mut(path)?.body_facts;
        if slots.len() <= index {
            slots.resize_with(index + 1, || None);
        }
        slots[index] = Some(facts.clone());
        Some(facts)
    }

    fn check_body(&mut self, file: u32, analysis: &Analysis, index: usize) -> Arc<ItemCheck> {
        let offsets = analysis.items.iter().map(|i| i.offset).collect();
        let mut ctx = FileCtx::new(
            self,
            file,
            analysis.syntax.clone(),
            analysis.scope.clone(),
            offsets,
        );
        ctx.check_item(index)
    }
}
