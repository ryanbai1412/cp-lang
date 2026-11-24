# Friction report: noip22p2

There is no dream.cp; the ideal is below. The port is the editorial's greedy:
- `twin[t]` says when each first card is cancelled;
- `stacks[s]` is a `Deque<Time>` of deal times;
- the Bitsets `empty`, `single` and `ripe` classify the stacks;
- `place` and `match_bottoms` emit `Move`s.

This review fixed one owed spelling: `empty` was `Bitset<Stack>.new()` filled by `for s in Stack: empty.add(s)`, and is now `Bitset<Stack>.full()`.

## Dream

```cp
enum Kind = Empty | Single | Ripe | Other
var kind = Buckets<Stack, Kind>(all: Empty)
fn refresh(s: Stack): kind[s] = classify(stacks[s])
...
let s = if kind.count(Empty) == 0: kind[Single].max(by: ...)! ...
```

## Three Bitsets are a hand-kept partition
- **Where:** lines 51-58: `refresh` removes `s` from all three sets and re-adds it to the one that fits.
- **What felt wrong:** "each stack is in exactly one class" is kept by hand. A missed `remove` would leave a stack in two classes with no check.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** the dream's `Buckets<Stack, Kind>`: a partition of a domain into enum classes, with per-class `first`/`max(by:)`/`count`.
- **General fix:** a std partition structure keyed by a domain and an enum (one Bitset per class, moves are O(1)). Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 638, accepted C++ 1254 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.23 s / 18 MB, accepted C++ 1.04 s / 34 MB, with the same output.
- **Emitted C++:** 27497 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
