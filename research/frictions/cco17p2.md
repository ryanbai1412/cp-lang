# Friction Report: cco17p2

## Joining optional (min, max) pairs
- **Where:** `fn join(a: ?Count, b: ?Count) -> ?Count`.
- **What felt wrong:** "the fewest and the most over the splits that work" needs a hand-written
  join of optional pairs; `min=`/`max=` work on one `?Int` each, but not on a record of both.
- **Ideal:** a `MinMax` monoid in std (`best.add(rest.fewest + k)`), with `?` meaning empty.

## Dream
```cp
let n: 1..=1e8 = read()
let m: 1..=1e8 = read()
@memo
fn districts(n: Int, m: Int) -> MinMax<Int>:
    if n == 0 or m == 0: return MinMax.of(0)
    let jump = if n % 2 == 0: n / 2 else: 2 * n
    fold(MinMax.empty, for k in max(1, (m - 2 * n) / jump - 1)..=m / jump:
        districts(m - k * jump, n).shift(k) | (if n % 2 == 0: districts(m - k * jump, n).shift(k / 4 + k % 4) else: MinMax.empty))
let both = districts(n, m) | districts(m, n)
print(both.min!, both.max!)
```

- Tokens: port 308, original 849 (the only C++ submissions are TLE; no accepted original)
- Max input (`gen.py --max`): port 0.00 s, 3072 KB
