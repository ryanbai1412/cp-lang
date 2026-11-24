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
        let check = if checked {
            item.check.clone()
        } else {
            self.check_body(id, &analysis, index)
        };
        let facts = crate::Facts::from_checks(&analysis, &[(item.offset, &*check)]);
        Some(Arc::new(facts))
    }

    /// Every body of `path` checked once, as facts: what lowering reads of a
    /// module the program imports. Kept until the file changes.
    pub fn module_facts(&mut self, path: &str) -> Option<Arc<crate::Facts>> {
        let file = self.files.get(path)?;
        if let Some(facts) = &file.module_facts {
            return Some(facts.clone());
        }
        let checked = file.program.is_some();
        let (id, analysis) = (
            file.id,
            file.program.clone().or_else(|| file.module.clone())?,
        );
        let checks: Vec<Arc<ItemCheck>> = (0..analysis.items.len())
            .map(|i| match checked {
                true => analysis.items[i].check.clone(),
                false => self.check_body(id, &analysis, i),
            })
            .collect();
        let items: Vec<(u32, &ItemCheck)> = (analysis.items.iter().zip(&checks))
            .map(|(item, check)| (item.offset, &**check))
            .collect();
        let facts = Arc::new(crate::Facts::from_checks(&analysis, &items));
        self.files.get_mut(path)?.module_facts = Some(facts.clone());
        Some(facts)
    }

    fn check_body(&mut self, file: u32, analysis: &Analysis, index: usize) -> Arc<ItemCheck> {
        let mut ctx = FileCtx {
            db: self,
            file,
            items: analysis.syntax.clone(),
            scope: analysis.scope.clone(),
            frame: Vec::new(),
            recording: false,
            in_progress: Vec::new(),
            current: None,
            offsets: analysis.items.iter().map(|i| i.offset).collect(),
            names: None,
            effects: Default::default(),
            own_effects: Default::default(),
            effects_of: Vec::new(),
            vars: None,
            impl_names: None,
        };
        ctx.check_item(index)
    }
}
