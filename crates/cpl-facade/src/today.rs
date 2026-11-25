//! What a tool still reads in today's forms: the checked file, its types
//! and hovers, the lowered program's statements, a run's recording. These
//! are answers, not entry points: nothing here checks, builds or runs a
//! program. Each use is one the rest of the facade's queries (hover,
//! complete, explain, replay: contract 9) take over in a later migration
//! stage, and debt-ratchet's `facade_today_uses` counts them until then.

use std::sync::Arc;

use cpl_diag::Diagnostic;

use crate::session::{self, Job, Layer, Program, Roles, Session};
use crate::snapshot::{FileId, Text};

pub use cpl_check::{
    arity_key, builtin_member, builtin_members, bundled_std, carry_only_embedded_std, char_literal,
    closest, configured_libraries, core_trait_fn, dev_std_problem, domain_member, domain_members,
    file_path, foreign_iter_call, foreign_member, foreign_str_fn, home_dir, human_bytes,
    import_edit, interval, is_list_form_hook, is_member, labels_fit, list_end_help, loaded_std,
    method_key, method_module, needs_order, optional_member, overload_key, overload_part,
    prelude_names, prelude_type, prelude_uses, provide_std, required_labels, split_builtin_params,
    std_calls, std_family, std_from_disk, std_index, std_methods, std_module, std_names,
    std_needed, std_needed_with, std_outline, std_self_heads, take_std_needs, type_arity,
    word_type, written_name, Analysis, AnalyzedItem, BuiltinMember, BuiltinType, CallFact, Callee,
    CalleeKind, Decl, DeclKind, Def, Definition, DomainInfo, ExprFacts, Facts, FsLoader, GlobalRef,
    Hover, HoverEntry, Import, ItemCheck, ItemSyntax, Loader, MemberKind, Module, NameMap,
    Overflow, Param, PreludeKind, RangeEnds, Receiver, Representation, Resolved, Stats, Symbol, Ty,
    TypedIndex, BUILTIN_MEMBERS, BUILTIN_TYPES, DOMAIN_MEMBERS, ITERATOR_POSITION_HELP,
    OPTIONAL_MEMBERS, OPTIONAL_UNWRAP_HELP, POS_MEMBERS, PRELUDE_VALUES, RANGE_MEMBERS,
};

/// The lowered program, for a tool that reads its statements.
pub mod ir {
    pub use cpl_ir::{has_cpp_body, has_stress_target};
    pub use cpl_ir::{sites, walk, Block, Program, Stmt, GRADER_BOUNDARY, STRESS_DECORATORS};
}

/// A run's recording and what is read from it.
pub mod trace {
    use std::io::Write;

    pub use cpl_interp::trace::*;

    use crate::{Outcome, Program, RunInput, Stop};

    /// The last `steps` steps of a run of `program` on `input`.
    pub fn tail(program: &Program, input: &[u8], config: TraceConfig, steps: usize) -> Outcome {
        cpl_interp::trace::tail(&program.ir, input, config, steps).into()
    }

    /// The run printing to `output`, with its trace from
    /// `config.from_step` on.
    pub fn windowed(
        program: &Program,
        input: &[u8],
        output: &mut (dyn Write + Send),
        config: TraceConfig,
    ) -> Outcome {
        cpl_interp::trace::windowed(&program.ir, input, output, config).into()
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
        cpl_interp::trace::resumed_tail(&program.ir, input, output, config, steps).into()
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
        cpl_interp::trace::resumed_tail_until(&program.ir, input, output, config, steps, stop)
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
        cpl_interp::trace::interactive(&program.ir, input.0, output, config).into()
    }

    /// A run recorded on a thread of its own, which traces any window of
    /// it on request.
    pub struct Replay(cpl_interp::trace::Replay);

    impl Replay {
        /// Records `program`'s run on `input` as `config` counts its steps.
        pub fn start(program: Program, input: Vec<u8>, config: TraceConfig) -> Replay {
            let program = program.ir.clone();
            Replay(cpl_interp::trace::Replay::start(program, input, config))
        }

        /// The run's trace from `config.from_step` on.
        pub fn window(&self, config: TraceConfig) -> Outcome {
            self.0.window(config).into()
        }
    }
}

/// One call of a recorded run, run again with its fn edited.
pub mod replay {
    pub use cpl_interp::replay::{call_at, Effect, Replay};

    use crate::{Limits, Program, Recording, Stop, TraceConfig};
    use cpl_interp::trace::Call;

    /// The call to run again, and the programs before and after the edit.
    pub struct Rerun<'a> {
        pub recorded: &'a Recording,
        pub original: &'a Program,
        pub edited: &'a Program,
        pub input: &'a [u8],
        pub call: &'a Call,
    }

    impl<'a> Rerun<'a> {
        fn today(&self) -> cpl_interp::replay::Rerun<'a> {
            cpl_interp::replay::Rerun {
                recorded: self.recorded,
                original: &self.original.ir,
                edited: &self.edited.ir,
                input: self.input,
                call: self.call,
            }
        }
    }

    /// Runs `rerun`'s call again with the fns of its edited program.
    pub fn replay(rerun: Rerun, config: TraceConfig) -> Replay {
        cpl_interp::replay::replay(rerun.today(), config)
    }

    /// [`replay`] where no thread can be started, its calls held to
    /// `limits` and each run ended by `stop`.
    pub fn replay_on_this_thread(
        rerun: Rerun,
        config: TraceConfig,
        limits: Limits,
        stop: Stop,
    ) -> Replay {
        cpl_interp::replay::replay_on_this_thread(rerun.today(), config, limits, stop)
    }
}

/// A run's profile.
pub mod profile {
    pub use cpl_interp::profile::*;
}

/// How a program lowers to C++, as `cpl explain` reports it.
pub mod explain {
    pub use cpl_codegen::explain::*;
    pub use cpl_codegen::{ClosureLowering, ClosureMode, Lowered};

    use crate::Program;

    /// The report of `program`, built from `text` as `path`, after what
    /// `storage` says of its memory.
    pub fn report_with(path: &str, text: &str, program: &Program, storage: Option<&str>) -> String {
        cpl_codegen::explain::report_with(path, text, &program.ir, storage)
    }

    /// How each closure the file calls is called in the C++.
    pub fn closures(program: &Program) -> Vec<ClosureLowering> {
        cpl_codegen::explain_closures(&program.ir)
    }

    /// The struct fields the C++ leaves out, never read.
    pub fn dropped_fields(program: &Program) -> Vec<String> {
        cpl_codegen::explain_dropped_fields(&program.ir)
    }

    /// Every choice the C++ makes that the source does not spell.
    pub fn lowered(program: &Program) -> Lowered {
        cpl_codegen::explain_lowered(&program.ir)
    }
}

/// What the tests of the C++ count in it.
pub mod cpp_counts {
    pub use cpl_codegen::cpp_counts::*;
}

/// The crates of today's compiler behind the facade, in pipeline order.
pub const COMPONENTS: &[&str] = &[
    cpl_check::NAME,
    cpl_ir::NAME,
    cpl_codegen::NAME,
    cpl_interp::NAME,
];

impl Program {
    /// Its statements, in today's lowered form.
    pub fn ir(&self) -> &ir::Program {
        &self.ir
    }
}

/// [`Session::diagnostics`] of a file a tool already holds checked.
pub fn diagnostics(analysis: &Analysis, upto: Layer) -> Vec<Diagnostic> {
    session::diagnostics_of(analysis, upto)
}

/// What `cpl check` and the editor report of a checked file:
/// [`Layer::Program`].
pub fn reported(analysis: &Analysis) -> Vec<Diagnostic> {
    session::diagnostics_of(analysis, Layer::Program)
}

/// Every diagnostic of a checked file taken as a program of its own:
/// [`Layer::Built`], what a fix is judged by.
pub fn built(analysis: &Analysis) -> Vec<Diagnostic> {
    session::diagnostics_of(analysis, Layer::Built)
}

/// [`Session::roles`] of a file a tool already holds checked.
pub fn roles(analysis: &Analysis) -> Roles<&Analysis> {
    Roles(analysis)
}

/// [`Session::program`] of a file a tool already holds checked, which had
/// no error: `Err` is what the build itself rejects.
pub fn program(analysis: &Analysis, job: &Job) -> Result<Program, Vec<Diagnostic>> {
    session::build_of(analysis, job)
}

macro_rules! asks {
    ($($name:ident($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(pub fn $name(&self, path: &str $(, $arg: $ty)*) -> $ret {
            self.synced(Some(&FileId::new(path))).db.$name(path $(, $arg)*)
        })*
    };
}

/// The checked file and the editor's questions of it, in today's forms.
impl Session {
    /// The checked form of `file`: `None` when the session has no such file.
    pub fn analyze(&self, file: impl Into<FileId>) -> Option<Arc<Analysis>> {
        self.checked(&file.into())
    }

    /// [`Session::analyze`] of the session moved on to `file` holding `text`.
    pub fn analyze_text(
        &mut self,
        file: impl Into<FileId>,
        text: impl Into<Text>,
    ) -> Arc<Analysis> {
        let file = file.into();
        self.set_file(&file, text);
        self.checked(&file).unwrap_or_default()
    }

    pub fn analysis(&self, file: impl Into<FileId>) -> Option<Arc<Analysis>> {
        self.analyze(file)
    }

    pub fn facts(&self, file: impl Into<FileId>) -> Option<Arc<Facts>> {
        self.analyze(file).map(|analysis| analysis.facts().clone())
    }

    /// The text of `path`, in the snapshot or as an import loaded it.
    pub fn file_text(&self, path: &str) -> Option<Text> {
        let held = self.snapshot().text(&FileId::new(path)).cloned();
        held.or_else(|| self.synced(None).db.file_text(path))
    }

    pub fn stats(&self) -> Stats {
        self.synced(None).db.stats()
    }

    pub fn import_cycles(&self) -> Vec<Vec<String>> {
        self.synced(None).db.import_cycles().to_vec()
    }

    /// Forgets what was loaded from outside the snapshot, after the world
    /// changed under it (a std module arrived).
    pub fn reload_modules(&self) {
        self.synced(None).db.reload_modules();
    }

    asks! {
        method_fits(receiver: &Ty, names: &[&str]) -> Vec<Option<bool>>;
        storage() -> Option<Storage>;
        representation(offset: u32) -> Option<Representation>;
        hover(offset: u32) -> Option<Hover>;
        costs_in(start: u32, end: u32) -> Vec<(cpl_diag::Span, String, String)>;
        typed_index(offset: u32, on_index: bool) -> Option<TypedIndex>;
        definition(offset: u32) -> Option<cpl_diag::Span>;
        definitions(offset: u32) -> Vec<Definition>;
        definition_location(offset: u32) -> Option<Definition>;
        type_definition(offset: u32) -> Option<Definition>;
        implementations(offset: u32) -> Vec<Definition>;
        symbols() -> Vec<Symbol>;
        fns() -> Vec<Symbol>;
    }
}

/// The bytes a file's globals hold, and each one's when it is known.
pub type Storage = (u128, Vec<(String, Option<u64>)>);
