//! The one stream of events a run emits, heard by each observer asked
//! for: the tracer, the profiler and the VM's fallback counts
//! (`CPL_VM_STATS`) all implement [`Observer`], and [`Watch`] passes
//! every event to each of them. An internal call
//! ([`Interp::quietly`](crate::Interp)) hushes the tracer and the
//! profiler, not the fallback counts: a fallback there is still one.

use crate::observe::Observer;
use crate::profile::{Profile, Profiler};
use crate::trace::{Recording, Tracer, View};
use crate::vm::VmStats;
use cpl_diag::Span;
use cpl_ir::{Place, Program, Stmt};

pub(crate) struct Watch {
    tracer: Option<Tracer>,
    profiler: Option<Profiler>,
    stats: Option<VmStats>,
    /// Internal calls running: the user's steps are not heard.
    quiet: u32,
}

impl Watch {
    /// What watches a run traced as `tracer` asks, profiled with
    /// `profile`; none when nothing would.
    pub(crate) fn of(
        program: &Program,
        tracer: Option<Tracer>,
        profile: bool,
    ) -> Option<Box<Self>> {
        let profiler = profile.then(|| Profiler::new(program));
        let stats = VmStats::from_env();
        (tracer.is_some() || profiler.is_some() || stats.is_some()).then(|| {
            Box::new(Watch {
                tracer,
                profiler,
                stats,
                quiet: 0,
            })
        })
    }

    fn live(&mut self) -> Option<&mut Tracer> {
        self.tracer.as_mut().filter(|_| self.quiet == 0)
    }

    fn seen(&self) -> Option<&Tracer> {
        self.tracer.as_ref().filter(|_| self.quiet == 0)
    }
}

impl Observer for Profiler {
    fn traces(&self) -> bool {
        false
    }
    fn times_steps(&self) -> bool {
        true
    }
    fn step(&mut self, stmt: &Stmt) {
        self.enter(stmt)
    }
}

impl Observer for VmStats {
    fn traces(&self) -> bool {
        false
    }
    fn walked(&mut self, stmt: &Stmt) {
        self.walked += 1;
        *self
            .walked_counts
            .entry(crate::vm::stmt_tag(stmt))
            .or_default() += 1;
    }
    fn fell_back(&mut self, tag: impl FnOnce() -> String, in_loop: bool) {
        self.record_with_loop(tag(), in_loop);
    }
    fn finish(self: Box<Self>, _view: &View, _failed: Option<(&str, Span)>) -> Option<Recording> {
        self.emit_stderr();
        None
    }
}

impl Observer for Watch {
    fn traces(&self) -> bool {
        self.seen().is_some()
    }
    fn times_steps(&self) -> bool {
        self.quiet == 0 && self.profiler.is_some()
    }
    fn hush(&mut self, on: bool) -> bool {
        match on {
            true => self.quiet += 1,
            false => self.quiet -= 1,
        }
        true
    }
    fn step(&mut self, stmt: &Stmt) {
        if let (Some(profiler), 0) = (self.profiler.as_mut(), self.quiet) {
            profiler.step(stmt);
        }
    }
    fn walked(&mut self, stmt: &Stmt) {
        if let Some(stats) = self.stats.as_mut() {
            stats.walked(stmt);
        }
    }
    fn fell_back(&mut self, tag: impl FnOnce() -> String, in_loop: bool) {
        if let Some(stats) = self.stats.as_mut() {
            stats.fell_back(tag, in_loop);
        }
    }
    fn profile(&mut self) -> Option<Profile> {
        self.profiler.take().map(Profiler::finish)
    }
    fn steps(&self) -> usize {
        self.seen().map_or(0, Observer::steps)
    }
    fn begin_cases(&mut self) {
        if let Some(t) = self.live() {
            t.begin_cases();
        }
    }
    fn divider(&mut self) {
        if let Some(t) = self.live() {
            t.divider();
        }
    }
    fn start_case(&mut self, case: u64) {
        if let Some(t) = self.live() {
            t.start_case(case);
        }
    }
    fn resume_case(&mut self, case: u64) {
        if let Some(t) = self.live() {
            t.resume_case(case);
        }
    }
    fn enter_fn(&mut self, name: &str) -> (bool, bool) {
        self.live().map_or((false, false), |t| t.enter_fn(name))
    }
    fn leave_fn(&mut self, entered: (bool, bool)) {
        if let Some(t) = self.live() {
            t.leave_fn(entered);
        }
    }
    fn steps_over(&self, name: &str) -> bool {
        self.seen().is_some_and(|t| t.steps_over(name))
    }
    fn over_library(&self) -> bool {
        self.seen().is_some_and(Observer::over_library)
    }
    fn stepped_over(&mut self, view: &View, span: Span) {
        if let Some(t) = self.live() {
            t.stepped_over(view, span);
        }
    }
    fn globals_may_change(&mut self) {
        if let Some(t) = self.live() {
            t.globals_may_change();
        }
    }
    fn enter_call(&mut self, name: &str, args: impl FnOnce() -> String) {
        if let Some(t) = self.live() {
            t.enter_call(name, args);
        }
    }
    fn keeps_call(&self) -> bool {
        self.seen().is_some_and(Observer::keeps_call)
    }
    fn leave_call(&mut self, returned: Option<String>) {
        if let Some(t) = self.live() {
            t.leave_call(returned);
        }
    }
    fn stopped(&self) -> bool {
        self.seen().is_some_and(Observer::stopped)
    }
    fn assignments(&self) -> bool {
        self.seen().is_some_and(Observer::assignments)
    }
    fn keeps_step(&self, depth: usize) -> bool {
        self.seen().is_some_and(|t| t.keeps_step(depth))
    }
    fn quiet_write(&mut self, place: &Place, depth: usize) -> bool {
        self.live().is_some_and(|t| t.quiet_write(place, depth))
    }
    fn declared(&mut self, id: usize, view: &View, at: Span) {
        if let Some(t) = self.live() {
            t.declared(id, view, at);
        }
    }
    fn changed(&mut self, id: usize, view: &View, at: Span, path: &[usize]) {
        if let Some(t) = self.live() {
            t.changed(id, view, at, path);
        }
    }
    fn mutated(&mut self, id: usize, view: &View, at: Span) {
        if let Some(t) = self.live() {
            t.mutated(id, view, at);
        }
    }
    fn local_changed(&mut self, view: &View, at: Span, name: &str) {
        if let Some(t) = self.live() {
            t.local_changed(view, at, name);
        }
    }
    fn local_changed_unseen(&mut self, depth: usize, name: &str) -> bool {
        self.live()
            .is_some_and(|t| t.local_changed_unseen(depth, name))
    }
    fn changed_unseen(&mut self, id: usize, name: &str, depth: usize, path: &[usize]) -> bool {
        self.live()
            .is_some_and(|t| t.changed_unseen(id, name, depth, path))
    }
    fn declared_unseen(
        &mut self,
        id: usize,
        name: &str,
        unnamed: bool,
        depth: usize,
        at: Span,
    ) -> bool {
        self.live()
            .is_some_and(|t| t.declared_unseen(id, name, unnamed, depth, at))
    }
    fn assigned_local(&mut self, view: &View, at: Span, name: &str) {
        if let Some(t) = self.live() {
            t.assigned_local(view, at, name);
        }
    }
    fn assigned_global(&mut self, view: &View, at: Span, id: usize) {
        if let Some(t) = self.live() {
            t.assigned_global(view, at, id);
        }
    }
    fn printed(&mut self, view: &View, at: Span, text: &str) {
        if let Some(t) = self.live() {
            t.printed(view, at, text);
        }
    }
    fn debugged(&mut self, view: &View, at: Span, name: &str, text: String, json: String) {
        if let Some(t) = self.live() {
            t.debugged(view, at, name, text, json);
        }
    }
    fn raw_write(&mut self, id: usize, view: &View, at: Span, path: &[usize]) -> bool {
        self.live().is_some_and(|t| t.raw_write(id, view, at, path))
    }
    fn finish(self: Box<Self>, view: &View, failed: Option<(&str, Span)>) -> Option<Recording> {
        if let Some(stats) = self.stats {
            Box::new(stats).finish(view, failed);
        }
        self.tracer.and_then(|t| Box::new(t).finish(view, failed))
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;

    fn watch(profiler: Option<Profiler>) -> Watch {
        Watch {
            tracer: None,
            profiler,
            stats: Some(VmStats::default()),
            quiet: 0,
        }
    }

    #[test]
    fn an_internal_call_still_counts_its_fallbacks() {
        let mut watch = watch(None);
        assert!(watch.hush(true));
        watch.fell_back(|| "stmt:Expr(Call)".into(), true);
        watch.hush(false);
        watch.fell_back(|| "expr:Read".into(), false);
        let counts = (watch.stats.as_ref()).map(|s| (s.total_fallbacks, s.loop_fallbacks));
        assert_eq!(counts, Some((2, 1)));
    }

    #[test]
    fn an_internal_call_is_not_profiled() {
        let program = Program::default();
        let mut watch = watch(Some(Profiler::new(&program)));
        assert!(watch.times_steps() && !watch.traces());
        watch.hush(true);
        assert!(!watch.times_steps());
        watch.hush(false);
        assert!(watch.times_steps());
    }
}
