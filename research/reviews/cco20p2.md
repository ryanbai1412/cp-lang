# cco20p2

Score: 2/5
Dream: 4/5

Days from the last: push the exercises due that day into a `MaxHeap`, take the furthest right, and count inversions with a `Fenwick`; failure is `?Int`. All of it is the editorial except the bucketing, which the language already does.

## Checklist failures

- idiomatic-code/R3: `var due_on: [Day]List<Ex> = []` and its push loop (lines 16-17) hand-roll std `buckets` (`[D]K -> [K]List<D>`): `let due_on = due.buckets()` on `due: [Ex]Day` runs under 0.2. The friction reports this as missing.

## Owed rewrites

- `let due_on = due.buckets()`; drop the friction section.

## Language gaps

None open.

## Dream notes

- The dream is the port with the bucketing named and `free.pop()?` for the failure: a 4.
