# ioi15p6 friction

## `majority(eq:)` spends 2n comparisons, past the query limit

- **Where:** `corpus/problems/ioi15p6/spec.cp` wants
  `T.filter(i => at[i].along == h).majority(eq: same)`, where `same` asks
  `getDistance`. std `majority` is two Boyer-Moore passes, up to 2n - 2 `eq`
  calls; with the 2n queries spent finding the diameter, subtask 6's
  `ceil(7N/2)` limit breaks (gen.py seeds 11, 71, 77: query 351 past 350,
  386 past 385). The spec hand-writes Fischer-Salzberg (at most 3n/2 calls).
- **Repro:** `var calls = 0`, `fn eq(a: Int, b: Int) -> Bool: calls += 1; a == b`,
  then `[1, 2, 1, 2, 1, 2, 1, 1].majority(eq: eq)` gives count 5 after 12 calls;
  Fischer-Salzberg needs at most 10.
- **Goal it hurt:** express (a standard algorithm hand-rolled) and costs-are-visible
  (the doc's cost line says nothing about `eq` calls).
- **General fix:** std `majority` as Fischer-Salzberg, with the `eq` call
  bound in its `cost:` line.
- **Routed:** stdlib worklog.

## A `@grader fn` has no local stub

- **Where:** `extern fn getDistance` traps in `cpl run` ("the judge's grader
  supplies it"), so the spec reads the sample grader's input and answers
  `getDistance` from the distance matrix, counting queries against the limit.
- **Routed:** already in the tools worklog (from apio23p1).
