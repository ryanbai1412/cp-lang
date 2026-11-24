# ioi25_festival

Score: 4/5
Dream: 4/5

Re-reviewed cold on fbed8d10d against the statement (IOI 2025 Day 2, Festival), `dream.cp` and
`port.cp`; the friction report was read only after scoring.

Dream 4/5. The header states the exchange argument and the doubling bound; the body partitions,
sorts by an exact `Ratio`, takes the free run with `while let`, runs a `[i][K]?Int` DP with
`max=`, and walks back. `LIMIT` and `cap` are each written once, with the cap derived. Port 4/5:
the same shape, with the free run in its own loop and the per-type cut as a filter over what is
left. Both now use 32 and `2e14`, and the friction report records the unification. At `--max`
the port takes 0.04-0.09 s against 0.06-0.13 s, at 0.91x orig.cpp's RSS.

## Checklist failures

None. The first review's two bounds, two caps and the unreported gap are fixed and reported
(friction "The port and the dream had two DP bounds and two caps").

## Owed rewrites

- `port.cp:21-26, 37, 46, 51`: `coupons[c].p` and `coupons[c].t` are read seven times, and the
  sort key `Ratio(coupons[c].p * coupons[c].t, coupons[c].t - 1)` is written inline. A
  `struct Coupon(p: 1..=1e9, t: 1..=4)` with a `@get fn key` says the key once. The dream's
  `(p, t)` tables are fine. Route: the author (minor).

## Language gaps

Reported: `read<T>()` storing wide, `sorted(by:)` buffers (fixed by 333c96870), the `Prefix` count
of sums `<= b` (`first_true` stands in), and `cpl fmt` splitting `buy(t, c)` at `port.cp:46-49`.

## Dream notes

- The port keeps only the cheapest 32 of each type past the free run; the dream keeps all of
  them and bounds the count by 32 instead. Both are correct and the port's header says why the
  cut is safe. The dream could state the cut too, so the two read the same.

Addressed (minor): `port.cp` has `struct Coupon(p: 1..=1e9, t: 1..=4)`. Its `@get fn key` gives the
exchange order once, and `fn buy(self, a)` gives the token update once. `coupons: [C]Coupon` keeps the
narrow layout (`cpl explain`: 8 B per element, 1.6 MB at n = 2e5).
