# noip06p2

Score: 3/5
Dream: 4/5

Cold review: the main-plus-accessories groups and the grouped 0/1 knapsack are the header's, but the accessory subsets are a push-while-reading-length loop and the group update is three nested loops; neither was reported.

## Checklist failures

- R3: lines 28-32 build the bundles by pushing onto `bundles` while walking its first `plain` entries, the C++ subset doubling.
- R3: the extras are grouped by a push loop (lines 21-23); std `group(by:)` gives the buckets as a `Map`, though not as a `[Item]List`.

## Owed rewrites

- None the language allows yet beyond the grouping.

## Language gaps

- The 0 owner sentinel read as `0..=n` then `Item.at` (line 18): in research/frictions/noip06p2.md, now with the `cpl run` errors.
- Subsets of a `List` and a grouped knapsack step: filed in research/frictions/noip06p2.md.

## Dream notes

- None.
