# Events by source identity (refactor 6)

**Status:** design note, 2026-11-27, lane H. Contract 8 is the contract;
this says how the new pipeline meets it and in what order.

## The problem

The new engines report events as strings (`ev enter f 3`, `ev set x 4`)
with no place in the source, a trap names a line number, and every tool
that traces still reads the old interpreter's step log
(`cpl_interp::trace`, 58 lines in `cpl-facade`). A line is not an
identity: an edit above it moves it, and two statements on one line are
one place.

## Ids

- An event names a **site**: `SourceId`, an `ItemId` (the item's path)
  and a place counted from the item's start. Today that place is the
  node's item-relative span. At syntax.md step 3 it becomes the node's
  preorder index, `(ItemId, index)`; that change is inside
  `cpl_next_ir::ids` and lowering's `site`, and no event consumer
  changes with it. No event names a byte offset, a line or a bytecode
  offset.
- `At { call: CallPath, site, nth }` (contract 8) is computed **once,
  from the stream**, by one fn in `cpl_next_ir::run`: the engines emit
  events in order and never count visits themselves, so every engine
  gives the same `At` by construction.
- A trap names its **origin**: the number link gives each site of the
  user's file in `Program::origins`, one per `SourceId` however many
  instances hold it, so every engine names the same one. The engines
  carry origins, never lines; the facade turns one into a line where it
  shows the trap (`Program::said`, and later `Session::source`). The C++
  prints its own trap, so `emit` takes each origin's line.

## The IR

- Lowering puts a `Stmt::Mark(Site)` before each source statement. It
  is generic IR, so it is cached once whatever a run asks for.
- Link keeps marks only when events are requested (requesting them is
  in the instance fingerprint): `add_events` turns each into a step
  event and places `Enter`, `Exit`, `Bind` and `Write`; otherwise link
  drops every mark, so an untraced program has none and pays nothing.
- `Stmt::Event` carries a typed `EventPoint { kind, site, vals }`, not
  a label string. `kind` is `Enter(instance)`, `Exit`, `Step`,
  `Bind(local)` or `Write(root)`.

## The engines

The reference interpreter and the bytecode engine both return
`Vec<Event>`, each an `EventPoint` with its values shown by the one
`show` rule. The differential test compares the two streams on every
passing case run with events. Generated C++ emits none: native trace is
contract 8's later backend, and the debugger runs the engine (round 5).

## The facade

`cpl-next`'s facade (lane E) gets the queries tools read, so no tool
names an engine:

- `Session::trace(input) -> Recording`: the run's stream with its `At`s.
- `Recording::step(i)`, `calls()`, `place(i) -> At`, and
  `Session::source(site) -> (line, column, span)`: where a site is in
  the file now, from the `spans` table. It is the only place that turns
  an id into a line.

`cpl-facade`'s trace moves onto these at the stage 5 switching landing,
when `cpl-interp` is deleted.

## Counters (hard zero)

- `trace_old_interp_uses`: tool and facade lines naming
  `cpl_interp::trace`. It is 58 now and reaches 0 at the switching
  landing that deletes `cpl-interp` (rule 13).
- `event_labels`: `Stmt::Event` built from a label string. It goes from
  today's 4 to 0 in the first landing.

## Landings

1. Typed `EventPoint`, marks, `At` from the stream, with the
   interpreter and the engine agreeing; `cpl-next --trace` prints it.
2. The facade's `trace` and `source` queries (with lane E).
3. `Bind`/`Write` values for composite types under the one `show`
   rule; the trap's site as `SourceId`, not a line.
4. At the switch: `cpl-facade`'s trace and every tool on it; the old
   step log is deleted.
