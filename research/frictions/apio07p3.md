# apio07p3

Ported from `corpus/submissions/5532787.cpp`. No `dream.cp`; the dream is below.

## Dream
```cp
// the pens a child names, as positions in its window starting at `pen`
fn read_view(pen: Pen, count: 0..=5) -> View:
    var s = View.empty
    for _ in 0..count:
        let x: Pen = read()
        s = s.with(Window.nth((x - pen) % n))
    s
...
    let fear = read_view(pen, fears)
    let love = read_view(pen, loves)
...
// the view one pen further on: every pen moves down one, then decide the new one
fn slide(v: View, empty_new: Bool) -> View:
    let kept = v.shifted(-1)
    if empty_new: kept.with(Window.last!) else: kept
```
The rest of `port.cp` stays: `happy[p, gone]` over `[Pen, View]`, and for each starting view an
optional DP `[View]?Int` round the ring that must end where it began.

## The fear and love sets are the same loop written twice
- **Where:** `port.cp` lines 21-28: two copies of `var s = View.empty; for _ in 0..c: let x: Pen =
  read(); s = s.with(Window.nth((x - pen) % n))`.
- **What felt wrong:** the same read-into-a-window-set code is written once per list.
- **Goal it hurts:** each fact once (idiomatic-code/R4).
- **Ideal code:** `fn read_view` in the dream, called twice.
- **General fix:** a port rewrite. Today's language has everything it needs.
- **Severity:** low.
- **Owner:** idioms (this port); lands with the next port-edit commit (the `copies` slice the
  pre-commit maps to `.cp` edits is red on main, routed to Perf C).

## The slide re-derives `Subset.shifted`
- **Where:** `port.cp` `fn slide`: `for w in Window: if let down = w.prev and v.contains(w): kept
  = kept.with(down)`.
- **What felt wrong:** "every position moves down one" is `std/subset.cp`'s
  `shifted(k)`. The port spells it as a loop over 5 positions with an `if let` each, and it
  runs 32 x 2 x n x 32 times.
- **Goal it hurts:** std over hand-rolled code (idiomatic-code/R3).
- **Ideal code:** `v.shifted(-1)`. Checked under `cpl run`: `{0, 1, 3}.shifted(-1)` is
  `{0, 2}`.
- **General fix:** a port rewrite.
- **Severity:** low.
- **Owner:** idioms (this port); lands with the fear/love rewrite.

## A subset cannot be collected from a sequence
- **Where:** what would be the one-line `read_view`.
- **What felt wrong:** `Subset<D>` has `empty`, `single` and `with`, but no way to build one from
  a sequence of members, so a set read from input needs a `var` and a loop.
- **Run:** `let u: Subset<W> = [W.nth(1), W.nth(2)].collect()` under `cpl run`:
  `error[E0302]: 'u' is declared a Subset<W>, but '[W.nth(1), W.nth(2)].collect()' is a List<W>`.
- **Ideal code:** `[read(Pen) ... for _ in 0..count].collect()` into a `View`, the way
  `collect` already targets other containers.
- **General fix:** make `Subset<D>` a `collect` target (FromSeq) in `std/subset.cp`.
- **Severity:** low.
- **Owner:** Content (stdlib).

## Measurements
- Tokens: port 369, original 677.
- `--max` (best of 3): port 63.9 ms / 5.1 MB, original 365 ms / 5.5 MB (0.18x). Same output.
- Emitted C++: 22,081 bytes.
