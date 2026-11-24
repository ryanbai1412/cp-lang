# Friction report: csps21p2

The port is the dream except for one typed `let`: the positions strictly inside a pair.

## A cut range iterates as cuts even where positions are meant

- **Where:** `let inside: Range<Pos> = l.r..r.l` then `for i in inside:`
- **What felt wrong:** `l.r..r.l` is exactly "the positions strictly between l and r", and
  `Prefix.fold` already accepts it as a `Range<Pos>`. Iterating it directly yields `Pos.cuts`,
  so indexing `seq[i, ...]` fails; the port needs a typed `let` to say the same range again.
  The other spelling, `l <.. r`, is not implemented.
- **Goal it hurts:** express (one range, two spellings depending on context)
- **Ideal code:** `for i in l.r..r.l: ... seq[i, r.prev!]`
- **Repro (fresh build):**
  ```
  domain P = 0..5
  let a = P.first!
  let b = P.last!
  let v: [P]Int = 0
  for i in a.r..b.l: print(v[i])
  ```
  `for i in a <.. b` gives `error[E0504]: not implemented yet: a range that leaves out its
  start, lo <.. hi (not in the design)`; the loop above gives
  `error[E0301]: v is indexed by P, but i is a P.cuts`.
- **General fix:** check: a cut range in a position context (loop over it then index by it)
  infers `Range<Pos>`, as it already does for a `Range<Pos>` parameter.
- **Severity:** low

## Measurements

gen.py --max (n = 500, all `?`): port 0.10 s, accepted C++ 0.15 s.

## Dream

```cp
// Source: CSP-S 2021 P2 (Bracket Sequence), https://dmoj.ca/problem/csps21p2
// Problem: a pattern of n <= 500 characters from ( ) * ?, and k <= n. Count, mod 1e9+7, the
//   fillings of the ?s that are super bracket sequences: (), (S), AB, ASB, (AS), (SA) with
//   A, B super sequences and S a run of 1 to k stars (in () and (S), up to k stars inside).
// Solution: interval DP, each string split one way. wrap[l, r]: l and r match around only
//   stars, stars then a sequence, or a sequence then at least one star. seq[l, r] =
//   wrap[l, r] + sum over the end i of the first wrapped block of wrap[l, i] times
//   tail[i, r], 0 to k stars after i and then a sequence. A prefix count of the positions that
//   can't be stars answers "is this run all stars" in O(1). O(n^3).

type Z = Mod<1_000_000_007>
let n: 1..=500 = read()
let k: 1..=n = read()
domain Pos = 0..n
let s: [Pos]Char = read()

fn can_be(i: Pos, c: Char) -> Bool = s[i] == '?' or s[i] == c
let not_star = Prefix([Pos] i => if can_be(i, '*'): 0 else: 1)
fn stars(run: Range<Pos>) -> Bool = run.len <= k and not_star.fold(run) == 0

var wrap: [Pos, Pos]Z = 0
var seq: [Pos, Pos]Z = 0
var tail: [Pos, Pos]Z = 0
for l in Pos.rev:
    for r in l.after:
        if can_be(l, '(') and can_be(r, ')'):
            if stars(l.r..r.l): wrap[l, r] += 1
            for i in l.r..r.l:
                if stars(l.r..i.l): wrap[l, r] += seq[i, r.prev!]
                if i.r < r.l and stars(i.r..r.l): wrap[l, r] += seq[l.next!, i]
        for j in l..r:
            if stars(l.r..j.r): tail[l, r] += seq[j.next!, r]
        seq[l, r] = wrap[l, r] + sum(wrap[l, i] * tail[i, r] for i in l..r)
print(seq[Pos.first!, Pos.last!])
```
