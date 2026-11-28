# Contract 9: everything → tools

The facade is the only interface a tool may use: CLI, language server,
formatter, lints, stress, debugger, trace, explain, the site and the
playground. The forms behind it are private and may change.

## What crosses

```rust
impl Session {
    fn open(snapshot: Snapshot) -> Session
    fn with_file(&self, file: FileId, text: Text) -> Session      // shares everything else
}
// Every query takes a cancel token and returns its diagnostics with its answer.
// typing path: contracts 1-3, plus per-unit facts
fn syntax(file) -> SyntaxFile
fn diagnostics(file, upto: Layer) -> List<Diagnostic>
fn hover(pos) -> Hover               // type, target, doc, facts, cost
fn type_at(pos) -> Option<Type>      // the type of the expression at pos
fn types_in(file, span) -> List<(Span, Type)>   // each typed expression inside span
fn trait_at(path: Str) -> Option<TraitId>  // a trait by its std or library path, `std.laws.Sample`: resolved once, never compared as text
fn complete(pos) -> List<Completion>
fn definition(pos), references(pos), rename(pos, new) -> List<Edit>
fn format(file) -> List<Edit>
fn fixes(range) -> List<Fix>
// idle path: whole program
fn lints(scope: Unit | Program) -> List<Diagnostic>
fn explain(target) -> Explanation    // layout, checks kept and why, cost, size by instance
fn estimate() -> Estimates           // peak memory, operation count, overflow sites
fn costs(scope: Unit | Program) -> List<CostNote>     // the performance view: one note per loop, container, copy, allocation
struct CostNote { at: SourceId, what: LoopCount | Size | Width | Copy | Alloc | Complexity, amount: Formula, certainty: Certainty }
enum Certainty { Proven { from: List<FactId> }, Estimated { how: Str }, Unknown { why: Str } }
// build and run
fn program(stage, mode) -> ProgramHandle
fn emit_cpp(profile) -> CppFile
fn run(backend, input, request: events::Request) -> (Outcome, Stream)
fn run_many(backend, inputs: List<RunInput>, jobs: usize, request) -> Iterator<(TestId, Outcome)>   // stress, tests; results in input order
fn show(value, type_, budget) -> Shown                // contract 8; the only value printer
fn compare(a: &Recording, b: &Recording) -> List<Aligned>   // contract 8
fn replay(recording) -> Stream
fn replay_call(recording, call: CallId, edited: Session) -> CallDiff   // one recorded call, rerun with edited code
fn layout(type_: Type) -> Layout     // pure in a concrete type and the profile; usable on the typing path

// A type as tools read it: opaque, and these queries are its whole API. Std's types are
// ordinary declarations a tool reaches through `decl`.
impl Type {
    fn text() -> Str                                  // as a diagnostic writes it: the one type printer
    fn range() -> Option<(Option<i64>, Option<i64>)>  // an integer's ends; None where the checker knows none
    fn decl() -> Option<Definition>                   // the struct or enum it is an instance of
    fn args() -> List<Type>                           // that declaration's type arguments
    fn implements(trait_: TraitId) -> bool            // the checker's impl lookup: how a tool asks what a type can do
    fn members() -> List<(Str, Definition)>           // what a `.` after a value of it can name
}
struct Diagnostic { code, layer, severity, at: SourceId, origin: List<Origin>, message, notes, fixes: List<Fix> }
struct Fix { title, edits: List<Edit>, safe: bool }
```

## Producer guarantees

- Every query is a pure function of the snapshot.
- A tool reads a type only through `Type`'s queries: it never matches
  on what a type is made of, and never compares a std type's name.
  Both facades answer them until the switch.
- A `CostNote` never states an estimate as proven. `Proven` names
  the facts it rests on, so a click shows why.
- `run` with `Entry::Call` is how the editor runs part of a program
  and how tests call one fn. There is no second way to execute code.
- An assistant (AI or otherwise) is a tool like any other: it reads
  and edits through these queries and gets no private view.
- Every query can be cancelled, and a newer snapshot never waits for
  an older query.
- One diagnostic type, one printer for types, one for diagnostics.
  The CLI, the editor and the playground show the same text.
- Typing-path queries read one unit and the signatures it names. Their
  cost does not grow with the file.
- A crash inside one unit becomes a compiler-bug diagnostic at that
  unit; other units still answer.

## Consumer may assume

- Anything a tool needs is a query here. If it is not, the fix is a
  new query, not a way around.

## Must never

- No tool may depend on a compiler crate other than the facade.
- No tool may keep its own cache of compiler answers, parse source, or
  build messages from parts of a diagnostic.

## Checked by

- Build graph: tool crates depend on the facade crate only.
  (Compiler-held.)
- Count gate: for a fixed list of edits to fixed files, the number of
  units rechecked, facts computed and instances made.
- Golden tests: each query's answer on the conformance suite.

## Open

- The facade is the stable interface and is unversioned. Editors and
  the site ship with the compiler, so that holds until a third party
  builds on it.
- (Audit 4, finding 3 agrees.) `hover` shows layout and cost. Layout is a function of a concrete
  type, so a top-level `var dist: [V]?Int` gets it on the typing path
  with no instances. Inside a generic function the answer waits for
  instances. Cost of a call is still open.
- Layout of a type with parameters returns a symbolic answer
  ("8 B x D.count"). No pass may change a stored width, or hover and
  the C++ disagree; "width narrowing" leaves the pass list.
- Missing check: the same job through the CLI, the server and the
  browser gives the same answer.

## Round 5 (2026-11-25, not yet reviewed)

```rust
fn eval(at: Paused, expr: Str) -> Result<Shown, Diagnostics>
    // type-checks new text in the scope of a paused frame, makes any instance it needs, runs it on the engine
fn program(stage, mode, job: Job) -> Program
struct Job {
    roots: List<DeclId>,                 // what to keep: the program, one test, one function
    strip: Set<Tests | Alts | Debug | Local>,   // submission strips all four
    substitutions: Map<DeclId, DeclId>,  // stress: swap in the brute force at call sites
    checks: On | Off | PerProfile,
}
fn sample_values(unit) -> List<(SourceId, Shown)>   // inlay hints with live values
fn refactor(kind: Extract | Inline, range) -> Edit  // needs captures and modes
```

- **The facade is built first, over today's compiler** (migration
  stage 2), with these signatures. Tools move onto it once; the new
  pipeline then replaces what is behind each query.
- **The typing path is one unit plus summaries.** With summaries in
  contract 2 and no errors from the link step, borrow errors show
  from the statement being typed.
