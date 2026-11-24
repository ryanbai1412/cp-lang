# Friction report: ioi25_souvenirs

## Dream

```cp
while T.any(i => price[i] == None):
    for i in T.rev:
        if let v = price[i]:
            if T.after(i).any(j => price[j] == None): break pay(v - 1)
        elif let e = eq[i]:
            break if e.types.len == 1: learn(i, e.sum) else: pay(e.sum / e.types.len)
```

The port is the dream: one equation per smallest unknown type, scanning from the right. It
replaced a port of the official model solution (leads walked forward, then peeled from the
right) whose `lead_cost` used `-1` for "unknown". Both are stdin programs that simulate the
sample grader's `transaction`.

## `break` takes no value, so a last call can't ride on it
- **Where:** `dream.cp`, the scan's two exits, which now read
  `if T.after(i).any(j => price[j] == None): pay(v - 1); break` and
  `if e.types.len == 1: learn(i, e.sum) else: pay(e.sum / e.types.len)` / `break`.
  The dream's form fails on a fresh `cpl`:
  `error[E0107]: expected the end of the statement, found a name` at `break pay(i)`.
  The grammar has `'break' LABEL?` only (spec/core/01-lexical-grammar.md).
- **What felt wrong:** "do this, then stop scanning" is one step. Written as two, it puts an
  if/else and its `break` on separate lines, and a revision that drops the `break` still checks
  and loops wrongly.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** `break pay(v - 1)` and `break if c: a() else: b()`, where a unit expression
  is evaluated and then the loop ends.
- **General fix (compiler/spec):** let `break` take a unit-typed expression, which is
  evaluated before the loop exits, as `return f()` already does in a unit fn. This is the
  same rule as `return expr`, and it helps any scan that acts once and stops (first-fit,
  "find and apply").
- **Severity:** low.

## No in-place loop over the present entries of `[T]?E`
- **Where:** `dream.cp` `learn`, now
  `for k in T if eq[k] is Some(e) and i in e.types:` / `let e = mut eq[k]!`. The dream's
  `for e in eq.values_mut() if i in e.types:` fails with
  ``error[E0203]: a `[3]?Eq` has no method `values_mut` `` and then
  ``error[E0401]: `e` is a loop variable, so it can't be assigned``.
- **What felt wrong:** to update every present equation that mentions `i` in place, you loop
  over positions, test with `is Some`, then take a second `mut` borrow of the same slot, which
  is the index written twice.
- **Goal it hurts:** express, coherence.
- **Ideal code:** `for e in eq.values_mut() if i in e.types: e.types.remove(i); e.sum -= v`,
  or a `for mut e in eq.present()` spelling, whatever std settles on for mutable element
  views.
- **General fix (stdlib):** a mutable view over an array's elements, which skips `None` for
  `[T]?E`, as a std method like `Map.values_mut`. It helps any "patch every live slot" loop
  (sparse tables of equations, pending queries, per-node optional state).
- **Severity:** medium.

## An interactive `@grader` dream can't be verified in a package
- **Where:** the dream as first written, `@grader(interactive) fn buy_souvenirs(n: 2..=100,
  p0: 1..=1e15)` calling the judge's `transaction`. `scripts/corpus.py verify` runs
  `cpl test dream.cp --tests <tmp>`, and `cpl test` looks for `grader.cpp` only through
  `find_tests_dir(source)` (crates/cpl-contest/src/problem.rs), not through the `--tests`
  folder. In a package with `tests.toml`, it finds the package's `grader.cpp` only after
  `corpus.py make` has written `target/corpus-tests/<id>/`, and otherwise reports
  `cpl: dream.cp links only with the judge's grader ... pass --grader grader.cpp, or put
  grader.cpp beside its tests`. No package commits a `grader.cpp`, and `corpus.py` passes no
  `--grader`. So the dream became a stdin program that simulates `transaction` itself, as
  ioi14p1's port does.
- **What felt wrong:** the program a contestant submits (the `@grader` fns) is not the
  program the corpus verifies. Each grader task copies the sample grader into CP Lang, and the
  dream carries a second copy, so the judge's interface is never checked against the
  reference.
- **Goal it hurts:** evidence, submission.
- **Ideal code:** the dream exactly as the judge takes it, `@grader fn
  transaction(m: 1..=1e15) -> (List<0..100>, 0..=1e15)` and `@grader fn
  buy_souvenirs(n, p0)`, verified by linking the package's committed sample grader.
- **General fix (tools):** a package may commit `grader.cpp` (the official sample grader plus
  its header). `corpus.py` then passes `--grader <pkg>/grader.cpp` to `cpl test` for the
  dream and port, and builds `orig.cpp` against the same file. Alternatively, `cpl test
  --tests DIR` also looks for `grader.cpp` beside the package's `tests.toml`. This covers
  every IOI function task in the corpus (all six ioi25_*, ioi14p1, ioi16p6, ioi18p1, ...).
- **Severity:** medium.

## Measurements

- **Tokens:** solution only, grader simulation excluded: port and dream 352 (the model-solution port was 299), original
  (author_solution.cpp) 410.
- **Runtime and memory:** on the `--max` case (N = 100), port 0.00 s / 3.7 MB, accepted C++
  0.00 s / 3.7 MB.
- **Emitted C++:** port and dream 20778 bytes (the model-solution port was 15835).
- **Agreement:** the port and dream match `orig.cpp` on all 54 official tests and on 2000 gen.py
  seeds; the port again on 30 seeds and its 12 package cases.
