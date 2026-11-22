# slope doc notes

- `Concave`, `Convex`: contradiction. `spec/lib/search-dp.md` §5 lists `SlopeTrick` (std.slope) as "piecewise-linear convex DP"; the stub and decision 81 have `Concave` and `Convex` over finite integer domains. Followed the stub.
- `Concave.flat`, `Convex.flat`: chose a trap on an empty `r` (a function needs at least one point) and documented that `r` may be written half-open or with `..=`.
- `Concave.at`, `max`, `Convex.at`, `min`: the stub states two costs ("with subtree sums" and "today"). Kept both, marking the second as the current implementation's; the spec should state one.
- `Concave.add_quadratic`, `Convex.add_quadratic`: which points `q` is read at is unstated; documented only "at most four calls" and a trap when the readings are not quadratic or the leading coefficient has the wrong sign (decision 81).
- `Concave.slide_max`, `Convex.slide_min`: chose a trap on `a > b` and documented the new domain `lo + a ..= hi + b` (the stub says only the formula).
- `Concave.argmax`, `Convex.argmin`: chose a half-open `Range<Int>` covering every optimal point.
- `Concave.prefix_max`, `suffix_max`, `Convex.prefix_min`, `suffix_min`: chose that the domain is unchanged (the max/min ranges over points of the domain only).
- `Concave.reserve`: chose a trap on `n < 0`. `Convex` has no `reserve`; asymmetry left as is.
- `Concave.max_convolve`, `Convex.min_convolve`: chose the domain `lo + g.lo ..= hi + g.hi` and that `g` is consumed into `self`.
