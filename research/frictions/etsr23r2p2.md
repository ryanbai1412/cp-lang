# etsr23r2p2: fewest signs to reach a target

Two interval DPs over saturated values. The port writes the saturation and
the "keep the smaller optional" update by hand.

## No saturating integer type
- **Where:** port lines 12-14: `const CAP`, `domain Val = 0..=CAP` and
  `fn sat(x) = Val(min(x, CAP))`, then `sat(a.ord * b.ord)` and
  `sat(a.ord + b.ord)` at every combine.
- **What felt wrong:** "values only matter up to CAP" is one idea; the port
  spells it as a domain, a helper and `.ord` round trips at three sites.
- **Goal it hurts:** express.
- **Repro:** on a fresh `cpl`, `type V = Sat<0..=254>` gives
  `error[E0204]: no type or domain named Sat, and not in the stdlib (yet)`.
- **Ideal code:** `type Val = Sat<0..=255>` with `+` and `*` that clamp, and
  `Val` usable as a domain.
- **General fix:** a std `Sat<R>` (saturating range integer) implementing
  `Add`, `Mul` and `Index`.
- **Severity:** low.

## Keeping the smaller of an optional and a candidate
- **Where:** port lines 37-38 and 51-52: `let cand = ...` then
  `x = min(x ?? cand, cand)`. Writing the candidate inline is rejected with
  "write `(x ?? a) + b` ... or `x ?? (a + b)`", so each site needs a name.
- **What felt wrong:** relaxing a `?Int` DP cell is the most common DP
  update; it has no name.
- **Repro:** on a fresh `cpl`, `x.lower(3)` on a `?Int` gives
  `error[E0203]: a ?Int has no method lower`.
- **Ideal code:** `prod[a][b][w].lower(rest + 1)` (a `?T` method setting
  `Some(min(old, x))`).
- **General fix:** a std `?T` relax method (`lower`/`raise`), and a lint
  that rewrites `x = min(x ?? c, c)` to it.
- **Severity:** low.

## Saturating at the largest target
- **Where:** the last port (deleted in e0502f639 for missing judge limits) saturated at
  `CAP = 254`, the largest `k`, so `s = 300, k = 254` printed `0` (correct: `-1`).
  `CAP = 255` with `k` read as `0..=254` prints `-1` and passed its 10 cases.
- **General fix:** none in the language; a `Sat<R>` type would make the cap one
  spelling, where this bug is easier to see.

## Dream

```cp
@!multitest
let s: Str = read()
type Val = Sat<0..=255>  // 255 > k: a value past every target
let k = Val(read<0..=254>())
let number = (a, b) => Val(s[a..=b].digits_value)
@memo(array)
fn prod(a: s.Pos, b: s.Pos) -> [Val]?Int:
    var best: [Val]?Int = None
    best[number(a, b)].lower(0)
    for m in a..b:
        for (v, rest) in prod(m.next!, b).some_entries:
            best[number(a, m) * v].lower(rest + 1)
    best
var sum: [s.cuts][Val]?Int = None
for b in s.Pos:
    sum[b.r] = prod(s.Pos.first!, b)
    for a in s.Pos.first!.after.take_while(a => a <= b):
        for (head, before) in sum[a.l].some_entries:
            for (v, term) in prod(a, b).some_entries:
                sum[b.r][head + v].lower(before + 1 + term)
print(sum[s.cuts.last!][k], none: -1)
```
