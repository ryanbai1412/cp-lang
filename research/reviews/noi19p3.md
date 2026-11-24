# noi19p3

Score: 2/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. No dream (cap 3), and the friction report describes a different program: it says the port is one `IndexedHeap<I, Int>` per side pair over a `Side` enum with `[A, B, Both].permutations()`, while this port (lines 25-31) is five lazy `Heap`s of `(-value, index)`. Its measurement (2.16 s vs C++ 1.03 s) is of that other port.

## Checklist failures

- Lines 27-31 negate keys to get max-heaps; std has `Heap(by: desc)` (std/heap.cp) (idiomatic-code/R3).
- Line 60 turns missing gains into a `-1` sentinel before `max()` (idiomatic-code/R5).
- Lines 61-81 repeat `total += top` in three branches and the `if in_d[...]: spare += 1` bookkeeping in three (idiomatic-code/R4).
- The friction report does not describe this port, so its gaps and speed are unreported (idiomatic-code/R10).

## Owed rewrites

- Rewrite the friction report for this port and re-measure `--max`.
- Use `Heap(by: desc)` with `(value, index)` entries.
- Pick the best move as one `?(gain, move)` with `max_by`, then apply it once, instead of the `-1` sentinel and the branch-per-move totals.

## Language gaps

- None.
