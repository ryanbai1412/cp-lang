# noi07p2

Score: 2/5
Dream: 4/5

Port 2/5, dream 4/5 (the friction's `## Dream`). The DP is the editorial's: the best money so far, each day's purchase as a line in the coupon ratio, and a std `LiChao` for the best line at today's ratio (lines 28-34).

## Checklist failures

- idiomatic-code/R3: `rank` (lines 21-22) inverts the sorted order with a hand loop; std `inverse` gives it (`by_ratio.inverse()`, checked with `cpl run`). Not reported.
- idiomatic-code/R4: the ratio `days[d].a / days[d].b` is written twice (lines 19 and 20). Not reported.
- `lines.at(rank[d]) ?? 0.0` (line 32) uses 0.0 for "no purchase yet"; harmless since `best` starts at `money`, but it reads as a value.

## Owed rewrites

- `fn ratio(d: Day) = days[d].a / days[d].b`, `let rank = by_ratio.inverse()`, and `if let w = lines.at(rank[d]): best max= t.b * w`.

## Language gaps

- `LiChao` keyed by `Int` positions rather than the sorted values: in the friction.

## Dream notes

`LiChao(max, over: by_ratio)` removes both the rank table and the `ratio` list.
