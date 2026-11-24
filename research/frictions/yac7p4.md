# Friction report: yac7p4

There is no dream.cp; the ideal is the port with the gap below closed. The occupied towns form a subtree `S`:
- the port tracks `S`'s top and its leaves;
- `jump_while` finds the town to add;
- adjacency inside `S` is a `[V]Set<V>`;
- each deleted leaf pays for its insertion.

## Dream

The port as written, with `leaves.remove_where(x => x != met)` returning the removed towns.

## A set is copied in order to delete from it while walking it
- **Where:** `for x in [x for x in leaves if x != met]:`, followed by `leaves.remove(x)` inside the loop.
- **What felt wrong:** the comprehension copies every leaf only because the loop body mutates `leaves`. "Remove all but one and visit each removed element" is one operation.
- **Goal it hurts:** express, costs-visible.
- **Ideal code:** `for x in leaves.remove_where(x => x != met):`.
- **General fix:** a `Set.remove_where(pred) -> List<T>` (or `drain_where`) that removes the matches and returns them. Owner: stdlib.
- **Severity:** low.

There are no other dream/port gaps.

## Measurements

- **Tokens:** port 434, accepted C++ 1662 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.23 s / 42 MB, accepted C++ 0.88 s / 82 MB, with the same output.
- **Emitted C++:** 29239 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
