# Friction report: noip19p6

There is no dream.cp; the ideal is below. The port roots at a centroid (`Rooted.centroid()`) and walks once with four `Fenwick<Size, Int>` counters over "vertices removed", as the editorial does.

## Dream

```cp
let t = Rooted(g, root: Rooted(g, root: V.first!).centroid())
...
t.walk(enter: x => ..., leave: x => ...)
```

## The DFS is a recursive fn over four global counters
- **Where:** lines 49-62: `fn walk(x: V)` adds to `above`, `seen` and `chain` on entry, recurses over `t.children(x)` and undoes the adds on exit.
- **What felt wrong:** an enter/leave traversal is spelled as recursion with paired add/undo lines. Nothing ties the undo to its add, and a 3e5-deep path relies on the emitted 1 GB stack.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `for (x, phase) in t.euler()` (enter and leave events) or `t.walk(enter:, leave:)` closures, iterative in the emitted C++.
- **General fix:** a std Euler-tour event iterator on `Rooted`. Owner: stdlib.
- **Severity:** low.

## `want` clamps a computed size into the domain by hand
- **Where:** line 50: `Size(max(n - 2 * t.size(x), 0))..=Size(n - 2 * heavy[x])`.
- **What felt wrong:** an interval of sizes that may start below 0 is clipped with `max(..., 0)` before it can become a `Size`.
- **Goal it hurts:** express.
- **Ideal code:** `Size.clip(n - 2 * t.size(x)..=n - 2 * heavy[x])`.
- **General fix:** a domain method that clips an `Int` range to the domain's range. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 487, accepted C++ 1455 (comments excluded).
- **Runtime and memory on gen.py --max:** port 1.77 s / 90 MB, accepted C++ 2.42 s / 100 MB, with the same output.
- **Emitted C++:** 32102 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
