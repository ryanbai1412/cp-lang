# Contract 8: execution → events

Trace, the debugger, coverage, profiling, `explain --run` and stress
shrinking are all readers of one event stream.

## What crosses

```rust
struct Request { classes: Set<EventClass>, scope: UserCode | Into(Set<LibName>) | All, filter }

enum Semantic {                 // compared across backends and modes
    Enter { instance: InstanceId, args }, Exit { result },      // user-visible functions only
    Bind  { local: SourceId, value },
    Write { root: SourceId, path: List<Step>, op: Set | Push | Pop | Insert | Remove | Resize, value },
    Read  { root, path },                                       // off by default
    Branch { at: SourceId, taken }, Output { bytes },
    Input { token: Bytes, value, read_at: SourceId },          // which line consumed which token
    OpBuilt { value, laws: List<DeclId> },                      // an operation value was made; stress samples its laws
    Fail  { check: CheckName, at: SourceId, origin },
}
enum Mechanical {               // reported, never compared
    Alloc, Free, CheckPassed, EnterCompilerMade, Instruction,
}
struct Recording { snapshot: Snapshot, input: RunInput, compiler: CompilerVersion, events: Stream }

// One rule for showing a value, used by print, failures, hover, the debugger and the trace viewer.
struct Shown { text: Str, parts: List<(Label, Shown)>, elided: Option<Count> }
fn show(value, type_: Type, budget: Budget) -> Shown     // driven by the type's declared logical view

// Where an event is in a run, so two runs can be lined up.
struct At { call: CallPath, site: SourceId, nth: u32 }    // nth time this site ran inside this call
struct CallPath(List<(InstanceId, u32)>)                  // each caller, and which of its calls this was
fn align(a: &Stream, b: &Stream) -> List<Aligned>         // Same | Differs { at, a, b } | OnlyA | OnlyB
fn first_difference(a, b) -> Option<Differs>
```

## Producer guarantees

- Requested semantic events are identical on every backend and stage.
- An event names source identities, never addresses or bytecode
  offsets. A write inside library code is attributed to the user's
  binding it belongs to, with the path from it.
- Events not requested cost nothing: they are not compiled in.
  Requesting events is part of the instance fingerprint.
- Tracing stops at the library boundary unless asked to go in.
- A recording replays to the same events on the same compiler version.
- Every semantic event carries its `At`. Two runs of the same program
  on different inputs, or of two versions of a program whose
  statements share lineage, align by `At`, not by position in the
  stream. This is what "compare two runs" and stress's "where do the
  solution and the brute force part ways" read.
- `show` gives the same text everywhere a value appears. `print(x)`
  is `show` with no budget; a failure message and hover use a small
  one. A library type's view is declared once, in its source.
- The stream is complete: with every class requested and scope `All`,
  the recording alone reproduces every value the program computed. A
  viewer needs no engine.
- Time per call and per line is a reader of `Enter`/`Exit` with a
  mechanical clock field. It is profiling, is never compared, and has
  its own overhead bar, separate from the 20% trace bar.

## Consumer may assume

- Stepping, breakpoints, watch, coverage and time travel need only
  semantic events and a recording.
- Stepping backwards in a live session needs the bytecode engine.
  From a complete recording, any reader can move both ways, whatever
  engine recorded it.

## Must never

- No tool may read engine state directly or ask the engine a question
  the event stream does not answer.
- No comparison across backends may include mechanical events.

## Checked by

- Differential: semantic streams of raw, canonical, optimized, engine
  and native trace on the corpus.
- Benchmark gate: tracing user code within 20% of an untraced run.

## Open

- Attribution of a write through a stored lend or a view to its root.
- Whether `Bind` and `Write` values are full values or handles, which
  decides the 20% budget for large containers.
- **Audit 4 (interactions).** A semantic write is a change to a user
  binding seen at the library boundary, in the library type's declared
  logical view (keys and values for a map). The same declaration
  drives printing, equality and the debugger. Storage operations
  report mechanical events only. Otherwise `h[5] = x` shows up as
  writes to the map's private arrays.
- Identity for events and recordings is a statement's lineage (which
  statement replaced which), not its content hash, or every lambda
  edit loses breakpoints.

## Round 5 (2026-11-25, not yet reviewed)

- **Values in events are full logical values in a form that does not
  depend on layout**, so a recording can be replayed against an
  edited program and compared across engines. Decided; it was open.
- **A debug run executes raw instance IR on the engine**, where event
  points are defined. A fused generator or a specialised method is
  still a frame there.
