# Friction report: noi08p5

Port and dream share the geometry:
- a cloud is a point (phase, phase + width) on the rightward sheared plane and (phase + p, phase - width + p) on the leftward one;
- a query is a rectangle on each plane, plus its copy one period on;
- each plane is counted with a `Fenwick2D<PX, PY, Int>`.

## Two Fenwick trees don't fit, so the port makes two passes
- **Where:** port lines 49-73: the events are kept as `enum Event` values, and `for plane in 0..2:` clears one tree, replays every event and adds the second plane's counts into `caught`. The dream keeps `rising` and `falling` and answers each `Ask` at once.
- **What felt wrong:** "count clouds over [l, r] at time t" becomes a two-pass replay with per-plane origin offsets `(ox, oy)` threaded through `count`. The reason is memory: a 2000 x 3000 `Fenwick2D` of `Int` is 48 MB, and two of them plus the event list break the 128 MB limit.
- **Goal it hurts:** express, revision.
- **Ideal code:** the dream's `rising`/`falling` pair, with `Fenwick2D<X, Y, I32>` (counts never exceed 2e5).
- **General fix:** a narrower element type for `Fenwick2D` (and `Fenwick`) counts, chosen by the program; today the element type is used as written, and the port would need it to keep both trees. Owner: stdlib.
- **Severity:** medium.

## Events are read by hand instead of as a tagged enum
- **Where:** port lines 31-47: `let kind: 1..=3 = read()` then `match kind:` with a read per arm. The dream reads `read<[q]Op>()` over `enum Op` with `@tag(1) Add`, `@tag(2) Ask`, `@tag(3) Remove`.
- **What felt wrong:** the input format is a tagged union, and the port re-derives it with a `match` on a number, a hand `t` read before the tag's fields, and `assert l <= r and d != 0`.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `@tag` enum read.
- **General fix:** none in the language: the dream's read compiles. It is a rewrite owed by the port. It waits on the memory fix above, since the two-pass port needs its event list anyway.
- **Severity:** low.

## Measurements

- **Tokens:** port 657, accepted C++ 1837 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.21 s / 75 MB, accepted C++ 0.37 s / 72 MB, with the same output.
- **Emitted C++:** 19794 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
