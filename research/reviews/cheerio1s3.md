# cheerio1s3

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port. The header states the trendline test as steepest/shallowest directions so far, and the body is that sweep. Exact integer cross products replace the dream's `Ratio`, which the friction measures at 30 s in `cpl run`.

## Checklist failures

- Line 111: one comment covers both `top` and `low`, and `?Pt` with `top ?? cur` relies on `cross(cur, cur) == 0` to make "no direction yet" count as a new extreme. The dream says this directly with `is_none_or(h => s >= h)`. `top.is_none_or(t => cross(t, cur) >= 0)` would state it without the trick.
- `pts` (line 107) is `obs` mapped to `Pt`, so the points are written twice. A read straight into `[0..n]Pt` would remove the copy.

## Gaps that need the language

- The friction's `scan` over a slice view (E0302), routed to core.
