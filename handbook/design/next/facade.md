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
    program: Text,                      // the user's file
    roots: Roots,                       // the libraries cp.toml names, read by whoever builds it
    std: Arc<Std>,                      // shared by every snapshot of a session or a worker
    events: bool,                       // a run reports semantic events (trace)
}
impl Snapshot {
    fn new(std: Arc<Std>, roots: Roots, program: Text) -> Snapshot
    fn with_program(&self, text: Text) -> Snapshot        // shares std and roots
    fn traced(&self) -> Snapshot                         // runs report semantic events
    fn with_bodies(&self) -> Snapshot                    // `@fast` fns run their CP bodies
}

impl Session {
    fn open(snapshot: Snapshot) -> Session              // computes nothing yet
    fn snapshot(&self) -> &Snapshot
    // typing path
    fn diagnostics(&self, upto: Layer) -> List<&Diag>    // Layer::Typed | Layer::Program
    fn units(&self) -> (&[Unit], &[Diag])                // the editor's view: each fn or statement
    fn std_unchecked(&self) -> List<(Module, usize)>     // every std fn checked (suite 4)
    // build and run
    fn program(&self) -> Result<&Program, &[Diag]>
    fn verdict(&self) -> Verdict                         // Accepted | Rejected | Refused: the exit rule
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
| `complete` (next) | Scope, Signatures | not built |

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
| `Representation`, `human_bytes` | `layout(at)` | the linked program's layout |

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
statement with its line, its call depth and frames (`Enter`/`Exit`),
the values each one bound or wrote (`Bind`/`Write`), and the visit
count per line inside a call (`At.nth`).

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

- `Std::read` reads live `std/` beside the crate. A snapshot of an
  edited std (the editor on a std file) needs `Std` built from given
  texts, the way `cpl-facade`'s `with_std` does.
