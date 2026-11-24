# Friction Report: cco07p5

The v0.2 port rolled its own linear congruential generator in a global `rng_state`, found a hit
block with `while true` and `break`, kept candidates in a `State` struct built by nested push
loops, and filtered them through an index loop into a second list. Now a seeded std `Rng` picks
blocks, the search is one `while not look(...)`, candidates are a `Guess` named tuple built and
filtered by comprehensions, and the hit test is `(c.p <= mid) == hit`.

## A typed `ask` reply needs a separate binding
- **Where:** `fn look`, which binds `let answer: Str = ask("Look", l, r)` before comparing.
- **Ideal:** an enum reply type read by `ask`, `enum Seen: @tag("True") Yes; @tag("False") No`,
  so `look` returns `ask("Look", l, r) == Yes`.

## Ring positions are plain `Int` with `% n`
- **Where:** `(x + v) % n` and `(c.p + c.v) % n`.
- **Ideal:** a cyclic domain `domain Pos = 0..n` with wrapping `+`, as `D.step` gives for one step.

## No selection of the k-th smallest
- **Where:** `cands.sort()` then `cands[(cands.len - 1) / 2].p`, once per round.
- **What felt wrong:** the round wants the median candidate; sorting all of them says more than
  that and costs O(m log m) where selection is O(m). There is no value-returning selection:
  `print(xs.select(1))` on `var xs = [5, 1, 4]` fails under `cpl run` (0.2) with
  `error[E0302]: this has no value to print`.
- **Goal it hurts:** express, performance.
- **Ideal code:** `let mid = cands.select((cands.len - 1) / 2).p` (or `cands.median`).
- **General fix:** a std `select(k) -> T` (introselect, like C++ `nth_element`) on lists, by key
  as `sort` is.
- **Severity:** low
- **Owner domain:** stdlib

## Dream
```cp
let n: 2..=1e5 = ask("GetSize")
var lo = rng.int(0, (n - 1) / block) * block
while ask("Look", lo, min(n, lo + block) - 1) == No: lo = rng.int(0, (n - 1) / block) * block
var cands = [(p: Pos(x) + v, v:) for v in 0..n for x in lo..min(n, lo + block)]
while cands.len > 1:
    let mid = cands.select((cands.len - 1) / 2).p
    let hit = ask("Look", 0, mid) == Yes
    cands = [(p: c.p + c.v, v: c.v) for c in cands if (c.p <= mid) == hit]
print("Guess", cands[0].v, cands[0].p)
```
