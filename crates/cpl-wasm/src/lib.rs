//! The compiler as a WebAssembly module.
//!
//! The docs site and the playground run CP Lang in the browser with no
//! server: this crate is the front end, the debug interpreter and the C++17
//! back end behind one small interface. [`Playground`] is that interface in
//! Rust, and it answers in JSON so that the same code serves a browser and a
//! native test. `abi` exports it from the `.wasm` file, and `js/cpl.js`
//! wraps those exports for JavaScript.
//!
//! Every answer comes from the crates `cpl` itself is built from, through
//! the same [`cpl_check::Database`], so a program means in the browser what
//! it means on the command line: same diagnostics, same output, same C++.
//! `tests/parity.rs` holds the two to that.
//!
//! Positions cross the boundary in UTF-16 code units, the unit of a
//! JavaScript string: `start` and `end` index the source string, and `range`
//! is a Language Server Protocol range (zero-based line and character).

pub mod abi;

use cpl_check::{Database, DeclKind};
use cpl_diag::{Diagnostic, LineCol, LineIndex, Severity, Span};
use cpl_json::{self as json, obj, Value};
use std::collections::BTreeMap;

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Version of the toolchain.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit status when the source has errors, as `cpl` exits.
pub const EXIT_ERRORS: u8 = 1;
/// Exit status when the program ran and then failed: a failed `assert`, an
/// index out of range, input outside its declared range.
pub const EXIT_FAILED: u8 = 3;

/// Bytes of stack a program's calls may take in the browser, unless the
/// caller of [`Playground::run_with_stack`] says otherwise. Only the bytes
/// are limited, never how deep calls nest.
///
/// A WebAssembly module has one thread, so the interpreter runs on the
/// caller's stack, and that stack is in two parts. The module's own part
/// holds the interpreter's locals: this much of the 4 MB the module is
/// linked with (`build.rs`), about 1.2 kB for each call the program
/// makes. The engine's part holds the return addresses. It cannot be sized
/// or measured from here, browsers give it about 1 MB, and how many calls
/// that is depends on the program and on the engine: a few hundred to a
/// little over a thousand.
///
/// So this limit is where a run starts, not a promise. When the engine's
/// part fills first, the call ends as a trap, and `js/cpl.js` runs the
/// program again with half the limit until it ends in the interpreter's
/// own error, which has a source span.
pub const MAX_STACK: u32 = 3 << 20;

/// One compiler session. It keeps the incremental database between calls,
/// so checking a file again after an edit redoes only the items the edit
/// touched.
#[derive(Default)]
pub struct Playground {
    db: Database,
    lsp: cpl_lsp::Server,
}

/// What a run printed, and the rest of its result as JSON.
pub struct Run {
    pub stdout: Vec<u8>,
    pub result: Value,
}

/// A file that passed the front end, lowered for the back ends.
struct Compiled {
    program: cpl_ir::Program,
    /// Warnings: they are reported and the program still runs.
    diagnostics: Vec<Diagnostic>,
    /// The warnings as `cpl` prints them.
    stderr: String,
}

impl Playground {
    pub fn new() -> Playground {
        Playground::default()
    }

    /// `{"version": "0.3.0", "maxStack": 3145728}`.
    pub fn version() -> Value {
        obj([("version", VERSION.into()), ("maxStack", MAX_STACK.into())])
    }

    #[cfg(any(not(target_family = "wasm"), feature = "lazy-std"))]
    /// The std modules a check of `source` loads that this compiler has
    /// not been given yet, by name (`["sparse"]`): give each with
    /// `provide_std`, then ask again until it is empty.
    pub fn std_needed(&mut self, name: &str, source: &str) -> Value {
        Value::Array(
            cpl_check::std_needed(name, source)
                .into_iter()
                .map(Value::from)
                .collect(),
        )
    }

    #[cfg(any(not(target_family = "wasm"), feature = "lazy-std"))]
    /// Gives the compiler `text`, the file `std/<module>.cp`, for every
    /// command after: `{"changed": false}` when it had it already.
    pub fn provide_std(&mut self, module: &str, text: &str) -> Value {
        match cpl_check::provide_std(module, text) {
            Ok(changed) => {
                if changed {
                    self.db.reload_modules();
                    self.lsp.std_changed();
                }
                obj([("changed", Value::Bool(changed))])
            }
            Err(error) => obj([("error", error.into())]),
        }
    }

    /// `cpl check`: every diagnostic in `source`.
    pub fn check(&mut self, name: &str, source: &str) -> Value {
        self.db.set_file(name, source);
        let analysis = self.db.analyze(name).expect("the file was just set");
        let diagnostics = cpl_ir::check(&analysis);
        let code = if diagnostics.iter().any(|d| d.severity == Severity::Error) {
            EXIT_ERRORS
        } else {
            0
        };
        let stderr = rendered(&diagnostics, name, source);
        outcome(code, &diagnostics, source, stderr, [])
    }

    /// `cpl fix`: `source` with the fixes `cpl fix` applies (`cpl_ir::fix`
    /// for errors, then lint fixes once it checks clean), as `fixed`, with
    /// `applied` and the errors left as its diagnostics.
    pub fn fix(&mut self, name: &str, source: &str) -> Value {
        let config = cpl_lint::Config::default();
        let (fixed, applied, left) = cpl_lsp::fix::document(&mut self.db, name, source, &config);
        let code = if left.is_empty() { 0 } else { EXIT_ERRORS };
        let stderr = rendered(&left, name, &fixed);
        let applied = Value::from(u32::try_from(applied).unwrap_or(u32::MAX));
        outcome(
            code,
            &left,
            &fixed,
            stderr,
            [("fixed", fixed.clone().into()), ("applied", applied)],
        )
    }

    /// `cpl lint`: the lint warnings for `source` at default levels, with
    /// their fixes, as `check` reports diagnostics. Empty when `source` has
    /// a syntax error. Kept apart from `check` so that matches `cpl check`.
    pub fn lint(&mut self, name: &str, source: &str) -> Value {
        self.db.set_file(name, source);
        let analysis = self.db.analyze(name).expect("the file was just set");
        let diagnostics = cpl_lint::lint_analysis(&analysis, &cpl_lint::Config::default());
        let stderr = rendered(&diagnostics, name, source);
        outcome(0, &diagnostics, source, stderr, [])
    }

    /// `cpl run`: checks `source`, then runs it in the debug interpreter
    /// with `input` as its standard input.
    pub fn run(&mut self, name: &str, source: &str, input: &[u8]) -> Run {
        self.run_with_stack(name, source, input, MAX_STACK)
    }

    /// [`run`](Self::run), stopping the program when its calls take more
    /// than `stack` bytes of the module's stack. The limit is the
    /// WebAssembly module's: a native run has a stack of its own.
    pub fn run_with_stack(&mut self, name: &str, source: &str, input: &[u8], stack: u32) -> Run {
        let mut stdout = Vec::new();
        let result = match self.compile(name, source) {
            Err(errors) => errors,
            Ok(mut compiled) => {
                let mut code = 0;
                if let Err(trap) = interpret(&compiled.program, input, &mut stdout, stack) {
                    let lines = LineIndex::new(source);
                    let mut failure = trap.to_diagnostic(name, &lines);
                    if let Some(locals) =
                        trap_locals(&compiled.program, source, input, stack, &trap)
                    {
                        failure = failure.with_note(format!("locals: {locals}"));
                    }
                    compiled.stderr += &cpl_diag::render(&failure, name, source, &lines);
                    compiled.diagnostics.push(failure);
                    code = EXIT_FAILED;
                }
                outcome(code, &compiled.diagnostics, source, compiled.stderr, [])
            }
        };
        Run { stdout, result }
    }

    /// `cpl run --trace --format json`: runs `source` like
    /// [`run_with_stack`](Self::run_with_stack) and records it. The result
    /// has two more members: `trace`, a document of
    /// `tools/trace/trace-schema.json` (`null` when the source has
    /// errors), and `debug`, what the program's `debug(...)`s showed.
    /// A file with a `@alt(program)` fn is traced through it too, as the
    /// document's `slow` run, lined up with the solution's.
    pub fn trace_with_stack(
        &mut self,
        name: &str,
        source: &str,
        input: &[u8],
        stack: u32,
    ) -> (Vec<u8>, String) {
        use cpl_interp::trace::{document, TraceConfig};
        let mut stdout = Vec::new();
        let (result, trace, debug) = match self.compile(name, source) {
            Err(errors) => (errors, "null".to_string(), String::new()),
            Ok(mut compiled) => {
                let config = || TraceConfig {
                    source: source.to_string(),
                    record_frames: true,
                    ..TraceConfig::default()
                };
                let observed = observe(&compiled.program, input, &mut stdout, stack, config());
                let mut code = 0;
                if let Some(trap) = observed.trap {
                    let lines = LineIndex::new(source);
                    let failure = trap.to_diagnostic(name, &lines);
                    compiled.stderr += &cpl_diag::render(&failure, name, source, &lines);
                    compiled.diagnostics.push(failure);
                    code = EXIT_FAILED;
                }
                let fast = observed.recording.unwrap_or_default();
                // The brute's side, when the file has one that builds.
                let slow = self.db.analyze(name).and_then(|analysis| {
                    let (brute_fn, _) = cpl_ir::brute(&analysis)?;
                    let program = cpl_ir::lower_brute(&analysis, &brute_fn).ok()?;
                    let mut sink = Vec::new();
                    let recording =
                        observe(&program, input, &mut sink, stack, config()).recording?;
                    Some((brute_fn, recording))
                });
                let note = match &slow {
                    Some(_) => "The same input through the solution and through its brute, lined up at each print and each debug(...) both ran.",
                    None => fast.note(),
                };
                let trace = document(
                    name,
                    note,
                    name,
                    source,
                    (
                        if slow.is_some() {
                            "solution"
                        } else {
                            "cpl run"
                        },
                        &fast,
                    ),
                    slow.as_ref()
                        .map(|(brute_fn, recording)| (brute_fn.as_str(), recording)),
                );
                let debug = String::from_utf8_lossy(&observed.debug).into_owned();
                (
                    outcome(code, &compiled.diagnostics, source, compiled.stderr, []),
                    trace,
                    debug,
                )
            }
        };
        // The trace is JSON text already: it goes in as it is.
        let mut json = result.to_string();
        json.pop();
        json += &format!(",\"debug\":{},\"trace\":{trace}}}", Value::from(debug));
        (stdout, json)
    }

    /// `cpl replay`: the call open at `step` (0-based) of the run of
    /// `original` on `input`, `frame` callers out of the innermost, run again
    /// with the fns of `edited`. The answer is
    /// [`cpl_interp::replay::Replay::json`], or `{"error": ...}`: the same
    /// engine and report as `cpl run --trace --live` serves.
    #[allow(clippy::too_many_arguments)]
    pub fn replay(
        &mut self,
        name: &str,
        original: &str,
        edited: &str,
        input: &[u8],
        step: u32,
        frame: u32,
        stack: u32,
    ) -> String {
        use cpl_interp::trace::TraceConfig;
        let error = |message: &str| {
            obj([("error", Value::from(message.trim_end().to_string()))]).to_string()
        };
        let config = |source: &str| TraceConfig {
            source: source.to_string(),
            record_frames: true,
            ..TraceConfig::default()
        };
        let before = match self.compile(name, original) {
            Ok(compiled) => compiled.program,
            Err(_) => return error("the recorded source does not build"),
        };
        let after = match self.compile(name, edited) {
            Ok(compiled) => compiled.program,
            Err(errors) => {
                let stderr = errors
                    .get("stderr")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                return error(&format!("the edited file does not build:\n{stderr}"));
            }
        };
        let recorded = observe(&before, input, &mut Vec::new(), stack, config(original))
            .recording
            .unwrap_or_default();
        let call = match cpl_interp::replay::call_at(&recorded, step as usize, frame as usize) {
            Ok(call) => call,
            Err(message) => return error(&message),
        };
        let limits = cpl_interp::Limits {
            stack_bytes: stack as usize,
        };
        cpl_interp::replay::replay_on_this_thread(
            &recorded,
            &before,
            &after,
            input,
            &call,
            config(edited),
            limits,
            cpl_interp::Stop::after(RUN_LIMIT),
        )
        .json()
    }

    /// `cpl stress --explain --format json`, running in-process on `cpl-interp`
    /// with the same generator, shrinker and JSON report as the CLI.
    #[cfg(feature = "build")]
    pub fn stress(&mut self, name: &str, source: &str, cases: u32, seed: u64, size: u32) -> String {
        self.stress_with_stack(name, source, cases, seed, size, MAX_STACK)
    }

    /// [`stress`](Self::stress), stopping the interpreter when its calls take
    /// more than `stack` bytes on WebAssembly.
    #[cfg(feature = "build")]
    pub fn stress_with_stack(
        &mut self,
        name: &str,
        source: &str,
        cases: u32,
        seed: u64,
        size: u32,
        stack: u32,
    ) -> String {
        let options = cpl_stress::run::Options {
            cases: cases.max(1) as usize,
            size: i128::from(size.max(1)),
            seed,
            jobs: 1,
            ..cpl_stress::run::Options::default()
        };
        #[cfg(target_family = "wasm")]
        let stack_limit = Some(stack);
        #[cfg(not(target_family = "wasm"))]
        let stack_limit = {
            let _ = stack;
            None
        };
        match cpl_stress::stress_source(name, source, &options, stack_limit, true) {
            Ok(result) => result.to_json(),
            Err(error) => format!(
                "{{\"status\":\"error\",\"error\":{}}}\n",
                cpl_json::escape_string(&error.0)
            ),
        }
    }

    /// A random input for `source`, made from its `read()` types as `cpl
    /// stress` makes one: sizes at most `size`, drawn from `seed`.
    /// `{"input": text}`, or `{"error": why}` when its types make none.
    #[cfg(feature = "build")]
    pub fn sample(&mut self, source: &str, seed: u64, size: u32) -> String {
        let made = cpl_stress::InputSpec::from_source(source)
            .map_err(|error| error.message)
            .and_then(|spec| {
                let size = cpl_stress::Size::Small(i128::from(size.max(1)));
                cpl_stress::generate(&spec, &mut cpl_stress::Rng::new(seed), size)
                    .ok_or_else(|| "no input fits its `read()` types".to_string())
            });
        match made {
            Ok(case) => format!(
                "{{\"input\":{}}}\n",
                cpl_json::escape_string(&case.render())
            ),
            Err(error) => format!("{{\"error\":{}}}\n", cpl_json::escape_string(&error)),
        }
    }

    /// `cpl build`: checks `source`, then writes it as one C++17 file.
    /// `cpp` is `null` when there is none to show.
    #[cfg(feature = "build")]
    pub fn build(&mut self, name: &str, source: &str) -> Value {
        let compiled = match self.compile(name, source) {
            Ok(compiled) => compiled,
            Err(mut errors) => {
                if let Value::Object(members) = &mut errors {
                    members.insert("cpp".into(), Value::Null);
                }
                return errors;
            }
        };
        let (mut code, mut stderr) = (0, compiled.stderr);
        // The browser has no `cp.toml`: the file's `// judge:` line or `generic`.
        let profile = match cpl_judge::named_in(source) {
            Some(judge) => cpl_judge::builtin(judge),
            None => cpl_judge::builtin("generic"),
        };
        let Some(profile) = profile else {
            stderr += &format!(
                "cpl: the `// judge:` line names judge `{}`, which is not one of: {}\n",
                cpl_judge::named_in(source).unwrap_or_default(),
                cpl_judge::Config::default().names().join(", ")
            );
            return outcome(
                2,
                &compiled.diagnostics,
                source,
                stderr,
                [("cpp", Value::Null)],
            );
        };
        let limit = profile.source_limit;
        let target = cpl_codegen::Target {
            stamp: Some(profile.stamp()),
            thread_stack: profile.thread_stack,
            windows: profile.os == cpl_judge::Os::Windows,
            // `cpl build`'s default: a judge build checks what hand-written C++ does.
            checks: cpl_codegen::Checks::None,
        };
        let aim = profile.aim();
        let cpp = match cpl_codegen::emit_aiming(
            &compiled.program,
            name,
            source,
            (aim, limit),
            &target,
        ) {
            Ok(fitted) => {
                stderr += &fitted
                    .note(name, &format!("{aim}-byte target"), &profile.name)
                    .unwrap_or_default();
                fitted.cpp
            }
            Err(shortest) => {
                stderr += &format!(
                    "cpl: the C++ for `{name}` is {} bytes, over the {limit}-byte limit of judge `{}`\n",
                    shortest.cpp.len(),
                    profile.name
                );
                code = EXIT_ERRORS;
                String::new()
            }
        };
        if let Some(problem) = profile.unsupported(&cpp).filter(|_| code == 0) {
            stderr += &format!("cpl: `{name}` can't be submitted as it is: {problem}\n");
            code = EXIT_ERRORS;
        }
        let cpp = if code == 0 { cpp.into() } else { Value::Null };
        outcome(code, &compiled.diagnostics, source, stderr, [("cpp", cpp)])
    }

    /// `cpl explain` on `source`: how its closures, views and copies lower
    /// to C++, as the CLI prints it. `text` is null when it doesn't compile.
    #[cfg(feature = "build")]
    pub fn explain(&mut self, name: &str, source: &str) -> Value {
        match self.compile(name, source) {
            Ok(compiled) => {
                let storage = cpl_lsp::explain::storage_section(&mut self.db, name);
                let text = cpl_codegen::explain::report_with(
                    name,
                    source,
                    &compiled.program,
                    Some(&storage),
                );
                outcome(
                    0,
                    &compiled.diagnostics,
                    source,
                    compiled.stderr,
                    [("text", text.into())],
                )
            }
            Err(mut errors) => {
                if let Value::Object(members) = &mut errors {
                    members.insert("text".into(), Value::Null);
                }
                errors
            }
        }
    }

    /// What the editor shows when the cursor is at `offset`: the type of
    /// the name there and what it costs. `null` where there is nothing to
    /// say.
    pub fn hover(&mut self, name: &str, source: &str, offset: u32) -> Value {
        self.db.set_file(name, source);
        let text = Text::new(source);
        match self.db.hover(name, text.byte(offset)) {
            Some(hover) => text.located(hover.span, [("text", hover.text.into())]),
            None => Value::Null,
        }
    }

    /// Where the name at `offset` is declared, or `null`.
    pub fn definition(&mut self, name: &str, source: &str, offset: u32) -> Value {
        self.db.set_file(name, source);
        let text = Text::new(source);
        match self.db.definition(name, text.byte(offset)) {
            Some(span) => text.located(span, []),
            None => Value::Null,
        }
    }

    /// The top-level declarations of `source`, in source order.
    pub fn symbols(&mut self, name: &str, source: &str) -> Value {
        self.db.set_file(name, source);
        let text = Text::new(source);
        let symbols = self.db.symbols(name).into_iter().map(|symbol| {
            text.located(
                symbol.span,
                [
                    ("name", symbol.name.into()),
                    ("kind", kind_name(symbol.kind).into()),
                    ("selection", text.located(symbol.name_span, [])),
                ],
            )
        });
        Value::Array(symbols.collect())
    }

    /// Hands one Language Server Protocol message to the language server
    /// and returns the messages it sends back, in order. This is the server
    /// behind `cpl lsp` without the stdio framing, for an editor component
    /// that already speaks the protocol.
    pub fn lsp(&mut self, message: &str) -> Value {
        match json::parse(message) {
            Ok(message) => {
                let replies = self.lsp.handle(&message);
                // Its diagnostics say what it needs; a later check's are its own.
                cpl_check::take_std_needs();
                Value::Array(replies)
            }
            Err(error) => Value::Array(vec![obj([
                ("jsonrpc", "2.0".into()),
                ("id", Value::Null),
                (
                    "error",
                    obj([
                        ("code", (-32700i64).into()),
                        (
                            "message",
                            format!("the message is not JSON: {error}").into(),
                        ),
                    ]),
                ),
            ])]),
        }
    }

    /// The gate `run` and `build` both go through, as in `cpl`: check, then
    /// lower. On failure, the finished result.
    fn compile(&mut self, name: &str, source: &str) -> Result<Compiled, Value> {
        self.db.set_file(name, source);
        let analysis = self.db.analyze(name).expect("the file was just set");
        let mut diagnostics = analysis.diagnostics.clone();
        let mut stderr = rendered(&diagnostics, name, source);
        if analysis.has_errors() {
            return Err(outcome(EXIT_ERRORS, &diagnostics, source, stderr, []));
        }
        match cpl_ir::lower(&analysis) {
            Ok(program) => Ok(Compiled {
                program,
                diagnostics,
                stderr,
            }),
            Err(errors) => {
                stderr += &rendered(&errors, name, source);
                diagnostics.extend(errors);
                Err(outcome(EXIT_ERRORS, &diagnostics, source, stderr, []))
            }
        }
    }
}

/// The diagnostics as `cpl` prints them on standard error.
fn rendered(diagnostics: &[Diagnostic], name: &str, source: &str) -> String {
    if diagnostics.is_empty() {
        String::new()
    } else {
        cpl_diag::render_all(diagnostics, name, source)
    }
}

/// How long one run in the module may take: it ends as a time limit trap,
/// well before `js/cpl.js` would end the worker.
const RUN_LIMIT: std::time::Duration = std::time::Duration::from_secs(5);

fn bounded(input: &[u8]) -> cpl_interp::Io<'_> {
    cpl_interp::Io::fixed(input).until(cpl_interp::Stop::after(RUN_LIMIT))
}

#[cfg(target_family = "wasm")]
fn interpret(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut Vec<u8>,
    stack: u32,
) -> Result<(), cpl_interp::Trap> {
    let limits = cpl_interp::Limits {
        stack_bytes: stack as usize,
    };
    cpl_interp::run_with_on_this_thread(program, bounded(input), output, limits)
}

#[cfg(not(target_family = "wasm"))]
fn interpret(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut Vec<u8>,
    _stack: u32,
) -> Result<(), cpl_interp::Trap> {
    cpl_interp::run_with(program, bounded(input), output)
}

#[cfg(target_family = "wasm")]
fn observe(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut Vec<u8>,
    stack: u32,
    config: cpl_interp::trace::TraceConfig,
) -> cpl_interp::Observed {
    let limits = cpl_interp::Limits {
        stack_bytes: stack as usize,
    };
    cpl_interp::observe_on_this_thread(program, bounded(input), output, limits, Some(config))
}

#[cfg(not(target_family = "wasm"))]
fn observe(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut Vec<u8>,
    _stack: u32,
    config: cpl_interp::trace::TraceConfig,
) -> cpl_interp::Observed {
    cpl_interp::run_observed(program, bounded(input), output, Some(config))
}

/// What `cpl run` notes on a trap: the innermost call's locals where it
/// stopped, from a second run, recorded. The browser has no clock to skip
/// a slow run by, so it always runs again.
fn trap_locals(
    program: &cpl_ir::Program,
    source: &str,
    input: &[u8],
    stack: u32,
    trap: &cpl_interp::Trap,
) -> Option<String> {
    let config = cpl_interp::trace::trap_config(source);
    let observed = observe(program, input, &mut Vec::new(), stack, config);
    (observed.trap?.message == trap.message).then_some(())?;
    let at = source.get(trap.span.start as usize..trap.span.end as usize);
    cpl_interp::trace::trap_locals(&observed.recording?, source, at.unwrap_or(""))
}

/// The result every command shares: its exit status as `cpl` would exit,
/// the diagnostics as data for an editor, and `stderr`, the text `cpl`
/// prints on standard error, for a console.
fn outcome<const N: usize>(
    code: u8,
    diagnostics: &[Diagnostic],
    source: &str,
    stderr: String,
    more: [(&str, Value); N],
) -> Value {
    let text = Text::new(source);
    let mut members = BTreeMap::from([
        ("ok".to_string(), Value::Bool(code == 0)),
        ("exitCode".to_string(), Value::from(u32::from(code))),
        (
            "diagnostics".to_string(),
            Value::Array(diagnostics.iter().map(|d| text.diagnostic(d)).collect()),
        ),
        ("stderr".to_string(), stderr.into()),
    ]);
    members.extend(
        more.into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    );
    let needs = cpl_check::take_std_needs();
    if cfg!(any(not(target_family = "wasm"), feature = "lazy-std")) && !needs.is_empty() {
        let needs = needs.into_iter().map(Value::from).collect();
        members.insert("needs".to_string(), Value::Array(needs));
    }
    Value::Object(members)
}

fn kind_name(kind: DeclKind) -> &'static str {
    match kind {
        DeclKind::Value => "value",
        DeclKind::Fn => "fn",
        DeclKind::Struct => "struct",
        DeclKind::Enum => "enum",
        DeclKind::Variant => "variant",
        DeclKind::Domain => "domain",
        DeclKind::Alias => "alias",
        DeclKind::Trait => "trait",
        DeclKind::Impl => "impl",
    }
}

/// A source text, for turning the compiler's byte spans into the UTF-16
/// positions JavaScript uses and back.
struct Text<'a> {
    text: &'a str,
    lines: LineIndex,
    /// Source is nearly always ASCII, where a byte is a code unit.
    ascii: bool,
}

impl<'a> Text<'a> {
    fn new(text: &'a str) -> Text<'a> {
        Text {
            text,
            lines: LineIndex::new(text),
            ascii: text.is_ascii(),
        }
    }

    /// The UTF-16 offset of a byte offset.
    fn unit(&self, byte: u32) -> u32 {
        let byte = (byte as usize).min(self.text.len());
        if self.ascii {
            return byte as u32;
        }
        let head = &self.text.as_bytes()[..byte];
        String::from_utf8_lossy(head).encode_utf16().count() as u32
    }

    /// The byte offset of a UTF-16 offset. Past the end is the end.
    fn byte(&self, unit: u32) -> u32 {
        if self.ascii {
            return unit.min(self.text.len() as u32);
        }
        let mut units = 0;
        for (at, ch) in self.text.char_indices() {
            if units >= unit {
                return at as u32;
            }
            units += ch.len_utf16() as u32;
        }
        self.text.len() as u32
    }

    fn position(&self, byte: u32) -> Value {
        let LineCol { line, col } = self.lines.utf16(byte, self.text);
        obj([("line", line.into()), ("character", col.into())])
    }

    /// `{"start", "end", "range", ...more}` for a span.
    fn located<const N: usize>(&self, span: Span, more: [(&str, Value); N]) -> Value {
        let range = obj([
            ("start", self.position(span.start)),
            ("end", self.position(span.end)),
        ]);
        let mut members = BTreeMap::from([
            ("start".to_string(), self.unit(span.start).into()),
            ("end".to_string(), self.unit(span.end).into()),
            ("range".to_string(), range),
        ]);
        members.extend(
            more.into_iter()
                .map(|(key, value)| (key.to_string(), value)),
        );
        Value::Object(members)
    }

    fn diagnostic(&self, diagnostic: &Diagnostic) -> Value {
        let severity = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
        let labels = diagnostic
            .labels
            .iter()
            .map(|label| self.located(label.span, [("message", label.message.as_str().into())]));
        let notes = diagnostic.notes.iter().map(|note| note.as_str().into());
        let fixes = diagnostic.fixes.iter().map(|fix| {
            let edits = fix
                .edits
                .iter()
                .map(|(span, text)| self.located(*span, [("text", text.as_str().into())]));
            obj([
                ("title", fix.title.as_str().into()),
                ("edits", Value::Array(edits.collect())),
            ])
        });
        self.located(
            diagnostic.span,
            [
                ("code", diagnostic.code.0.into()),
                ("severity", severity.into()),
                ("message", diagnostic.message.as_str().into()),
                ("labels", Value::Array(labels.collect())),
                ("notes", Value::Array(notes.collect())),
                (
                    "help",
                    diagnostic.help.as_deref().map_or(Value::Null, Value::from),
                ),
                ("fixes", Value::Array(fixes.collect())),
            ],
        )
    }
}
