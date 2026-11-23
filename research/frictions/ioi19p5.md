# ioi19p5 — Vision Program

The task is a grader function (`construct_network` building gates through
`add_and`/`add_or`/`add_not`). `cpl run` has no local grader stub yet (filed
with tools from apio23p1), so spec.cp plays the sample grader: it defines
the three gate builders over a gate list, checks the instruction limits,
builds the network once and evaluates it on each image.

## `cpl check` passes a scan that `cpl run` rejects
- **Where:** the old spec.cp's running OR of the diagonal lines:
  ```cp
  let seen = ls.scan((a, b) => add_or([a, b]))
  ```
- **What felt wrong:** `cpl check` called the file clean, and `cpl run`
  stopped at E0302 "expected `Int`, found `(Int, Int) -> Int`" on the
  lambda. A lambda that is not an operator is not a `Monoid`
  (`Monoid<T>.of` falls back to `m.monoid()`), so the run is right and the
  checker misses it when the lambda calls an `extern fn`. Repro:
  `extern fn g(a: Int, b: Int) -> Int` / `@grader` /
  `fn solve(n: Int) -> Int = [1, 2, n].scan((a, b) => g(a, b))[2]`
  (check clean, run E0302); with a plain `fn g` both report E0302.
  `scan` is the wrong tool here anyway: it is exclusive and needs an
  identity, and a gate builder has none. The port writes the inclusive
  running OR as a loop over `seen.last`.
- **Goal it hurts:** doubt-to-evidence (the checker and the interpreter
  must agree on every program).
- **Ideal code:** the check reports the same E0302, naming `Monoid(id, op)`
  as the fix.
- **General fix:** type an `extern fn` call inside a lambda the same way as
  a defined fn's call, so `cpl check` and `cpl run` give one verdict.
  Routed to the compiler lane.
- **Severity:** low (the loop works).

Measurements: on tests/10 (200 x 200, K = 1) spec.cp builds 3,192 gates
with 86,369 inputs (the AC 3,190 / 86,365); on tests/11 (K = 398) 1,606 /
81,611 (the AC 808 / 120,014). Both are within 10,000 gates and 10^6
inputs. `cpl run` takes 0.21 s on tests/11 (48 images).
