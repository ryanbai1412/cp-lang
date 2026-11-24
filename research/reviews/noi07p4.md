# noi07p4

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`). The rethink is right: beads stay in physical slots of a lazy `SegTree` of `Run`s, rotation and flip only change the position-to-slot map (`slot`, line 35), and a segment is one or two slot ranges (`arcs`, lines 38-41).

## Checklist failures

- idiomatic-code/R4: the header's Solution says "(left colour, right colour, parts) and paint as the action ... map from position to slot" twice (lines 9-10 and 11-12). Not reported.
- The commands are an `if`/`elif` chain on `Str` (lines 44-66) with each argument read inside its branch, and `k: Int` (line 46) is unbounded.
- `Run(l: Int, r: Int, ...)` (line 22) holds colours as bare `Int`, though `bead` is typed `1..=colours`.
- The paint `Action` (lines 26-31) is spelled out because std has no assignment action over an optional monoid: in the friction.

## Owed rewrites

- Drop the repeated header sentence; read a tagged `enum Cmd` and `match` it; type the run ends as colours.

## Language gaps

- A std assignment action: in the friction.

## Dream notes

`SegTree(Run.join.opt(), ..., act: assign)` and `beads.apply(a, Run.of(x))` remove the hand action; the commands are not in the dream.
