# The new pipeline's facade

**Status:** 2026-11-27, written by lane E; not yet reviewed. It says
what contract 9 looks like over `crates/cpl-next` while that pipeline
grows, which views each query reads, and the order tools move onto it.

## Why a second front door, and why not for long

Contract 9 says a tool reaches the compiler through one facade over an
immutable snapshot. `cpl-facade` is that facade over today's compiler.
The new pipeline cannot sit behind it yet: migration rule 5 keeps the
new back half reachable from the test harness only until the switching
landing, and `cpl-facade` ships.

So `cpl-next` gets the same front door, `cpl_next::facade`, with
contract 9's names and shapes. Every tool of the new pipeline (its
CLI, the judge output `tests/programs.rs` reads, trace, `--all`, the
tolerance and std-module tests) uses it and nothing else. The
switching landing then puts `cpl_next::facade` behind `cpl-facade`'s
`Session`, query by query, and tools do not change.

## What crosses

```rust
struct Snapshot {                       // contract 0; cheap to clone, never changed in place
    files: List<(FileId, Text)>,        // cpl-contract's types, the ones cpl-facade's API names
    world: World,                       // Disk: a file's cp.toml names its libraries; Closed: std only
    std: List<(Module, Text)>,          // std's texts, the core first: read from std/ when the snapshot is made
    events: bool,                       // a run reports semantic events (trace)
}
impl Snapshot {
    fn new(world: World) -> Snapshot
    fn with_file(&self, file: impl AsRef<str>, text: impl AsRef<str>) -> Snapshot
    fn without_file(&self, file: &FileId) -> Snapshot
    fn text(&self, file: &FileId) -> Option<&Text>
    fn files(&self) -> impl Iterator<Item = (&FileId, &Text)>
    fn world(&self) -> World
    fn with_std_dir(&self, dir: &Path) -> Snapshot      // `cpl-next --std DIR`
    fn traced(&self) -> Snapshot                         // runs report semantic events
    fn with_bodies(&self) -> Snapshot                    // `@fast` fns run their CP bodies
}

impl Session {                                           // every answer is per file, computed once
    fn open(snapshot: Snapshot) -> Session              // computes nothing yet
    fn at(self, snapshot: Snapshot) -> Session          // keeps the database; sets only what changed
    fn snapshot(&self) -> &Snapshot
    // typing path
    fn diagnostics(&self, file: &str, upto: Layer) -> List<&Diag>    // Layer::Typed | Layer::Program
    fn units(&self, file: &str) -> (&[Unit], &[Diag])                // the editor's view
    fn std_unchecked(&self) -> List<(Module, usize)>     // every std fn checked (suite 4)
    // build and run
    fn program(&self, file: &str) -> Result<&Program, &[Diag]>
    fn verdict(&self, file: &str) -> Verdict             // Accepted | Rejected | Refused: the exit rule
}
impl Program {
    fn run(&self, backend: Backend, input: &str) -> Outcome   // Raw | Canonical | Engine
    fn cpp(&self) -> CppFile
    fn proved(&self) -> &[Proved]
}

struct Diag {                           // one diagnostic, one printer, one JSON form
    stage: Stage, kind: Verdict(Code) | Refusal(Unbuilt), at: Option<At>, module, message,
    help, labels, fixes,
}
impl Diag { fn diagnostic(&self) -> cpl_diag::Diagnostic; fn json(&self) -> Json; Display }
```

- A `Session` answers every query from one snapshot. Its answers are
  computed once, on the first query that needs them, and never change.
  Two sessions of equal snapshots give equal answers.
- `verdict` is the one place the exit rule lives. The CLI, `--all` and
  the judge harness read it; none of them decides from diagnostics.
- `Diag`'s `Display` and `json` are the only printers of a diagnostic
  in the new pipeline, and `diagnostic()` is the `cpl_diag` form every
  other tool reads.

## Queries and the views behind them

Lane I's four views are the inputs each query is built from. Until a
view exists, the query reads the batch `front`/`back` result, and the
signature above does not change when the view lands.

| Query | Reads | Today |
|---|---|---|
| `diagnostics(Typed)` | Scope, Signatures, one body per unit | `front` |
| `units` | Scope, Signatures, one body per unit | `front` |
| `diagnostics(Program)` | Summaries (calls checked against them) | `back` (own on linked instances) |
| `program` | Generics, then link | `back` and canonicalize |
| `hover`, `definition` | Scope, Signatures | `items` |
| `complete` | Scope | `items`: the scope's names a prefix starts; after a `.`, `Type::members` |

### The editor queries lsp and lint still take from `today`

About 66 of `facade_today_uses`' lines are lsp and lint reading the old
checker (`cpl-check`) directly: its `Analysis`, `Facts` and `Ty`, and its
built-in member tables. Each has a facade query to take its place,
answered from cpl-next's views. None of them is wrapped over `cpl-check`
in the meantime: a facade name for an old checker item only renames it,
so the lines leave `today` when their query is answered from cpl-next.

| `today` items | Query | Reads |
|---|---|---|
| `Analysis`, `Facts`, `Ty`, `types_in` (lint rules, semantic tokens, rename) | `type_at(at)`, built: the innermost expression's type as a diagnostic writes it (`show_ty`, the checker's one printer); `types_in(span)` and an opaque facade `Type` for lint's structural questions come next | the typed tree (`TExpr::each`) |
| `reported` | `diagnostics(upto)` | as above |
| `builtin_members`, `BUILTIN_MEMBERS`, `RANGE_MEMBERS`, `OPTIONAL_MEMBERS`, `POS_MEMBERS`, `domain_member_docs`, `split_builtin_params`, `is_list_form_hook`, `Receiver` | `members(type)` | Signatures: std's members are declarations, so no table is kept |
| `arity_key`, `import_edit`, `prelude_uses`, `BUILTIN_TYPES` | `complete(at)` | Scope |
| `DeclKind`, `CalleeKind` | the kind on `Item` | Signatures |
| `interval`, `RangeEnds`, `Overflow` | `bounds(span)` | the typed tree's range types |
| `Representation`, `human_bytes` | `layout(at)`, not built | the backend's layout table (below) |

Lint asks the old `Ty` structural questions: a range's ends (`Range`), the
domain a position is in (`Pos`), an `Optional`'s or `Array`'s parts, and a
named type's arguments. The facade's `Type` answers them without naming a
std type, since `List`, `Array` and `Optional` are ordinary declarations:

```rust
fn type_at(&self, file: &str, at: u32) -> Option<Type<'_>>
fn types_in(&self, file: &str, span: Span) -> Vec<(Span, Type<'_>)> // outermost first
struct Type<'s>(..)                      // opaque, borrows the session's typed tree
impl Type<'_> {
    fn text(&self) -> String             // as a diagnostic writes it (show_ty)
    fn range(&self) -> Option<(Option<i64>, Option<i64>)> // an integer's ends
    fn domain(&self) -> Option<&str>     // a position's domain, by name
    fn decl(&self) -> Option<Item>       // the struct or enum it is an instance of
    fn args(&self) -> Vec<Type<'_>>      // that declaration's type arguments
    fn implements(&self, name: &str) -> bool // the trait the program's scope names; Signatures::has_impl
    fn members(&self) -> Vec<(String, Item)> // fields, own fns, and each found impl's fns
}
```

`implements` and `members` read the checker's own impl lookup through
the Signatures view (`impl_args`, `has_impl`), so a tool never matches
impls by hand.

`layout(at)` waits on the backend. The old `represent` is the old
checker's own model of storage, and nothing ties it to the C++ the judge
runs. cpl-next-cpp decides no layout in one place today: a type's
storage is whatever its emitted C++ struct is. The query is answered once
cpl-next-cpp has a layout table, one fn deciding each instance's storage
(width, where an optional keeps `None`, field offsets) that the emitter
and the facade both read. A second Rust model of `sizeof` beside the
emitter is not built.

Ruling: there is no `is_array` or `is_optional`. A lint rule that today
matches `Ty::Array` or `Ty::Optional` asks `implements` of the trait it
relies on (`Index`, `Seq`), or compares `decl` with the std `Item` it was
given by `definition`, so the rule reads std the way a user's code would.

## Trace and replay

`cpl-facade` used to hand tools the old interpreter's step log through
`today::trace` and `today::replay`: about 140 of `facade_today_uses`'
lines (cli 77, stress 51, wasm 13). The trace query below has taken their
place, and `today::trace` is gone. It is shaped by contract 8 so stage 5's engine can answer it from its event
stream without a tool changing.

```rust
struct Recording(..)                    // opaque: contract 8's Recording
impl Recording {
    fn len(&self) -> usize; fn is_empty(&self) -> bool
    fn truncated(&self) -> bool           // steps past the limit were not kept
    fn skipped(&self) -> usize            // the first kept step is this step of the run
    fn step(&self, i) -> Option<StepView> // line, label, stage, frames, the values it showed
    fn line(&self, i) -> Option<u32>; fn depth(&self, i) -> usize
    fn place(&self, i) -> Option<Place>; fn find(&self, p: Place, at: usize) -> usize
    fn calls(&self) -> &[Call]; fn vars(&self) -> &[Var]; fn lives(&self) -> &[Live]
    fn assigned(&self) -> &[Assigned]; fn shown(&self) -> &[Shown]
}
struct Place { line: u32, visit: usize }      // contract 8's At, by line until sites have ids
struct Call { name, ordinal, entry, exit, depth, args, returned }
struct Change { step, line, variable, before, after }
struct Divergence { name, nth, fast, brute, inside, blame, call }

fn first_change(a: &Recording, b: &Recording, within: Option<usize>) -> Option<Change>
fn divergence(source, fast: &Recording, slow: Option<&Recording>) -> Option<Divergence>
fn stages(fast: &Recording, slow: &Recording) -> List<Stage>
fn document(title, note, file, source, fast: (&str, &Recording), slow: Option<..>) -> String
fn trap_locals(recording: &Recording, source, at) -> Option<String>
fn call_at(recording: &Recording, step, frame) -> Result<Call, String>
fn replay(rerun: Rerun, config: TraceConfig) -> Replay           // contract 9's replay_call
```

- `Recording` is opaque so that stage 5 backs it with contract 8's
  stream. `Call`, `Change`, `Divergence`, `Live`, `Var` and `Place`
  are plain data that tools build and match on, so they are the
  facade's own structs, not the interpreter's.
- `first_change` is contract 8's `first_difference`, and `divergence`
  is its `align` read at the first `Exit` that differs. Each one is
  computed once, behind the facade. Tools print what it found and
  never walk two step logs themselves.
- `document` is the trace viewer's one printer, used by the CLI, the
  browser and `cpl run --trace`.
- A text helper that has no compiler answer in it (`mentions`) moves
  into the tool that uses it.

What stage 5's engine must supply for this query: a step per
statement with its site, its call depth and frames (`Enter`/`Exit`),
the values each one bound or wrote (`Bind`/`Write`), and the visit
count per site inside a call (`At.nth`).

### Sites are statement lineage, not lines

An event names its statement by lineage (contract 8: identity is which
statement replaced which, not a content hash), and never by a line
string or a line number:

```rust
struct StmtId { item: ItemId, stmt: u32 }   // ItemId: module, name, nth, file (contract 1)
struct At { call: CallPath, site: StmtId, nth: u32 }
fn source_of(&self, site: StmtId) -> Option<Source>  // Session: module, span, line, now
struct Source { module: String, span: Span, line: u32 }
```

- `ItemId` is already stable across edits of other items. `stmt` numbers
  a statement inside its item in source order when the item is first
  parsed, and an edit keeps the number of each statement it did not
  replace, so a breakpoint and a recording outlive an edit elsewhere.
- Every layer carries the `StmtId` it was given: lowering, inlining and
  optimization keep the original's id on what they make from it (an
  inlined call's statements keep the callee's ids under the caller's
  `CallPath`), and a statement the compiler makes with no source of its
  own takes the id of the statement it was made for. No event has a
  site no source statement answers.
- Only the facade turns a site into text: `Session::source_of` reads
  the snapshot, so a viewer, the debugger and replay ask the facade
  where an event is, and the answer follows the snapshot, not the run.
  `Recording::line`, `Place.line` and `Change.line` become reads of
  `source_of` at stage 5. Until then they are the interpreter's lines,
  behind the same accessors, so no tool changes when they switch.
- `Place` becomes `{ site: StmtId, visit }`, and `find` aligns by it,
  which is contract 8's `align` by `At`.

## Running

A run is contract 0's `RunInput` and contract 7's `Outcome`, the same
types on the reference interpreter and the engine (the C++ gets its
limits from the judge profile), so the VM and the C++ agree. Audit S10;
`next_run_api_unbuilt` counts each contract member not built yet.

```rust
pub struct RunInput { entry: Entry, io: Io, out: Option<Box<dyn Write + Send>>, args: Vec<String>, choices: Choices, limits: Limits, cold_caches: bool }
pub enum Entry { Program, Call { f: Item, args: Vec<Value> } }
pub enum Io { Fixed(Vec<u8>), Stream(Box<dyn Read + Send>), Peer { program: Snapshot, input: Vec<u8> }, Transcript(Vec<Turn>) }
pub struct Limits { time: Option<Duration>, memory: u64, stack: u64, output: u64, peer_turns: u64 }
pub enum Outcome {
    Finished { output: Vec<u8>, exit: i32, used: Usage },
    Failed   { check: Code, at: StmtId, origin: Vec<Frame>, message: String, output_so_far: Vec<u8> },
    Limit    { which: Limit, estimated: bool },
    Stalled  { waiting_on: Side },
    CompilerBug { instance: InstanceId, detail: String },
}
fn run(&self, input: RunInput, stop: &Stop, events: &mut dyn Sink) -> Outcome
fn entries(&self) -> Vec<(Role, Item)>        // Role: Test, Alt, Check, Interactor
```

- **Entry roots.** `@test`, `@alt`, `@check` and an interactor are
  roles decls records on a fn, so `entries()` reads them through
  `Signatures`, never by name. Running one is `Entry::Call` of its item;
  an interactor runs as the `Peer` of the solution.
- **I/O is pulled.** The engine's `ops::Input` asks its source for bytes
  when its buffer runs dry. `Fixed` is one piece. `Stream` reads a
  terminal or pipe as the program reads, which contract 0 lacks; it is
  added there in the same landing. `Peer` runs the other program on a
  thread of its own with a pipe each way. A read that must wait for
  more input sends the reader's pending output on first, to its peer or
  to `out`, the sink a `Stream` run's output goes to; a flush and the
  exit send it too, and `Outcome` keeps the whole output either way
  (R10.4). `ops::Input` decides when a read must wait, for every back
  end. Both sides reading is `Stalled`, and
  each turn counts against `peer_turns`. `Transcript` replays a recorded
  exchange.
- **Limits are the snapshot's, not constants.** Time is a deadline that
  the step counter checks, beside `Stop`'s cancel. Memory is `Limits::memory`, the judge's: each storage a run makes is checked against it by one rule (`ops::storage_size`, cells of `Layouts::cell_bytes`), which replaced `ops::MAX_STORAGE`; counting the live heap words waits for its ruling. Stack is the
  frame words; it replaces the engine's frame-count constant. Output is
  the bytes written. Each limit hit is `Outcome::Limit`, never a trap.
- **Frames at a failure.** `Failed::origin` is the calls the check left,
  innermost first, each a `Frame { call: Call, site: StmtId, locals }`,
  so the debugger and `explain` show locals at a trap. A trace keeps a
  window of the last events (a ring buffer the `Request` sizes) when the
  whole stream is too long; lane H's `Recording` holds it.

Landings, each building its piece whole: (1) the types in
`cpl-next-ir::run`, with `Fixed`/`Stream` and every limit on both the
interpreter and the engine; (2) `Peer`, `Transcript` and `Stalled`;
(3) `Entry::Call` and `entries()`; (4) `Session::run` on the facade, and
the cpl-next CLI and harness move onto it.

## Must never

- A tool of the new pipeline names `compile`, `units`, `Compiled`,
  a `cpl_next_*` crate or a layer function. Once the first
  tools have moved, those stop being `pub`.
- A query reads a file, the clock or the environment. `Std::read` and
  `Roots::from` read the disk; only the code that builds a snapshot
  (the CLI, a test) calls them.

## Order of work

1. The facade module and the `cpl-next` binary onto it: run, `--check`,
   `--cpp`, `--json`, `--trace`, `--all`. The exit rule moves into
   `verdict`.
2. The crate's own tests (`tolerance.rs`, `std_modules.rs`, the lib
   tests) onto it; `compile` and `units` go private.
3. `hover` and `definition` from Scope and Signatures (lane I's views):
   both read one `Item` per name in the program's scope, its declaring
   module, span, line and first line past decorators. A trait has no
   source position in `TraitDecl` yet, so it has no `Item`.
4. At the switching landing, `cpl-facade`'s queries answer from this
   one.

## Open

- A snapshot holds std's texts, read from `std/` (or `--std DIR`) when it
  is made, and `std()` builds `Std` from them as database sources. An
  editor's unsaved std edit still needs a `with_std_file`, the way
  `cpl-facade`'s `with_std` takes given texts.
