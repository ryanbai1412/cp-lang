# Friction report: ioi16p6

spec.cp had no tests. With the two statement samples and eight generated
cases, its `MonotoneHull` DP printed 206 for 207 on tests/03.in, because the
hull does not honor its tie rule; spec.cp now keeps the counts in a `LiChao`
key instead.

## `MonotoneHull` does not give ties to the smaller tag

- **Severity:** high (silent wrong answer)
- **Status:** open (routed to stdlib)
- **Goal it hurts:** hard to get wrong
- **Area:** std (`std/dp.cp` `MonotoneHull`)
- **What happened:** the docs promise "equal values go to the smaller tag
  (the rule `aliens` needs)", but `best_next` steps to the later line on a
  tie, `add` drops a line that is best at a single point, and an equal slope
  keeps the older line whatever its tag. `aliens` then searches with a count
  that is not the least, and the answer is off by one photo's fee.
- **Repro:** `var h = MonotoneHull<Int>(min); h.add(Line(1, 0), tag: 3);
  h.add(Line(-1, 2), tag: 7); print(h.best_next(1)!.tag)` prints 7, not 3.
- **Workaround:** `LiChao<Photo, (val: Int, count: Int)>` with a line struct
  whose `at` returns `(val, count)`, the tie mechanism `LiChao` documents.
  It is O(log m) per operation and about 15x slower than the hull in
  `cpl run` (2.9 s vs 0.2 s on tests/07.in).
- **General fix:** one tie mechanism for every line structure: drop `tag`
  and let `MonotoneHull` take lines whose `at` returns any ordered `K`, as
  `LiChao` does, so a `(val, count)` key breaks ties by comparison. Then
  spec.cp can go back to the O(1) hull.
