# Friction Report: cco04p4

The v0.2 port moved two points minute by minute with a `var best = inf` running minimum. Now each
orbit is read as a list of its positions over one lap, and the answer is one
`min(... for t in 0..lcm(a.len, b.len))!` with no sentinel.

## A lap of unit steps is built by hand
- **Where:** `fn read_orbit`, which pushes `at` once per unit of each `d axis` move.
- **Ideal:** `moves.flat_map(m => m.unit_steps).scan(start, +)` with a `Dir4`/`Axis` step type,
  as the dream's `w.moves.ring.scan(...)` sketches.

## `min` over a generator is optional even when the range is non-empty
- **Where:** `min(...)!` over `0..lcm(a.len, b.len)`, where the lcm is at least 1.

## Dream
`dream.cp` (unverified, older syntax) zips the two ring scans and takes `period + 1` positions.
Today's closest form:
```cp
let (a, b) = (read_orbit(), read_orbit())
print(min(hypot(a[t % a.len] - b[t % b.len]) for t in 0..lcm(a.len, b.len))!, digits: 2)
```

- Tokens: port 299, original 692
- Max input (`gen.py --max`): port 0.34 s, 3660 KB, original 1.48 s, 3968 KB
