# ioi25_festival

Score: 4/5
Dream: 4/5

Scored cold on main against the statement (IOI 2025 Day 2, Festival), `dream.cp` and
`port.cp`; the friction report was read only after scoring.

Dream 4/5. The header (lines 1-5) states the exchange argument and the order; the body
partitions, sorts by an exact `Ratio`, takes the free run with `while let`, and runs a
`[i][K]?Int` DP with `max=` and a walk back. Port 4/5: the same shape, with the header
deriving why only the cheapest 32 of each type past the free run matter. Time 0.04 s against
0.06 s and RSS 1.05x at `--max` (friction report).

## Checklist failures

- The port's DP keeps the cheapest 32 of each type (`port.cp:28-36`); the dream keeps all of
  the rest with K <= 70 (`dream.cp:24-26`). Two different bounds, each derived only in its own
  header ("fewer than 33" against "fewer than 70"), and the gap is not in the friction report
  (porting step 4, R4). Route: the author, to give the dream the port's bound or report why.
- `port.cp:20` caps at `1e15`, `dream.cp:10` at `3e14`, for the same fact ("more than every
  price combined", which is 2e14) (R4).
- `dream.cp:7`: `cpl lint` L0506, `1e9` written twice.

## Owed rewrites

- `port.cp:30-36` mixes two phases in one loop, the free run guarded by `rest.is_empty`; the
  dream's `while let c = left.first and buy(a, c) >= a` (`dream.cp:20-23`) says it, and the
  per-type cut is then a filter over what is left. Route: the author.
- `port.cp:47`, `cost.first_gt(b).map(c => Int(c) - 1) ?? plain.len` is "how many prefix sums
  fit in b" written as index arithmetic (R2). Name it: a count of the prefix sums `<= b`
  (`partition_point`-style), or the `.cuts` form if `scan` keeps its leading 0.
- `port.cp:21-24` reads `coupons[c].p` and `coupons[c].t` eight times; a `type Coupon` with a
  `key` getter (`p * t / (t - 1)` as a `Ratio`) says it once.

## Language gaps

Reported: `read<T>()` storing wide (fixed in the port), `sorted(by:)` buffers. Both are
measured.
