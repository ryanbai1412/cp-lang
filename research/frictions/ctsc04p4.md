# Friction report: ctsc04p4

The port is the dream except for one helper: the answer is a minimum over every switch's
last step, and a `for` can't walk over an optional, so `finish(v)` unwraps it.

## A generator can't draw from an optional

- **Where:** `fn finish(v: Switch) -> (Int, Int)` and `min(finish(v) for v in Switch)!`
- **What felt wrong:** `root_plan[v].last` is a `?T`; the natural comprehension
  `for s in root_plan[v].last` is rejected, so a three-line helper restates it.
- **Goal it hurts:** express
- **Ideal code:** `min((s.cost + cost[v], s.cores + 1) for v in Switch for s in root_plan[v].last)!`
- **Repro (fresh build):**
  ```
  let a: List<?Int> = [Some(1), None]
  print(min(x for o in a for x in o)!)
  ```
  `error[E0302]: a for can't walk over a ?Int`
- **General fix:** check: an optional is a sequence of zero or one value in `for`, as in Rust's
  `IntoIterator for Option`.
- **Severity:** low

## Measurements

gen.py --max (n = 400, p = n, path-heavy tree): port 0.43 s, accepted C++ 0.61 s.

## Dream

The port as written, with `finish` inlined as in "Ideal code" above.
