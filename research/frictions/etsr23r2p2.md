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
- **Ideal code:** `type Val = Sat<0..=254>` with `+` and `*` that clamp, and
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
- **Ideal code:** `prod[a][b][w].lower(rest + 1)` (a `?T` method setting
  `Some(min(old, x))`).
- **General fix:** a std `?T` relax method (`lower`/`raise`), and a lint
  that rewrites `x = min(x ?? c, c)` to it.
- **Severity:** low.

## Dream

```cp
@!multitest
let s: Str = read()
let k: Sat<0..=254> = read()
type Val = Sat<0..=254>
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
