//! What a run tells whoever watches it: the interface the debugger needs
//! from the interpreter. The interpreter is built once per kind of
//! [`Observer`]: an untraced run with [`Unobserved`], whose hooks compile
//! away, a traced one with [`Tracer`].

use crate::trace::{Recording, Tracer, View};
use crate::Value;
use crate::{leading_indices, relifetime, root_global, root_local, trace, Interp};
use cpl_diag::Span;
use cpl_ir::{Place, PlaceKind};

/// Each hook is a step of the run, seen through a [`View`] of its state.
/// The defaults ignore it, so an observer implements what it watches.
pub(crate) trait Observer {
    /// False compiles every hook out of the run.
    const ON: bool = true;
    /// The run is a multitest; its cases come next.
    fn begin_cases(&mut self) {}
    /// The output between two cases.
    fn divider(&mut self) {}
    fn start_case(&mut self, _case: u64) {}
    /// A fn is entered: what [`Observer::leave_fn`] gets back.
    fn enter_fn(&mut self, _name: &str) -> (bool, bool) {
        (false, false)
    }
    fn leave_fn(&mut self, _entered: (bool, bool)) {}
    fn enter_call(&mut self, _name: &str, _args: impl FnOnce() -> String) {}
    /// Whether [`Observer::leave_call`] wants the returned value.
    fn keeps_call(&self) -> bool {
        false
    }
    fn leave_call(&mut self, _returned: Option<String>) {}
    /// The observer has seen enough: the run stops.
    fn stopped(&self) -> bool {
        false
    }
    /// Whether every assignment is a step, not only changes.
    fn assignments(&self) -> bool {
        false
    }
    /// Global `id`, or a part of it, was written where no step is kept:
    /// true when the observer has taken note and wants no view.
    fn quiet_write(&mut self, _place: &Place, _depth: usize) -> bool {
        false
    }
    /// Global `id` was given its first value.
    fn declared(&mut self, _id: usize, _view: &View, _at: Span) {}
    /// A cell of global `id`, at `path`, changed.
    fn changed(&mut self, _id: usize, _view: &View, _at: Span, _path: &[usize]) {}
    /// Global `id` was mutated in place, by a method call.
    fn mutated(&mut self, _id: usize, _view: &View, _at: Span) {}
    /// Local `name` of the running call changed.
    fn local_changed(&mut self, _view: &View, _at: Span, _name: &str) {}
    fn assigned_local(&mut self, _view: &View, _at: Span, _name: &str) {}
    fn assigned_global(&mut self, _view: &View, _at: Span, _id: usize) {}
    fn printed(&mut self, _view: &View, _at: Span, _text: &str) {}
    fn debugged(&mut self, _view: &View, _at: Span, _name: &str, _text: String, _json: String) {}
    /// [`Observer::changed`] for an `Int` cell, with the locals raw in
    /// [`View::raw`]; false asks for the full view instead.
    fn raw_write(&mut self, _id: usize, _view: &View, _at: Span, _path: &[usize]) -> bool {
        false
    }
    /// The run is over, or trapped at `failed`.
    fn finish(self: Box<Self>, _view: &View, _failed: Option<(&str, Span)>) -> Option<Recording> {
        None
    }
}

/// No one watches: the interpreter built for it has no hooks.
pub(crate) struct Unobserved;

impl Observer for Unobserved {
    const ON: bool = false;
}

impl Observer for Tracer {
    fn begin_cases(&mut self) {
        self.case = Some(0);
    }
    fn divider(&mut self) {
        Tracer::divider(self)
    }
    fn start_case(&mut self, case: u64) {
        Tracer::start_case(self, case)
    }
    fn enter_fn(&mut self, name: &str) -> (bool, bool) {
        Tracer::enter_fn(self, name)
    }
    fn leave_fn(&mut self, entered: (bool, bool)) {
        Tracer::leave_fn(self, entered)
    }
    fn enter_call(&mut self, name: &str, args: impl FnOnce() -> String) {
        Tracer::enter_call(self, name, args)
    }
    fn keeps_call(&self) -> bool {
        Tracer::keeps_call(self)
    }
    fn leave_call(&mut self, returned: Option<String>) {
        Tracer::leave_call(self, returned)
    }
    fn stopped(&self) -> bool {
        self.stopped
    }
    fn assignments(&self) -> bool {
        self.assignments
    }
    fn quiet_write(&mut self, place: &Place, depth: usize) -> bool {
        Tracer::quiet_write(self, place, depth)
    }
    fn declared(&mut self, id: usize, view: &View, at: Span) {
        Tracer::declared(self, id, view, at)
    }
    fn changed(&mut self, id: usize, view: &View, at: Span, path: &[usize]) {
        Tracer::changed(self, id, view, at, path)
    }
    fn mutated(&mut self, id: usize, view: &View, at: Span) {
        Tracer::mutated(self, id, view, at)
    }
    fn local_changed(&mut self, view: &View, at: Span, name: &str) {
        Tracer::local_changed(self, view, at, name)
    }
    fn assigned_local(&mut self, view: &View, at: Span, name: &str) {
        Tracer::assigned_local(self, view, at, name)
    }
    fn assigned_global(&mut self, view: &View, at: Span, id: usize) {
        Tracer::assigned_global(self, view, at, id)
    }
    fn printed(&mut self, view: &View, at: Span, text: &str) {
        Tracer::printed(self, view, at, text)
    }
    fn debugged(&mut self, view: &View, at: Span, name: &str, text: String, json: String) {
        Tracer::debugged(self, view, at, name, text, json)
    }
    fn raw_write(&mut self, id: usize, view: &View, at: Span, path: &[usize]) -> bool {
        Tracer::raw_write(self, id, view, at, path)
    }
    fn finish(self: Box<Self>, view: &View, failed: Option<(&str, Span)>) -> Option<Recording> {
        Some(Tracer::finish(*self, view, failed))
    }
}

impl<O: Observer> Interp<'_, '_, O> {
    /// Whether the run is watched; false at compile time when unobserved.
    pub(crate) fn observed(&self) -> bool {
        O::ON && self.tracer.is_some()
    }

    /// A trap once the tracer has what it wanted, as after the call to replay.
    pub(crate) fn unstopped(&self, span: Span) -> crate::Res<()> {
        match &self.tracer {
            Some(tracer) if tracer.stopped() => crate::trap(span, trace::STOPPED),
            _ => Ok(()),
        }
    }

    /// Runs `record` on the tracer, when the run is traced.
    pub(crate) fn traced(&mut self, record: impl FnOnce(&mut O, &trace::View)) {
        if !O::ON {
            return;
        }
        if let Some(mut tracer) = self.tracer.take() {
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
        let Some(global) = root_global(place) else {
            if let Some(slot) = root_local(place) {
                self.trace_local(slot, place.span);
            }
            return;
        };
        if let PlaceKind::Global(id) = &place.kind {
            let id = *id;
            return self.traced(|tracer, view| tracer.declared(id, view, place.span));
        }
        let mut path = std::mem::take(&mut self.trace_path);
        path.clear();
        path.extend(
            indices
                .iter()
                .take(leading_indices(place))
                .map(|index| index.int().max(0) as usize),
        );
        self.traced(|tracer, view| tracer.changed(global, view, place.span, &path));
        self.trace_path = path;
    }

    /// A write to global `place` that the observer takes note of without
    /// a view, as when the running fn is not traced.
    pub(crate) fn quiet_write(&mut self, place: &Place) -> bool {
        let depth = self.call_frames.len();
        match self.tracer.as_mut() {
            Some(tracer) if O::ON => tracer.quiet_write(place, depth),
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
    pub(crate) fn trace_assigned(&mut self, place: &Place) {
        if !self.assignments() {
            return;
        }
        let span = place.span;
        if let Some(id) = root_global(place) {
            self.traced(|tracer, view| tracer.assigned_global(view, span, id));
        } else if let Some(slot) = root_local(place) {
            self.trace_assigned_local(slot, span);
        }
    }

    /// A step for local `slot` of a running call, which just changed.
    pub(crate) fn trace_local(&mut self, slot: usize, span: Span) {
        // Top-level locals are loop counters and the like: no step of their own.
        if !self.observed() || self.call_frames.is_empty() {
            return;
        }
        let name = self.locals()[slot].name.as_str();
        self.traced(|tracer, view| tracer.local_changed(view, span, name));
    }

    pub(crate) fn trace_assigned_local(&mut self, slot: usize, span: Span) {
        if !self.assignments() {
            return;
        }
        let name = self.locals()[slot].name.as_str();
        self.traced(|tracer, view| tracer.assigned_local(view, span, name));
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
        fn declared(&mut self, id: usize, view: &View, _at: Span) {
            self.say(format!("declared {}", view.program.globals[id].name));
        }
        fn changed(&mut self, id: usize, view: &View, _at: Span, path: &[usize]) {
            self.say(format!("changed {}{path:?}", view.program.globals[id].name));
        }
        fn enter_call(&mut self, name: &str, _args: impl FnOnce() -> String) {
            self.say(format!("enter {name}"));
        }
        fn leave_call(&mut self, _returned: Option<String>) {
            self.say("leave".into());
        }
        fn local_changed(&mut self, view: &View, _at: Span, name: &str) {
            self.say(format!("local {name} at depth {}", view.depth));
        }
        fn printed(&mut self, _view: &View, _at: Span, text: &str) {
            self.say(format!("printed {}", text.trim_end()));
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
        let mut db = cpl_check::Database::new();
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
        let observed = run_as(&program, io, &mut out, limits, observer, false);
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

    #[test]
    fn an_unobserved_run_does_the_same_work() {
        let program = program(SOURCE);
        let limits = Limits {
            stack_bytes: 1 << 22,
        };
        let (io, mut out) = (Io::fixed(b""), Vec::new());
        let observed = run_as::<Unobserved>(&program, io, &mut out, limits, None, false);
        assert!(observed.trap.is_none() && observed.recording.is_none());
        assert_eq!(String::from_utf8(out).unwrap(), "2\n");
    }
}
