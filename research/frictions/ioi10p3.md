# ioi10p3

The port is the dream: every step of the solution has a direct spelling today (typed bounds, domains, std structures), so there is no gap to report.

## Dream

The port as written. `first_true` over the answer, and each check is one `Prefix` built from the table `[Row, Col] (i, j) => Int(rank[i, j] <= x)` tested over every `windows(h)` by `windows(w)` pair.
