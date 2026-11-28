# Contract 0: the world → a snapshot

A snapshot is the only input of every layer. Nothing below the command
line reads a file, the clock, the environment or a random source.

## What crosses

```rust
struct Snapshot {
    files:    Map<FileId, Text>,          // user files, exact bytes
    manifest: Option<Manifest>,           // cp.toml, parsed
    libs:     Map<LibName, LibPin>,       // std and each pinned library
    profile:  JudgeProfile,               // data, see contract 7
    checks:   Set<CheckName>,             // which checks are kept; "checked" and "release" are two named sets
    overlay:  List<(DeclId, DeclId)>,     // run B where A is called: how stress compares an alternative
    contexts: ContextDefaults,            // check switches, lint levels
    compiler: CompilerVersion,
}
struct LibPin { hash: ContentHash, store: Option<LibStore> }   // store: contract 2 and 4 outputs, prebuilt
struct RunInput { entry: Entry, io: Io, out: Option<Writer> /* where a `Stream` run's output goes as it is flushed (R10.4) */,
                  args: List<Str>, choices: ChoiceTape /* seed, or recorded generator choices to shrink */,
                  limits: Limits, cold_caches: bool }
enum Entry { Program,                              // the file's statements, top to bottom
             Call { f: DeclId, args: List<Value> } // one fn on given values: "try this part" in the editor, a unit test, a law sample
           }
enum Io    { Fixed(Bytes),                         // ordinary problems
             Stream(Reader),                       // read as the program reads: a terminal or a pipe
             Peer { program: Snapshot, input: Bytes }, // interactive and grader problems: a second program answers back
             Transcript(List<Turn>) }              // a recorded exchange, replayed
struct Workspace { problems: Map<ProblemId, Problem>, shared: Manifest }
struct Problem   { snapshot: Snapshot, tests: List<Test>, limits: Limits, judge: JudgeProfile, kind: Batch | Multitest | Interactive | Grader }
```

`RunInput` is separate: it changes what a run prints, never what the
compiler answers.

## Producer guarantees (CLI, language server, playground)

- Two equal snapshots give byte-identical answers from every query.
- A new snapshot made by changing one file shares everything else.
- A run with `Io::Peer` records its exchange as a `Transcript`; rerun
  on the transcript, the program sees the same bytes at the same
  points without the peer running.
- `Entry::Call` runs the statements the fn's captures need first
  (the units it depends on, in file order) and nothing else.
- A snapshot can be written to one file and loaded on another machine
  with the same compiler version.

## Consumer may assume

- A `FileId` is stable for the life of a session and maps to one path.
- A library's `hash` covers its normalized source and everything it
  pins, transitively.

## Must never

- A layer must not look up anything by path, read `cp.toml` itself, or
  ask what platform it runs on. Platform facts are in `profile`.

## Checked by

- Build graph: only the CLI and language-server crates may depend on
  file-system, clock or environment APIs. (Compiler-held.)
- Test: run every query twice on a reloaded snapshot and compare bytes.

## Open

- Whether `salt` belongs to `RunInput` or to the profile, once hash
  maps iterate in insertion order.
- **Audit 4, finding 5.** `--laws=off`, the stress size caps, and the
  salt all change checking, instantiation or output. Rule adopted:
  every such switch is a context value in `contexts`, so an instance
  records the ones it uses and no identity field is added. `overlay`
  is one of them. The salt moves from `RunInput` into the snapshot.
- A snapshot is one program; `Workspace` holds a contest folder. How
  two problems share checked library units is contract 2's store.
- `Entry::Call` on a fn whose captures depend on `read()`: the run
  needs input for those statements, or values supplied for them.
  Rule to decide: supply values, or refuse with the list of inputs.
- `Io::Peer` deadlock and turn limits: who reports "both sides are
  waiting", and as which `Outcome`.

## Round 5 (2026-11-25, not yet reviewed)

- **The hash salt is in the snapshot**, not in `RunInput`: it can
  change which checks are proven. `RunInput` holds only what changes
  a run's output.
- **The manifest reaches the formatter.** `cp.toml` has
  `indent_width` and `line_width`; the formatter spec says "no
  options". One of the two changes.
