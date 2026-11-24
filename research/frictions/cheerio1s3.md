# cheerio1s3 — Stock Trading

From each point `p`, a later point `q` makes a trendline when its slope from
`p` ties or beats the steepest or the shallowest slope seen so far. The
natural form is a scan of the slopes after `p`; spec.cp writes it as a
running `?Ratio` minimum and maximum instead.

## `scan` over a slice can't be read at its cuts
- **Where:** the earlier spec.cp:
  ```cp
  let s = pts[p.r..].map(q => Ratio(q.y - pts[p].y, q.x - pts[p].x))
  let (lo, hi) = (s.scan(min), s.scan(max))
  s.keys.count(q => s[q] == lo[q.r] or s[q] == hi[q.r])
  ```
  `lo[q.r]` is E0302 "expected `D.cuts`, found `P.cuts`": the `Prefix<D, T>`
  from `scan` on a slice view keeps `D` unbound. Minimal: `let s =
  a[P.first!.r..]` / `let lo = s.scan(max)` / `lo[s.keys.last!.r]`; the same
  read on the whole array checks.
- **What felt wrong:** a slice keeps its keys (decision 496), so its scan
  should be indexed by the source's cuts; instead the idiom from spec 22
  (`prefix records`) stops at the type checker.
- **Goal it hurts:** express, coherence.
- **Ideal code:** the scan above, or `zip(s, s.scan(max), s.scan(min))`.
- **General fix:** bind a generic `[D]T` parameter's `D` to the slice's
  source domain, so every `[D]T -> X<D>` std function (scan, `Prefix`,
  `suffix_fold`) works on views. Routed to the compiler lane.
- **Severity:** medium (a loop works).

Measurements: spec.cp runs the 8 tests and 60 gen.py cases (n <= 300) equal
to the accepted C++; `cpl run` takes about 30 s at n = 5000 (12.5M exact
ratios), the C++ 0.4 s on the judge.
