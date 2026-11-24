# noi17p2

Score: 3/5
Dream: 4/5

Cold review on main c8a46b201: port, dream and friction report, not the C++. The dream keeps one online `Counter` of every run; the port stores only the asked windows for speed, which the friction report ("Query windows only") explains and measures at 1.06-1.08x the C++.

## Checklist failures

- Lines 43-53 and 89-91 thread one `at_ask` counter by hand through three passes over `ops`, reset on line 53, to pair each `Ask` with its windows (idiomatic-code/R7).
- `reach` (lines 61-70) picks the concatenation order with a `back: Bool` flag, and its `while ...: if let w = at: ... else: break` is a `while let` with a length guard.
- Line 19 builds a `RollingHash61` over 50 zeros only to borrow its base, and line 22 builds the empty block from it (reported, `combine` section).

## Owed rewrites

- Keep the asked windows beside their op (e.g. `asked: [q]?List<Hash61>` filled by `ops.items`), so the second pass and the ask branch index them by position, not by a shared counter.

## Language gaps

- The length-carrying hash with `+` and an empty identity: friction 1.
- `combine` needing a pre-sized power table: reported.

## Dream notes

- Line 9 wraps the base in `Some(...)` for no reason the header states.
- The dream's header does not bound the join cost (about 50^2 / 2 crossings per join).
