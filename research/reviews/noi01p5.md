# noi01p5

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`). Meet in the middle as the header says: the left half's sums go into a `Counter` as they are made (lines 20-28) and the right half's are looked up one at a time (lines 31-35). At `--max` the built port takes 0.29 s and 42 MB against the 1000 ms / 64 MB limit.

## Checklist failures

- idiomatic-code/R3: `count_left` and `matches` (lines 23-34) hand-roll the enumeration of every sum of one value per term, twice, with an `at` cursor. The friction's dream names it `sums(left)`; the gap was not reported (now in the friction).
- The friction described a sorted-halves rewrite with `equal_range` that the port no longer is (it uses a `Counter`), with that version's 0.72 s / 88 MB. Corrected in the friction with this port's measurement.

## Owed rewrites

- `T.collect().partition(t => Int(t) < n / 2)` (line 19) converts the domain to `Int` to split it; a range split of `T` at `n / 2` says it.

## Language gaps

- No std enumeration of the sums over a product of lists (now in the friction).

## Dream notes

`count_pairs(sums(left), sums(right), (x, y) => x + y == 0)` names the algorithm, but a predicate join hides the hash lookup and suggests both halves are materialised; `sums(right).sum(s => low[-s])` over a counted left half states the memory bound the header relies on.
