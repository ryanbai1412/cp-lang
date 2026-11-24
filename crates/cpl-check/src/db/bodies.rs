//! Body checks of an imported module, on demand. A module read through
//! `use` has its signatures checked, not its bodies (0f3ca2a93: the bodies
//! were most of the front end's time); a consumer that needs what a body
//! says, IR lowering a std fn, asks for that one body here.

use super::*;

impl Database {
    /// What checking found in the item of `path` around `at`, in file
    /// positions, checking its body if `path` was only imported.
    pub(crate) fn body_facts(&mut self, path: &str, at: Span) -> Option<Arc<crate::Facts>> {
        let file = self.files.get(path)?;
        let (id, analysis) = (file.id, file.analysis.clone()?);
        let index = (analysis.items).partition_point(|item| item.span.end < at.start);
        let item = analysis
            .items
            .get(index)
            .filter(|i| i.span.touches(at.start))?;
        let check = if file.analysis_checked {
            item.check.clone()
        } else {
            let mut ctx = FileCtx {
                db: self,
                file: id,
                items: analysis.syntax.clone(),
                scope: analysis.scope.clone(),
                frames: Vec::new(),
                in_progress: Vec::new(),
                current: None,
                offsets: analysis.items.iter().map(|i| i.offset).collect(),
                names: None,
                effects: HashMap::default(),
                own_effects: HashMap::default(),
                effects_of: Vec::new(),
                vars: None,
                impl_names: None,
            };
            ctx.check_item(index)
        };
        let facts = crate::Facts::from_checks(analysis.revision, &[(item.offset, &*check)]);
        Some(Arc::new(facts))
    }
}
