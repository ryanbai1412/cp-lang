//! What a tool still reads in today's forms: the checked file, its types
//! and hovers, the lowered program's statements, a run's recording. These
//! are answers, not entry points: nothing here checks, builds or runs a
//! program. Each use is one the rest of the facade's queries (hover,
//! complete, explain, replay: contract 9) take over in a later migration
//! stage, and debt-ratchet's `facade_today_uses` counts them until then.

use std::sync::Arc;

use cpl_diag::Diagnostic;

use crate::session::{self, Layer, Session};
use crate::snapshot::Text;

pub use cpl_check::{
    arity_key, builtin_members, closest, domain_member_docs, human_bytes, import_edit, interval,
    is_list_form_hook, loaded_std, needs_order, prelude_names, prelude_type, prelude_uses,
    split_builtin_params, type_arity, Analysis, AnalyzedItem, BuiltinMember, CalleeKind, Decl,
    DeclKind, Facts, FsLoader, HoverEntry, Import, Loader, MemberKind, Module, Overflow, Param,
    PreludeKind, RangeEnds, Receiver, Representation, Stats, Symbol, Ty, TypedIndex,
    BUILTIN_MEMBERS, BUILTIN_TYPES, OPTIONAL_MEMBERS, POS_MEMBERS, RANGE_MEMBERS,
};

/// A run's recording and what is read from it.
pub mod trace {
    use std::io::Write;

    pub use crate::TraceConfig;

    /// What a second run records to name the locals where a trap stopped the first.
    pub fn trap_config(source: &str) -> TraceConfig {
        cpl_interp::trace::trap_config(source).into()
    }
    pub use crate::{
        Assigned, Call, Change, Divergence, Live, Place, Recording, Shown, Stage, Step, Var,
    };

    /// The trace document of a run, beside its brute's when there is one.
    pub fn document(
        title: &str,
        description: &str,
        file: &str,
        source: &str,
        fast: (&str, &Recording),
        slow: Option<(&str, &Recording)>,
    ) -> String {
        let slow = slow.map(|(name, r)| (name, &r.inner));
        cpl_interp::trace::document(
            title,
            description,
            file,
            source,
            (fast.0, &fast.1.inner),
            slow,
        )
    }

    /// The first step where `after` differs from `before`.
    pub fn first_change(before: &Recording, after: &Recording) -> Option<Change> {
        cpl_interp::trace::first_change(&before.inner, &after.inner).map(Into::into)
    }

    /// [`first_change`] within the steps `old` of `before` and `new` of `after`.
    pub fn first_change_in(
        before: &Recording,
        old: std::ops::Range<usize>,
        after: &Recording,
        new: std::ops::Range<usize>,
    ) -> Option<Change> {
        cpl_interp::trace::first_change_in(&before.inner, old, &after.inner, new).map(Into::into)
    }

    /// Each variable `step` names, with its value.
    pub fn named_values(rec: &Recording, step: &Step) -> std::collections::HashMap<String, String> {
        cpl_interp::trace::named_values(&rec.inner, &step.clone().into())
    }

    /// The locals where a trap stopped the run, as `cpl run` names them.
    pub fn trap_locals(recording: &Recording, source: &str, at: &str) -> Option<String> {
        cpl_interp::trace::trap_locals(&recording.inner, source, at)
    }
    pub use cpl_interp::trace::*;

    /// Where a solution and its brute first part ways on one input.
    pub fn explain_divergence(
        source: &str,
        fast: &Recording,
        slow: &Recording,
    ) -> Option<Divergence> {
        cpl_interp::trace::explain_divergence(source, &fast.inner, &slow.inner).map(Into::into)
    }

    /// Where a solution that never finishes goes round.
    pub fn stuck_divergence(source: &str, fast: &Recording) -> Option<Divergence> {
        cpl_interp::trace::stuck_divergence(source, &fast.inner).map(Into::into)
    }

    /// The solution's last answer, when it is the line that differs.
    pub fn answered_divergence(source: &str, fast: &Recording) -> Option<Divergence> {
        cpl_interp::trace::answered_divergence(source, &fast.inner).map(Into::into)
    }

    /// The stages a solution and its brute both name, side by side.
    pub fn stages(fast: &Recording, slow: &Recording) -> Vec<Stage> {
        cpl_interp::trace::stages(&fast.inner, &slow.inner)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    use crate::{Outcome, Program, RunInput, Stop};

    /// The last `steps` steps of a run of `program` on `input`.
    pub fn tail(program: &Program, input: &[u8], config: TraceConfig, steps: usize) -> Outcome {
        cpl_interp::trace::tail(&program.ir, input, config.into(), steps).into()
    }

    /// The run printing to `output`, with its trace from
    /// `config.from_step` on.
    pub fn windowed(
        program: &Program,
        input: &[u8],
        output: &mut (dyn Write + Send),
        config: TraceConfig,
    ) -> Outcome {
        cpl_interp::trace::windowed(&program.ir, input, output, config.into()).into()
    }

    /// The last `steps` steps of a run, traced again from a late
    /// checkpoint of an untraced run: the steps are numbered from the
    /// window's first.
    pub fn resumed_tail(
        program: &Program,
        input: &[u8],
        output: &mut (dyn Write + Send),
        config: TraceConfig,
        steps: usize,
    ) -> Outcome {
        cpl_interp::trace::resumed_tail(&program.ir, input, output, config.into(), steps).into()
    }

    /// [`resumed_tail`], both runs ended by `stop`.
    pub fn resumed_tail_until(
        program: &Program,
        input: &[u8],
        output: &mut (dyn Write + Send),
        config: TraceConfig,
        steps: usize,
        stop: Stop,
    ) -> Outcome {
        cpl_interp::trace::resumed_tail_until(
            &program.ir,
            input,
            output,
            config.into(),
            steps,
            stop.0,
        )
        .into()
    }

    /// The untraced run talking to its judge, and every byte the judge
    /// sent it.
    pub fn sent(program: &Program, input: RunInput, output: &mut (dyn Write + Send)) -> Vec<u8> {
        cpl_interp::trace::sent(&program.ir, input.0, output)
    }

    /// The trace of an interactive run: the judge is asked only once.
    pub fn interactive(
        program: &Program,
        input: RunInput,
        output: &mut (dyn Write + Send),
        config: TraceConfig,
    ) -> Outcome {
        cpl_interp::trace::interactive(&program.ir, input.0, output, config.into()).into()
    }

    /// A run recorded on a thread of its own, which traces any window of
    /// it on request.
    pub struct Replay(cpl_interp::trace::Replay);

    impl Replay {
        /// Records `program`'s run on `input` as `config` counts its steps.
        pub fn start(program: Program, input: Vec<u8>, config: TraceConfig) -> Replay {
            let program = program.ir.clone();
            Replay(cpl_interp::trace::Replay::start(
                program,
                input,
                config.into(),
            ))
        }

        /// The run's trace from `config.from_step` on.
        pub fn window(&self, config: TraceConfig) -> Outcome {
            self.0.window(config.into()).into()
        }
    }
}

/// One call of a recorded run, run again with its fn edited.
pub mod replay {
    use crate::{Change, Recording as Recorded, Trap};

    /// What a call did: its result and each top-level variable it changed.
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Effect {
        /// JSON.
        pub returned: Option<String>,
        /// Variable ids and their JSON values when the call returned; `stdout`
        /// is what it printed.
        pub writes: Vec<(String, String)>,
    }

    impl From<cpl_interp::replay::Effect> for Effect {
        fn from(e: cpl_interp::replay::Effect) -> Effect {
            Effect {
                returned: e.returned,
                writes: e.writes,
            }
        }
    }

    #[derive(Debug)]
    pub struct Replay {
        /// The call as recorded, and as the edited fn ran it.
        pub recorded: Call,
        pub replayed: Option<Call>,
        pub before: Effect,
        pub after: Effect,
        /// The first step of the replayed call that differs from the recording,
        /// as a step of [`Replay::recording`]; `variable` is `returned` when
        /// only the result differs.
        pub change: Option<Change>,
        /// A step before the call that already differs: the edit changes an
        /// earlier call of the fn too.
        pub earlier: Option<Change>,
        /// Why the whole file ran instead of the one call.
        pub fallback: Option<String>,
        pub recording: Recorded,
        pub trap: Option<Trap>,
        json: String,
    }

    impl Replay {
        /// The report as JSON, for the trace viewer: values are the trace's
        /// JSON, `null` when there is none.
        pub fn json(&self) -> String {
            self.json.clone()
        }
    }

    impl From<cpl_interp::replay::Replay> for Replay {
        fn from(r: cpl_interp::replay::Replay) -> Replay {
            let json = r.json();
            Replay {
                json,
                recorded: r.recorded.into(),
                replayed: r.replayed.map(Into::into),
                before: r.before.into(),
                after: r.after.into(),
                change: r.change.map(Into::into),
                earlier: r.earlier.map(Into::into),
                fallback: r.fallback,
                recording: r.recording.into(),
                trap: r.trap.map(Into::into),
            }
        }
    }

    /// The call running at `step`, `frame` calls out from the innermost.
    pub fn call_at(recorded: &Recording, step: usize, frame: usize) -> Result<Call, String> {
        cpl_interp::replay::call_at(&recorded.inner, step, frame).map(Into::into)
    }

    use crate::Call;
    use crate::{Limits, Program, Recording, Stop, TraceConfig};

    /// The call to run again, and the programs before and after the edit.
    pub struct Rerun<'a> {
        pub recorded: &'a Recording,
        pub original: &'a Program,
        pub edited: &'a Program,
        pub input: &'a [u8],
        pub call: &'a Call,
    }

    impl<'a> Rerun<'a> {
        fn today<'b>(&'b self, call: &'b cpl_interp::trace::Call) -> cpl_interp::replay::Rerun<'b> {
            cpl_interp::replay::Rerun {
                recorded: &self.recorded.inner,
                original: &self.original.ir,
                edited: &self.edited.ir,
                input: self.input,
                call,
            }
        }
    }

    /// Runs `rerun`'s call again with the fns of its edited program.
    pub fn replay(rerun: Rerun, config: TraceConfig) -> Replay {
        let call = rerun.call.clone().into();
        cpl_interp::replay::replay(rerun.today(&call), config.into()).into()
    }

    /// [`replay`] where no thread can be started, its calls held to
    /// `limits` and each run ended by `stop`.
    pub fn replay_on_this_thread(
        rerun: Rerun,
        config: TraceConfig,
        limits: Limits,
        stop: Stop,
    ) -> Replay {
        let call = rerun.call.clone().into();
        cpl_interp::replay::replay_on_this_thread(
            rerun.today(&call),
            config.into(),
            limits.into(),
            stop.0,
        )
        .into()
    }
}

/// A run's profile.
pub mod profile {
    pub use cpl_interp::profile::*;
}

/// What the tests of the C++ count in it.
#[cfg(feature = "cpp")]
pub mod cpp_counts {
    pub use cpl_codegen::cpp_counts::*;
}

/// What `cpl check` and the editor report of a checked file:
/// [`Layer::Program`].
pub fn reported(analysis: &Analysis) -> Vec<Diagnostic> {
    session::diagnostics_of(analysis, Layer::Program)
}

macro_rules! asks {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(pub fn $name(&self, path: &str $(, $arg: $ty)*) -> $ret {
            self.asked(path).db.$name(path $(, $arg)*)
        })*
    };
}

/// The checked file and the editor's questions of it, in today's forms.
impl Session {
    /// The checked form of `file`: `None` when the session has no such file.
    #[inline(always)]
    pub fn analyze(&self, file: impl AsRef<str>) -> Option<Arc<Analysis>> {
        self.checked(file.as_ref())
    }

    /// [`Session::analyze`] of the session moved on to `file` holding `text`.
    #[inline(always)]
    pub fn analyze_text(&mut self, file: impl AsRef<str>, text: impl AsRef<str>) -> Arc<Analysis> {
        self.set_file(&file, text);
        self.checked(file.as_ref()).unwrap_or_default()
    }

    #[inline(always)]
    pub fn analysis(&self, file: impl AsRef<str>) -> Option<Arc<Analysis>> {
        self.analyze(file)
    }

    #[inline(always)]
    pub fn facts(&self, file: impl AsRef<str>) -> Option<Arc<Facts>> {
        self.analyze(file).map(|analysis| analysis.facts().clone())
    }

    /// The text of `path`: a file of the snapshot, a module of its
    /// standard library (`std/deque.cp`), or what an import loaded.
    pub fn file_text(&self, path: &str) -> Option<Text> {
        let snapshot = self.snapshot();
        if let Some(held) = snapshot.text_at(path) {
            return Some(held.clone());
        }
        let module = path
            .strip_prefix("std/")
            .and_then(|m| m.strip_suffix(".cp"));
        if let Some(text) = module.and_then(|m| snapshot.std_text(m)) {
            return Some(Text::from(text));
        }
        self.synced(None).db.file_text(path)
    }

    pub fn stats(&self) -> Stats {
        self.synced(None).db.stats()
    }

    pub fn import_cycles(&self) -> Vec<Vec<String>> {
        self.synced(None).db.import_cycles().to_vec()
    }

    asks! {
        method_fits(receiver: &Ty, names: &[&str]) -> Vec<Option<bool>>;
        storage() -> Option<Storage>;
        representation(offset: u32) -> Option<Representation>;
        costs_in(start: u32, end: u32) -> Vec<(cpl_diag::Span, String, String)>;
        typed_index(offset: u32, on_index: bool) -> Option<TypedIndex>;
        symbols() -> Vec<Symbol>;
        fns() -> Vec<Symbol>;
    }
}

/// The bytes a file's globals hold, and each one's when it is known.
pub type Storage = (u128, Vec<(String, Option<u64>)>);
