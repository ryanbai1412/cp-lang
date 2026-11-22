# window.cp doc notes

## Contradictions

- `Sweep` (whole struct): Mo and divide-and-conquer DP must read the `Track`'s state after `move`, but `Sweep` has no `state` getter, only the positional `state` field. Left as is; a `@get fn state` is probably missing.
- `Undoing.pop`: it checks that the front is `x`, which needs `T: Eq`; the struct does not bound `T`. Signature left unchanged.

## Choices

- `Window.pop`: popping an empty window traps.
- `Tracked.pop`: it calls `remove(state, x)` without checking `x` against the front, because `Tracked` does not store elements.
- `Undoing.pop`: an empty window, or a front that is not `x`, traps. `Undoing` stores the elements, so the check is free.
- `Sweep.new`: it starts at the empty range at the first cut. `start` pushes its positions in increasing order.
- `Sweep.move`: "expands before shrinking" is made exact. First push the new right positions (increasing), then the new left positions (decreasing). Then pop the removed left positions (increasing), then the removed right positions (decreasing). The `Track` receives positions of `D`, since `Sweep` holds no array.
