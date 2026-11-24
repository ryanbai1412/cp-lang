//! What a run tells whoever watches it: the interface the debugger needs
//! from the interpreter. The interpreter is built once per kind of
//! [`Observer`]: an untraced run with [`Unobserved`], whose hooks compile
//! away, a watched one with [`crate::watch::Watch`].

pub(crate) use crate::event::Event;
use crate::trace::{Recording, Tracer, View};
use crate::Value;
use crate::{leading_indices, relifetime, trace, Interp};
use cpl_diag::Span;
use cpl_ir::{Place, PlaceKind, Stmt};

/// What an observer hears of a run: each [`Event`], through
/// [`Observer::on`], and the questions the run asks before it spends work
/// on an event no one keeps.
pub(crate) trait Observer {
    /// False compiles every event out of the run.
    const ON: bool = true;
    /// Whether it hears the user's steps: the run is traced.
    fn traces(&self) -> bool {
        true
    }
    /// Whether it times each [`Event::Step`]: the run is profiled.
    fn times_steps(&self) -> bool {
        false
    }
    /// One event of the run; true when the observer took note of a write
    /// with no view, or hushes itself for [`Event::Hush`].
    fn on(&mut self, _event: Event<'_, '_>) -> bool {
        false
    }
    /// Where the run spent its time, once it is over.
    fn profile(&mut self) -> Option<crate::profile::Profile> {
        None
    }
    /// Whether the trace steps over a call of `name` (see
    /// [`Tracer::steps_over`]).
    fn steps_over(&self, _name: &str) -> bool {
        false
    }
    /// Inside a call the trace steps over.
    fn over_library(&self) -> bool {
        false
    }
    /// Whether [`Event::LeaveCall`] wants the returned value.
    fn keeps_call(&self) -> bool {
        false
    }
    /// The observer has seen enough: the run stops.
    fn stopped(&self) -> bool {
        false
    }
    /// Whether every assignment is a step, not only changes.
    fn assignments(&self) -> bool {
        false
    }
    /// Whether a step at call depth `depth` would be kept: what a step
    /// costs before the observer sees it (`impl Show` calls) is paid only then.
    fn keeps_step(&self, _depth: usize) -> bool {
        true
    }
    /// Steps seen so far, kept or not.
    fn steps(&self) -> usize {
        0
    }
    /// The run is over, or trapped at `failed`.
    fn finish(self: Box<Self>, _view: &View, _failed: Option<(&str, Span)>) -> Option<Recording> {
        None
    }
}

/// No one watches: the interpreter built for it has no hooks.
pub(crate) struct Unobserved;

/// The observer of an untraced run. Native builds compile a copy of the
/// interpreter with no hooks; the browser reuses the traced copy with no
/// tracer, a branch per hook, so `cpl.wasm` carries one interpreter, not two.
#[cfg(not(target_family = "wasm"))]
pub(crate) type Untraced = Unobserved;
#[cfg(target_family = "wasm")]
pub(crate) type Untraced = crate::watch::Watch;

impl Observer for Unobserved {
    const ON: bool = false;
}

impl Observer for Tracer {
    fn on(&mut self, event: Event<'_, '_>) -> bool {
        match event {
            Event::Step(_) | Event::Walked(_) | Event::Hush(_) => return false,
            #[cfg(not(target_family = "wasm"))]
            Event::FellBack { .. } => return false,
            Event::BeginCases => self.case = Some(0),
            Event::Divider => self.divider(),
            Event::StartCase(case) => self.start_case(case),
            Event::ResumeCase(case) => self.case = Some(case),
            Event::EnterFn(name) => self.enter_fn(name),
            Event::LeaveFn => self.leave_fn(),
            Event::EnterFedBody => self.enter_fed_body(),
            Event::LeaveFedBody => self.leave_fed_body(),
            Event::SteppedOver { view, at } => self.stepped_over(view, at),
            Event::GlobalsMayChange => self.touch_all(),
            Event::EnterCall { name, args } => self.enter_call(name, args),
            Event::LeaveCall(returned) => self.leave_call(returned),
            Event::Declared { id, view, at } => self.declared(id, view, at),
            Event::Changed { id, view, at, path } => self.changed(id, view, at, path),
            Event::Mutated { id, view, at } => self.mutated(id, view, at),
            Event::LocalChanged { view, at, name } => self.local_changed(view, at, name),
            Event::AssignedLocal { view, at, name } => self.assigned_local(view, at, name),
            Event::AssignedGlobal { view, at, id } => self.assigned_global(view, at, id),
            Event::Printed { view, at, text } => self.printed(view, at, text),
            Event::Debugged {
                view,
                at,
                name,
                text,
                json,
            } => self.debugged(view, at, name, text, json),
            Event::QuietWrite { place, depth } => return self.quiet_write(place, depth),
            Event::LocalChangedUnseen { depth, name } => {
                return self.local_changed_unseen(depth, name)
            }
            Event::ChangedUnseen {
                id,
                name,
                depth,
                path,
            } => return self.changed_unseen(id, name, depth, path),
            Event::DeclaredUnseen {
                id,
                name,
                unnamed,
                depth,
                at,
            } => return self.declared_unseen(id, name, unnamed, depth, at),
            Event::IntWrite(write) => return self.int_write(write),
        }
        false
    }
    fn steps(&self) -> usize {
        self.steps_so_far()
    }
    fn steps_over(&self, name: &str) -> bool {
        Tracer::steps_over(self, name)
    }
    fn over_library(&self) -> bool {
        Tracer::over_library(self)
    }
    fn keeps_call(&self) -> bool {
        Tracer::keeps_call(self)
    }
    fn stopped(&self) -> bool {
        self.stopped
    }
    fn assignments(&self) -> bool {
        self.assignments
    }
    fn keeps_step(&self, depth: usize) -> bool {
        self.recording(depth)
    }
    fn finish(self: Box<Self>, view: &View, failed: Option<(&str, Span)>) -> Option<Recording> {
        Some(Tracer::finish(*self, view, failed))
    }
}

impl<O: Observer> Interp<'_, '_, O> {
    /// Whether the run is watched; false at compile time when unobserved.
    pub(crate) fn observed(&self) -> bool {
        O::ON && self.tracer.as_ref().is_some_and(|o| o.traces())
    }

    /// Whether each user statement is a timed step: the run is profiled.
    pub(crate) fn times_steps(&self) -> bool {
        O::ON && self.tracer.as_ref().is_some_and(|o| o.times_steps())
    }

    /// A user statement starts (see [`Event::Step`]).
    pub(crate) fn step_event(&mut self, stmt: &Stmt) {
        if let (true, Some(o)) = (O::ON, self.tracer.as_mut()) {
            o.on(Event::Step(stmt));
        }
    }

    /// The tree-walker runs `stmt` (see [`Event::Walked`]).
    pub(crate) fn walked_event(&mut self, stmt: &Stmt) {
        if let (true, Some(o)) = (O::ON, self.tracer.as_mut()) {
            o.on(Event::Walked(stmt));
            o.on(Event::Step(stmt));
        }
    }

    /// The VM fell back to the tree-walker (see [`Event::FellBack`]).
    /// Only `CPL_VM_STATS` reads the tag, and `cpl.wasm` has no stats: there
    /// the tag is never made into a `dyn Fn`, so its formatting isn't linked.
    pub(crate) fn fell_back(&mut self, tag: impl Fn() -> String, in_loop: bool) {
        #[cfg(target_family = "wasm")]
        let _ = (tag, in_loop);
        #[cfg(not(target_family = "wasm"))]
        if let (true, Some(o)) = (O::ON, self.tracer.as_mut()) {
            o.on(Event::FellBack { tag: &tag, in_loop });
        }
    }

    /// Runs `run` unwatched: an internal call, such as a comparator's
    /// strict-weak-order check, is no step of the user's program, so no
    /// observer (tracer or profiler) hears of it.
    pub(crate) fn quietly<R>(&mut self, run: impl FnOnce(&mut Self) -> R) -> R {
        let hushed = self.hush();
        let out = run(self);
        self.unhush(hushed);
        out
    }

    /// The start of an internal call (see [`Interp::quietly`]): the
    /// observer, when it does not hush itself.
    pub(crate) fn hush(&mut self) -> Option<Box<O>> {
        match self.tracer.as_mut().map(|o| o.on(Event::Hush(true))) {
            Some(true) => None,
            _ => self.tracer.take(),
        }
    }

    /// The end of the internal call [`Interp::hush`] started.
    pub(crate) fn unhush(&mut self, taken: Option<Box<O>>) {
        match (taken, self.tracer.as_mut()) {
            (Some(o), _) => self.tracer = Some(o),
            (None, Some(o)) => {
                o.on(Event::Hush(false));
            }
            (None, None) => {}
        }
    }

    /// A trap once the tracer has what it wanted, as after the call to replay.
    pub(crate) fn unstopped(&self, span: Span) -> crate::Res<()> {
        match &self.tracer {
            Some(tracer) if tracer.stopped() => {
                crate::trap(span, cpl_diag::msgs::interp::STOPPED.at(&[]))
            }
            _ => Ok(()),
        }
    }

    /// A call of `id` from `span` returned: a call the trace stepped over
    /// that took a `mut` argument gets the step at its call.
    pub(crate) fn after_call(&mut self, id: cpl_ir::FnId, span: Span) {
        let f = &self.program.fns[id];
        if self.observed()
            && f.params.iter().any(|p| p.mutable)
            && (self.tracer.as_ref()).is_some_and(|t| t.steps_over(&f.name) && !t.over_library())
        {
            self.traced(|tracer, view| {
                tracer.on(Event::SteppedOver { view, at: span });
            });
        }
    }

    /// Runs `record` on the tracer, when the run is traced.
    pub(crate) fn traced(&mut self, record: impl FnOnce(&mut O, &trace::View)) {
        if !self.observed() {
            return;
        }
        if let Some(mut tracer) = self.tracer.take() {
            if tracer.keeps_step(self.call_frames.len()) {
                self.fill_shown();
            }
            let locals = relifetime(std::mem::take(&mut self.view_locals));
            let view = self.view_in(locals);
            record(&mut tracer, &view);
            self.view_locals = relifetime(view.locals);
            self.tracer = Some(tracer);
        }
    }

    /// A step for a change to `place`, when it is in a top-level variable.
    pub(crate) fn trace_change(&mut self, place: &Place, indices: &[Value]) {
        if !self.observed() {
            return;
        }
        let Some(global) = place.root_global() else {
            if let Some(slot) = place.root_local() {
                // A view or a reference writes through to what it names.
                let through = matches!(
                    self.stack[self.base + slot],
                    Value::MutView(_) | Value::Part(_) | Value::Alias(..)
                );
                if let (true, Some(tracer)) = (through, self.tracer.as_mut()) {
                    tracer.on(Event::GlobalsMayChange);
                }
                self.trace_local(slot, place.span);
            }
            return;
        };
        if let PlaceKind::Global(id) = &place.kind {
            let id = *id;
            if self.trace_global_unseen(place, &[]) {
                return;
            }
            return self.traced(|tracer, view| {
                tracer.on(Event::Declared {
                    id,
                    view,
                    at: place.span,
                });
            });
        }
        let mut path = std::mem::take(&mut self.trace_path);
        path.clear();
        path.extend(
            indices
                .iter()
                .take(leading_indices(place))
                .map(|index| index.int().max(0) as usize),
        );
        if !self.trace_global_unseen(place, &path) {
            self.traced(|tracer, view| {
                tracer.on(Event::Changed {
                    id: global,
                    view,
                    at: place.span,
                    path: &path,
                });
            });
        }
        self.trace_path = path;
    }

    /// A write to global `place`, a cell of it at `path`, when the tracer
    /// needs no view of it, as for a step it skips: true when it is done.
    pub(crate) fn trace_global_unseen(&mut self, place: &Place, path: &[usize]) -> bool {
        if !self.observed() || self.assignments() {
            return false;
        }
        let Some(id) = place.root_global() else {
            return false;
        };
        let depth = self.call_frames.len();
        let global = &self.program.globals[id];
        let Some(tracer) = self.tracer.as_mut() else {
            return false;
        };
        match place.kind {
            PlaceKind::Global(_) => {
                let unnamed = global.span.end == 0;
                tracer.on(Event::DeclaredUnseen {
                    id,
                    name: &global.name,
                    unnamed,
                    depth,
                    at: place.span,
                })
            }
            _ => tracer.on(Event::ChangedUnseen {
                id,
                name: &global.name,
                depth,
                path,
            }),
        }
    }

    /// A write to global `place` that the observer takes note of without
    /// a view, as when the running fn is not traced.
    pub(crate) fn quiet_write(&mut self, place: &Place) -> bool {
        let depth = self.call_frames.len();
        match self.tracer.as_mut() {
            Some(tracer) if O::ON => tracer.on(Event::QuietWrite { place, depth }),
            _ => false,
        }
    }

    pub(crate) fn assignments(&self) -> bool {
        O::ON
            && self
                .tracer
                .as_ref()
                .is_some_and(|tracer| tracer.assignments())
    }

    /// With [`trace::TraceConfig::assignments`], the value of the
    /// variable `place` is in, which was just given a value.
    /// `mut` work on `place` (`push`, `sort`, ...) is done: a step for it.
    pub(crate) fn trace_mutated(&mut self, place: &Place, span: Span) {
        match place.root_global() {
            Some(global) if self.observed() => self.traced(|tracer, view| {
                tracer.on(Event::Mutated {
                    id: global,
                    view,
                    at: span,
                });
            }),
            _ => {
                if let Some(slot) = place.root_local() {
                    self.trace_local(slot, span);
                }
            }
        }
        self.trace_assigned(place);
    }

    pub(crate) fn trace_assigned(&mut self, place: &Place) {
        if !self.assignments() {
            return;
        }
        let span = place.span;
        if let Some(id) = place.root_global() {
            self.traced(|tracer, view| {
                tracer.on(Event::AssignedGlobal { view, at: span, id });
            });
        } else if let Some(slot) = place.root_local() {
            self.trace_assigned_local(slot, span);
        }
    }

    /// A step for local `slot` of a running call, which just changed.
    pub(crate) fn trace_local(&mut self, slot: usize, span: Span) {
        // Top-level locals are loop counters and the like: no step of their own.
        if !self.observed() || self.call_frames.is_empty() || self.trace_local_unseen(slot) {
            return;
        }
        let name = self.locals()[slot].name.as_str();
        self.traced(|tracer, view| {
            tracer.on(Event::LocalChanged {
                view,
                at: span,
                name,
            });
        });
    }

    /// [`Interp::trace_local`] in a call, and with no assignments traced,
    /// when the tracer needs no view of it, as for a step it skips: true
    /// when it is done.
    pub(crate) fn trace_local_unseen(&mut self, slot: usize) -> bool {
        if !self.observed() || self.call_frames.is_empty() || self.assignments() {
            return false;
        }
        let depth = self.call_frames.len();
        let name = self.locals()[slot].name.as_str();
        self.tracer
            .as_mut()
            .is_some_and(|tracer| tracer.on(Event::LocalChangedUnseen { depth, name }))
    }

    pub(crate) fn trace_assigned_local(&mut self, slot: usize, span: Span) {
        if !self.assignments() {
            return;
        }
        let name = self.locals()[slot].name.as_str();
        self.traced(|tracer, view| {
            tracer.on(Event::AssignedLocal {
                view,
                at: span,
                name,
            });
        });
    }
}

impl Recording {
    /// What a trace of this run holds, as its document's description says it.
    pub fn note(&self) -> &'static str {
        if self.skipped > 0 {
            "A window of the run from a later step: under `--live`, Earlier steps and Later steps load the rest."
        } else if self.truncated {
            "The run was longer than a trace holds: the first steps are kept."
        } else {
            "Every change to a top-level variable, every print and every debug(...)."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{run_as, Io, Limits};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// The debugger's side of the contract: what it hears of a run, in order.
    struct Heard(Rc<RefCell<Vec<String>>>);

    impl Heard {
        fn say(&self, what: String) {
            self.0.borrow_mut().push(what);
        }
    }

    impl Observer for Heard {
        fn on(&mut self, event: Event<'_, '_>) -> bool {
            let names = |view: &View, id: usize| view.program.globals[id].name.clone();
            match event {
                Event::Declared { id, view, .. } => {
                    self.say(format!("declared {}", names(view, id)));
                }
                Event::Changed { id, view, path, .. } => {
                    self.say(format!("changed {}{path:?}", names(view, id)));
                }
                Event::EnterCall { name, .. } => self.say(format!("enter {name}")),
                Event::LeaveCall(_) => self.say("leave".into()),
                Event::LocalChanged { view, name, .. } => {
                    self.say(format!("local {name} at depth {}", view.depth));
                }
                Event::Printed { text, .. } => self.say(format!("printed {}", text.trim_end())),
                _ => {}
            }
            false
        }
        fn finish(
            self: Box<Self>,
            _view: &View,
            failed: Option<(&str, Span)>,
        ) -> Option<Recording> {
            self.say(format!(
                "finish {}",
                failed.map_or("ok", |(message, _)| message)
            ));
            None
        }
    }

    fn program(source: &str) -> cpl_ir::Program {
        let mut db = cpl_check::Session::new();
        db.set_file("t.cp", source);
        let analysis = db.analyze("t.cp").expect("the file was just set");
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        cpl_ir::lower(&analysis).expect("lowers")
    }

    const SOURCE: &str = "\
var a: [0..3]Int = 0
fn bump(i: Int) -> Int:
    let j = i + 1
    return j
a[1] = bump(1)
print(a[1])
";

    #[test]
    fn an_observer_hears_globals_calls_locals_and_prints_in_order() {
        let program = program(SOURCE);
        let heard = Rc::new(RefCell::new(Vec::new()));
        let limits = Limits {
            stack_bytes: 1 << 22,
        };
        let (io, mut out) = (Io::fixed(b""), Vec::new());
        let observer = Some(Box::new(Heard(heard.clone())));
        let observed = run_as(&program, io, &mut out, limits, observer);
        assert!(observed.trap.is_none() && observed.recording.is_none());
        assert_eq!(String::from_utf8(out).unwrap(), "2\n");
        let heard = heard.borrow();
        let expected = [
            "declared a",
            "enter bump",
            "local j at depth 1",
            "leave",
            "changed a[1]",
            "printed 2",
            "finish ok",
        ];
        let mut rest = heard.iter();
        for want in expected {
            assert!(
                rest.any(|got| got == want),
                "no {want:?} in order in {heard:?}"
            );
        }
    }

    /// The strict-weak-order checks of a sort call the comparator too; the
    /// observer hears only the calls the sort itself makes.
    #[test]
    fn a_comparator_check_is_heard_by_no_observer() {
        let program = program(
            "fn lt(a: Int, b: Int) -> Bool:\n    return a < b\nvar xs = [2, 1]\nxs.sort(cmp: (a, b) => lt(a, b))\nprint(xs[0])\n",
        );
        let heard = Rc::new(RefCell::new(Vec::new()));
        let limits = Limits {
            stack_bytes: 1 << 22,
        };
        let (io, mut out) = (Io::fixed(b""), Vec::new());
        let observer = Some(Box::new(Heard(heard.clone())));
        let observed = run_as(&program, io, &mut out, limits, observer);
        assert!(observed.trap.is_none(), "{:?}", observed.trap);
        assert_eq!(String::from_utf8(out).unwrap(), "1\n");
        let heard = heard.borrow();
        // Merging [2] and [1] asks lt(1, 2) once; the checks ask six more.
        let calls = heard.iter().filter(|said| *said == "enter lt").count();
        assert_eq!(calls, 1, "{heard:?}");
    }

    #[test]
    fn an_unobserved_run_does_the_same_work() {
        let program = program(SOURCE);
        let limits = Limits {
            stack_bytes: 1 << 22,
        };
        let (io, mut out) = (Io::fixed(b""), Vec::new());
        let observed = run_as::<Unobserved>(&program, io, &mut out, limits, None);
        assert!(observed.trap.is_none() && observed.recording.is_none());
        assert_eq!(String::from_utf8(out).unwrap(), "2\n");
    }
}
