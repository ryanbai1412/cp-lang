//! The one stream of events a run emits, heard by each observer asked
//! for: the tracer, the profiler and the VM's fallback counts
//! (`CPL_VM_STATS`) all implement [`Observer`], and [`Watch`] passes
//! every event to each of them. An internal call
//! ([`Interp::quietly`](crate::Interp)) hushes the tracer and the
//! profiler, not the fallback counts: a fallback there is still one.

#[cfg(not(target_family = "wasm"))]
use crate::{
    observe::{Event, Observer},
    profile::{Profile, Profiler},
    trace::{Recording, Tracer, View},
    vm::VmStats,
};
#[cfg(not(target_family = "wasm"))]
use cpl_diag::Span;
#[cfg(not(target_family = "wasm"))]
use cpl_ir::Program;

/// The browser has no profiler and no fallback counts: its watch is the
/// tracer alone, so `cpl.wasm` carries no events for observers it lacks.
#[cfg(target_family = "wasm")]
pub(crate) type Watch = crate::trace::Tracer;

#[cfg(target_family = "wasm")]
impl crate::trace::Tracer {
    /// [`Watch::of`] in the browser: the tracer, when the run is traced.
    pub(crate) fn of(
        _program: &cpl_ir::Program,
        tracer: Option<Self>,
        _profile: bool,
    ) -> Option<Box<Self>> {
        tracer.map(Box::new)
    }
}

#[cfg(not(target_family = "wasm"))]
pub(crate) struct Watch {
    tracer: Option<Tracer>,
    profiler: Option<Profiler>,
    stats: Option<VmStats>,
    /// Internal calls running: the user's steps are not heard.
    quiet: u32,
}

#[cfg(not(target_family = "wasm"))]
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

    /// Every event but the hot writes [`Observer::on`] takes first.
    fn dispatch(&mut self, event: Event<'_, '_>) -> bool {
        match event {
            Event::Hush(true) => self.quiet += 1,
            Event::Hush(false) => self.quiet -= 1,
            Event::Step(_) => {
                if let (Some(profiler), 0) = (self.profiler.as_mut(), self.quiet) {
                    profiler.on(event);
                }
            }
            Event::Walked(_) | Event::FellBack { .. } => {
                if let Some(stats) = self.stats.as_mut() {
                    stats.on(event);
                }
            }
            _ => return self.live().is_some_and(|t| t.on(event)),
        }
        true
    }
}

#[cfg(not(target_family = "wasm"))]
impl Observer for Profiler {
    fn traces(&self) -> bool {
        false
    }
    fn times_steps(&self) -> bool {
        true
    }
    fn on(&mut self, event: Event<'_, '_>) -> bool {
        if let Event::Step(stmt) = event {
            self.enter(stmt);
        }
        false
    }
}

#[cfg(not(target_family = "wasm"))]
impl Observer for VmStats {
    fn traces(&self) -> bool {
        false
    }
    fn on(&mut self, event: Event<'_, '_>) -> bool {
        match event {
            Event::Walked(stmt) => {
                self.walked += 1;
                *self
                    .walked_counts
                    .entry(crate::vm::stmt_tag(stmt))
                    .or_default() += 1;
            }
            Event::FellBack { tag, in_loop } => self.record_with_loop(tag(), in_loop),
            _ => {}
        }
        false
    }
    fn finish(self: Box<Self>, _view: &View, _failed: Option<(&str, Span)>) -> Option<Recording> {
        self.emit_stderr();
        None
    }
}

#[cfg(not(target_family = "wasm"))]
impl Observer for Watch {
    fn traces(&self) -> bool {
        self.seen().is_some()
    }
    fn times_steps(&self) -> bool {
        self.quiet == 0 && self.profiler.is_some()
    }
    // The writes every traced step asks about go to the tracer without a
    // second dispatch, so the VM's hot loop inlines them.
    #[inline(always)]
    fn on(&mut self, event: Event<'_, '_>) -> bool {
        match event {
            Event::QuietWrite { place, depth } => {
                self.live().is_some_and(|t| t.quiet_write(place, depth))
            }
            Event::IntWrite(write) => self.live().is_some_and(|t| t.int_write(write)),
            _ => self.dispatch(event),
        }
    }
    fn profile(&mut self) -> Option<Profile> {
        self.profiler.take().map(Profiler::finish)
    }
    fn steps(&self) -> usize {
        self.seen().map_or(0, Observer::steps)
    }
    fn steps_over(&self, name: &str) -> bool {
        self.seen().is_some_and(|t| t.steps_over(name))
    }
    fn over_library(&self) -> bool {
        self.seen().is_some_and(Observer::over_library)
    }
    fn keeps_call(&self) -> bool {
        self.seen().is_some_and(Observer::keeps_call)
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
        assert!(watch.on(Event::Hush(true)));
        watch.on(Event::FellBack {
            tag: &|| "stmt:Expr(Call)".into(),
            in_loop: true,
        });
        watch.on(Event::Hush(false));
        watch.on(Event::FellBack {
            tag: &|| "expr:Read".into(),
            in_loop: false,
        });
        let counts = (watch.stats.as_ref()).map(|s| (s.total_fallbacks, s.loop_fallbacks));
        assert_eq!(counts, Some((2, 1)));
    }

    #[test]
    fn an_internal_call_is_not_profiled() {
        let program = Program::default();
        let mut watch = watch(Some(Profiler::new(&program)));
        assert!(watch.times_steps() && !watch.traces());
        watch.on(Event::Hush(true));
        assert!(!watch.times_steps());
        watch.on(Event::Hush(false));
        assert!(watch.times_steps());
    }
}
