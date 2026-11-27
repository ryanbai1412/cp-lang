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
| `hover`, `definition`, `complete` (next) | Scope, Signatures | not built |

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
3. `hover` and `definition` from Scope and Signatures, as lane I lands
   them.
4. At the switching landing, `cpl-facade`'s queries answer from this
   one.

## Open

- `Std::read` reads live `std/` beside the crate. A snapshot of an
  edited std (the editor on a std file) needs `Std` built from given
  texts, the way `cpl-facade`'s `with_std` does.
