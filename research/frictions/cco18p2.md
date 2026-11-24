# Friction Report: cco18p2

## Printing the strict upper triangle
- **Where:** `for r in V.all.drop_last(1): print([... for c in V if c > r])`.
- **What felt wrong:** "the cells above the diagonal, row by row" filters every row instead of
  iterating the sub-range after `r` directly.
- **Ideal:** `r.after` as the domain range `r + 1..` (`for c in V.after(r)`).

## Dream
```cp
const N = 100
domain V = 0..N
fn cheap(r: V, c: V) -> Bool:
    if r.ord < 2: return c.ord <= 3 and (r.ord, c.ord) != (1, 3)
    if r.ord % 2 == 0: c.ord == r.ord + 3 else: c.ord == r.ord + 1 and r.ord + 2 < N
print(N)
for r in V.all.drop_last(1): print([if cheap(r, c): 1 else: 100 for c in V.after(r)])
```

- Tokens: port 138, original 498
- Max input (`gen.py --max`): port 0.00 s, 3200 KB, original 0.00 s, 3584 KB
