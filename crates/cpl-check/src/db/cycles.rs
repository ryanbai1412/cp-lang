//! How `Database` names a module import cycle and keeps it out of the cache.

use super::*;

impl Database {
    /// Every module import cycle met so far, as the paths around it with
    /// the first repeated last, including those an implicit import of the
    /// library skipped.
    pub fn import_cycles(&self) -> &[Vec<String>] {
        &self.cycles
    }

    /// A module that reached a file now being analyzed would meet a cycle
    /// there today, so reusing it would depend on who asked first.
    pub(super) fn reaches_open(&self, analysis: &Analysis) -> bool {
        analysis
            .scope
            .modules
            .iter()
            .any(|m| self.analyzing.contains(&m.path))
    }

    /// Records the cycle that importing `path`, open at `analyzing[at]`,
    /// closes, and returns the error message for that import.
    pub(super) fn meet_cycle(&mut self, at: usize, name: &str, path: &str) -> String {
        self.cycle_floor = self.cycle_floor.min(at);
        let mut cycle = self.analyzing[at..].to_vec();
        cycle.push(path.to_string());
        let message = if cycle.len() == 3 {
            format!("`{name}` imports this file, so they import each other")
        } else {
            format!("`{name}` imports this file back: {}", cycle_names(&cycle))
        };
        let ring = &cycle[..cycle.len() - 1];
        let start = (0..ring.len()).min_by_key(|&i| &ring[i]).unwrap_or(0);
        let mut around: Vec<String> = ring[start..]
            .iter()
            .chain(&ring[..start])
            .cloned()
            .collect();
        around.push(around[0].clone());
        self.cycle_hits += 1;
        self.last_cycle = around.clone();
        if !self.cycles.contains(&around) {
            self.cycles.push(around);
        }
        message
    }

    /// The error on a `use` of `module` whose cycle closed in a module it
    /// imports: a program never sees an imported module's own errors.
    pub(super) fn cycle_error(&self, module: &[ast::Name]) -> Option<Diagnostic> {
        let (first, last) = (module.first()?, module.last()?);
        let dotted: Vec<&str> = module.iter().map(|n| n.text.as_str()).collect();
        Some(
            Diagnostic::error(
                codes::UNRESOLVED_MODULE,
                first.span.to(last.span),
                format!(
                    "`{}` is in an import cycle: {}",
                    dotted.join("."),
                    cycle_names(&self.last_cycle)
                ),
            )
            .with_help(
                "move what the modules of the cycle share into one that imports none of them",
            ),
        )
    }
}

/// `a -> b -> a`, by file name.
fn cycle_names(cycle: &[String]) -> String {
    cycle
        .iter()
        .map(|p| {
            std::path::Path::new(p)
                .file_stem()
                .map_or(p.clone(), |s| s.to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}
