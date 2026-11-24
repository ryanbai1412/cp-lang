# ioi09p7 review

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: rethought where the old port fell short. It kept each region's preorder positions as
a sorted `List<Int>` and counted a subtree's share with a hand `before` built on `first_true`,
under a WISH for a range count. Now `spots` is a `[Region]SortedSet<t.Pre>`, so a small region's
answer is `sum(spots[r2].range(t.sub(a)).len for a in members[r1])`, memoized with `??=`; the
big-region sweep is cached as `?[Region]Int`. Reads stay typed (`V`, `Region`). At --max it runs
in 1.98 s, against 2.50 s for the old port, with a 7 s limit.

## Checklist failures

- Lines 4-10 read the supervisors one row at a time into `var home` and `var boss`, seeded with a placeholder `head` for every vertex. One typed read of the rows for `V.all.drop(1)` would give both arrays as values.
- `members[r1].len * members[r1].len > n` is the sqrt split between the sweep and the per-member count, written inline with no name. `const`-like naming (`heavy(r)`) would say which side is which.
- `members` and `spots` are filled by one loop over `V` into two parallel per-region tables. `spots` alone (`SortedSet` of positions) can stand in for `members` everywhere but the length test.

## Gaps that need the language

- Reading the supervisor rows as typed values: research/frictions/ioi09p7.md.
