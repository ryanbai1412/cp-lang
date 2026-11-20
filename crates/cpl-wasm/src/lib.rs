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
use cpl_diag::{codes, Diagnostic, LineCol, LineIndex, Severity, Span};
use cpl_lsp::json::{self, obj, Value};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Version of the toolchain.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit status when the source has errors, as `cpl` exits.
pub const EXIT_ERRORS: u8 = 1;
/// Exit status when the program ran and then failed: a failed `assert`, an
/// index out of range, input outside its declared range.
pub const EXIT_FAILED: u8 = 3;

/// The emitted C++ has to fit a judge's source limit.
pub const MAX_CPP_BYTES: usize = 64 * 1024;

/// How deep a program's calls may nest in the browser, unless the caller
/// of [`Playground::run_to_depth`] says otherwise.
///
/// A WebAssembly module has one thread, so the interpreter runs on the
/// caller's stack, and that stack is in two parts. The module's own part
/// holds the interpreter's locals: [`STACK_BYTES`] of the 4 MB the module
/// is linked with (`build.rs`), about 1.2 kB for each call the program
/// makes. The engine's part holds the return addresses. It cannot be sized
/// or measured from here, browsers give it about 1 MB, and how many calls
/// that is depends on the program and on the engine: a few hundred to a
/// little over a thousand.
///
/// So this limit is where a run starts, not a promise. When the engine's
/// part fills first, the call ends as a trap, and `js/cpl.js` runs the
/// program again with half the limit until it ends in the interpreter's
/// own error, which has a source span.
pub const MAX_DEPTH: u32 = 1_000;

/// Bytes of the module's stack a program's calls may use.
#[cfg(target_family = "wasm")]
const STACK_BYTES: usize = 3 << 20;

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

    /// `{"version": "0.0.0", "maxDepth": 1000}`.
    pub fn version() -> Value {
        obj([("version", VERSION.into()), ("maxDepth", MAX_DEPTH.into())])
    }

    /// `cpl check`: every diagnostic in `source`.
    pub fn check(&mut self, name: &str, source: &str) -> Value {
        self.db.set_file(name, source);
        let diagnostics = self.db.diagnostics(name);
        let code = if diagnostics.is_empty() {
            0
        } else {
            EXIT_ERRORS
        };
        let stderr = rendered(&diagnostics, name, source);
        outcome(code, &diagnostics, source, stderr, [])
    }

    /// `cpl run`: checks `source`, then runs it in the debug interpreter
    /// with `input` as its standard input.
    pub fn run(&mut self, name: &str, source: &str, input: &[u8]) -> Run {
        self.run_to_depth(name, source, input, MAX_DEPTH)
    }

    /// [`run`](Self::run), stopping the program when its calls nest `depth`
    /// deep. The limit is the WebAssembly module's: a native run has a
    /// stack of its own and the interpreter's own limit.
    pub fn run_to_depth(&mut self, name: &str, source: &str, input: &[u8], depth: u32) -> Run {
        let mut stdout = Vec::new();
        let result = match self.compile(name, source) {
            Err(errors) => errors,
            Ok(mut compiled) => {
                let mut code = 0;
                if let Err(trap) = interpret(&compiled.program, input, &mut stdout, depth) {
                    let failure = Diagnostic::error(codes::RUNTIME, trap.span, trap.message);
                    compiled.stderr +=
                        &cpl_diag::render(&failure, name, source, &LineIndex::new(source));
                    compiled.diagnostics.push(failure);
                    code = EXIT_FAILED;
                }
                outcome(code, &compiled.diagnostics, source, compiled.stderr, [])
            }
        };
        Run { stdout, result }
    }

    /// `cpl build`: checks `source`, then writes it as one C++17 file.
    /// `cpp` is `null` when there is none to show.
    pub fn build(&mut self, name: &str, source: &str) -> Value {
        let compiled = match self.compile(name, source) {
            Ok(compiled) => compiled,
            Err(mut errors) => {
                if let Value::Object(members) = &mut errors {
                    members.push(("cpp".into(), Value::Null));
                }
                return errors;
            }
        };
        let mut cpp = cpl_codegen::emit(&compiled.program, name, Some(source));
        let (mut code, mut stderr) = (0, compiled.stderr);
        if cpp.len() > MAX_CPP_BYTES {
            // The source in the header is for the reader; a file the judge
            // turns away helps nobody.
            let bytes = cpp.len();
            cpp = cpl_codegen::emit(&compiled.program, name, None);
            if cpp.len() > MAX_CPP_BYTES {
                stderr += &format!(
                    "cpl: the C++ for `{name}` is {} bytes, over the {MAX_CPP_BYTES}-byte limit judges accept\n",
                    cpp.len()
                );
                code = EXIT_ERRORS;
            } else {
                stderr += &format!(
                    "cpl: with its source in the header the C++ for `{name}` would be {bytes} bytes, over the \
                     {MAX_CPP_BYTES}-byte limit judges accept; the header is left out\n"
                );
            }
        }
        let cpp = if code == 0 { cpp.into() } else { Value::Null };
        outcome(code, &compiled.diagnostics, source, stderr, [("cpp", cpp)])
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
            Ok(message) => Value::Array(self.lsp.handle(&message)),
            Err(error) => Value::Array(vec![obj([
                ("jsonrpc", "2.0".into()),
                ("id", Value::Null),
                (
                    "error",
                    obj([
                        ("code", Value::Number(-32700.0)),
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

#[cfg(target_family = "wasm")]
fn interpret(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut Vec<u8>,
    depth: u32,
) -> Result<(), cpl_interp::Trap> {
    let limits = cpl_interp::Limits {
        depth: depth as usize,
        stack_bytes: STACK_BYTES,
    };
    cpl_interp::run_on_this_thread(program, input, output, limits)
}

#[cfg(not(target_family = "wasm"))]
fn interpret(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut Vec<u8>,
    _depth: u32,
) -> Result<(), cpl_interp::Trap> {
    cpl_interp::run(program, input, output)
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
    let mut members = vec![
        ("ok".to_string(), Value::Bool(code == 0)),
        ("exitCode".to_string(), Value::from(u32::from(code))),
        (
            "diagnostics".to_string(),
            Value::Array(diagnostics.iter().map(|d| text.diagnostic(d)).collect()),
        ),
        ("stderr".to_string(), stderr.into()),
    ];
    members.extend(
        more.into_iter()
            .map(|(key, value)| (key.to_string(), value)),
    );
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
        let mut members = vec![
            ("start".to_string(), self.unit(span.start).into()),
            ("end".to_string(), self.unit(span.end).into()),
            ("range".to_string(), range),
        ];
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
