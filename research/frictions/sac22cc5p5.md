# sac22cc5p5 — Querying Rectangles

## Hash2D keeps two full prefix tables
- **Where:** `port.cp`, `Hash2D.new`.
- **What felt wrong:** at 5000 x 5000 the two `n m` Int prefix tables cost 400 MB and most of the
  run; the max case takes about 3 s native (the accepted C++ about 4.6 s on the same machine), so
  the seeded tests stop at size 1024 rather than `--max`, which `cpl test` would time out at the
  generic 2 s.
- **General fix:** a `Hash2D.windows(k)` that rolls k x k hashes row band by row band in O(m)
  extra space, the way the original does.
- **Goal it hurts:** fast.
- **Severity:** medium.

## Dream

```cp
let (n, m, k) = read<(1..=5_000, 1..=5_000, 1..=5_000)>()
let g = read<[n, m]Char>()
print(Hash2D(g).windows(k, k).distinct_count)
```
