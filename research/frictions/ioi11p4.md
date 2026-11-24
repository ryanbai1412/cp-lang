# ioi11p4

Every step of the solution has a direct spelling today (typed bounds, domains, std structures); the one gap is the judge's function form.

## The judge's function task has no CP Lang form
- **Where:** the port reads `n m k`, the corridors and the exits from stdin; the accepted C++
  (`corpus/submissions/3734919.cpp`) is `travel_plan(n, m, edges[][2], weights[], k, escapes[])`
  called by a sample grader. A `@grader` fn can't take the judge's C arrays (the same E0501 as
  ioi14p1).
- **Dream:** `@grader fn travel_plan(n: 1..=1e5, m: 1..=1e6, edges: [m](Room, Room), weights: [m]1..=1e9, k: 1..=1e5, escapes: [k]Room) -> Int`
- **General fix:** grader fns over C arrays, with lengths from other params.
- **Owner:** core.
- **Severity:** high (the port can't be submitted to the judge as written).
