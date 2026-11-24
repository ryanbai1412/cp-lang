# ioi09p7 review

Score: pending
Dream: pending

Cold review: rethought where the old port fell short. It kept each region's preorder positions as
a sorted `List<Int>` and counted a subtree's share with a hand `before` built on `first_true`,
under a WISH for a range count. Now `spots` is a `[Region]SortedSet<t.Pre>`, so a small region's
answer is `sum(spots[r2].range(t.sub(a)).len for a in members[r1])`, memoized with `??=`; the
big-region sweep is cached as `?[Region]Int`. Reads stay typed (`V`, `Region`). At --max it runs
in 1.98 s, against 2.50 s for the old port, with a 7 s limit.
