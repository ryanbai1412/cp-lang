# cco22p5

Score: 4/5
Dream: 4/5

Cold review against the friction's `## Dream`. Each company is a `Net` over `Dsu<House>` (lines 25-38), B with rollback undone by `mark`/`undo_to` (lines 45-46, 69), the overlap `HashMap` is keyed by the `(House, House)` group pair (line 49) and the answer prints with `none: -1` (line 79). The two-pointer sweep in `relax` (lines 65-73) is the header's argument.

## Checklist failures

- R7: `history.push((mark, pairs, net_b.link(l.u, l.v)))` (line 46) performs the merge inside the argument of a push.
- R5, minor: the header gives `a.w, b.w: 1..=1e9` (line 13) but both reads take `w: 0..=1e9` (lines 21-22).
- R7, minor: `enter`/`regroup` (lines 51-59) thread `both`, `together` and `group` as globals mutated from three call sites.

## Owed rewrites

- Bind `let moved = net_b.link(l.u, l.v)` before the push (line 46).
- Read `w: 1..=1e9` to match the Bounds line.

## Language gaps

- `List.take()` to move a cell out (lines 33-34 clone then clear): in the friction, with a repro.
- `Mark` needs `use std.dsu.{Mark}` (line 14): in the friction.

## Dream notes

- The dream covers only `Net` and `alone` (the `take()` change), judged as the port with that change; it reads like the editorial, and the push-with-merge at line 46 is a port-side fix (R7); bind the merge as its own statement there too.
