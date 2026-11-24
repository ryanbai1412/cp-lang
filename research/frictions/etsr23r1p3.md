# etsr23r1p3: every ordering as a subsequence

The dream is a memoised `latest(set)` over `Subset<Letter>` reading a std
"last occurrence before each cut" table, with `?` poisoning the min. The port
builds both by hand.

## No last-occurrence-before-cut table
- **Where:** port lines 17-21 build `before: [Pos.cuts][Letter]?Pos` row by row.
  Dream: `let before = s.last_before(Letter)`. Today: E0203 "a `Str` has no
  method `last_before`".
- **What felt wrong:** the "next/previous occurrence of each letter" table is
  a standard subsequence-automaton step; every port hand-writes the copy loop.
- **Goal it hurts:** express.
- **Ideal code:** `s.last_before(Letter)` / `s.next_after(Letter)` returning
  `[s.cuts][Letter]?s.Pos`.
- **General fix:** a std subsequence-automaton (`next_occurrence`) over a
  keyed sequence and a value domain.
- **Severity:** low (a five-line loop).

## Min over an optional that poisons on None
- **Where:** port lines 26-33: a `best` loop with `else: best = None; break`.
  Dream: `set.min(x => before[latest(set.without(x))?][x]?.l)?` inside a
  `@memo(array)` fn.
- **What felt wrong:** "the min, unless one of them is missing" is a fold the
  reader must decode; `?` inside a lambda does not propagate to the caller.
- **Goal it hurts:** express.
- **Ideal code:** `set.items.map(f).all_some()?.min()` or the dream line.
- **General fix:** a `Seq<?T>.collect_some() -> ?List<T>` (or `all_some`) in std.
- **Severity:** low.

## A type alias of a domain's cuts is not a value
- **Where:** `type Cut = Pos.cuts` then `Cut.last` is E0302 "`Cut` is a
  type, and a value is needed here"; the port spells `Pos.cuts` each time.
- **Goal it hurts:** coherence (one name per index space).
- **General fix:** let a type alias of a domain-like type forward its
  associated values. Routed to Core with the next batch.
- **Severity:** low.

## Subset table sized at top level even when unreached
- **Where:** with `domain Letter = 0..m` at top level, a `[Subset<Letter>]`
  local in a fn that is called only for m < 22 traps for m = 26 (R0401
  "Subset over 26 values: at most 25" at the declaration; the fn body never
  runs). The port declares `Letter` and `Pos` inside `solve()`. Repro sent to
  Core.
- **Goal it hurts:** correct (a guarded path traps).
- **Severity:** medium (silent until a large m test).

## Dream

Kept here until the std features above exist (a dream.cp must pass `scripts/corpus.py verify`).

```cp
// Source: ETSR 2023 R1 P3, https://dmoj.ca/problem/etsr23r1p3
// Problem: multitest; a string s over the first m letters. Is every ordering of the m letters a
//   subsequence of s?
// Solution: latest[S] is the latest cut after which every ordering of S still fits. An ordering
//   that starts with x needs an x before latest[S - x], so latest[S] = min over x in S of the last
//   x before latest[S - x]; the answer is whether latest[all] exists. O(2^m * m).
@!multitest
let m: 1..=26 = read()
let s: Str = read()
domain Letter = 0..m
let before = s.last_before(Letter)               // [s.cuts][Letter]?s.Pos
if m >= 22: return print("NO")                    // too many orderings for |s|
@memo(array)
fn latest(set: Subset<Letter>) -> ?s.cuts:
    if set.is_empty: return s.cuts.last
    set.min(x => before[latest(set.without(x))?][x]?.l)?   // any letter that cannot start poisons the set
print(yes(latest(Subset<Letter>.full) != None))
```
