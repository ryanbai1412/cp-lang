# cmt21r2ztr friction

spec.cp was stale (a `flat_map`/`map` compress key, `TwoSat<(P, Side)>`, direct
`P.adjacent`) and the problem had no tests. It now checks clean, carries 10 tests
(2 hand-made from the statement's shape, 8 from `gen.py`, answers from the AC
submission 4007960) and a `checker.py`, and every output it prints is accepted by
`checker.py`. The suite still reads `wrong` because the answer is not unique.

## The corpus suite ignores the problem's checker

- **Where:** `cpl-conform corpus cmt21r2ztr` reads `wrong: 04.in`. Any array whose
  medians match is accepted; spec.cp's 2-SAT assignment differs from the AC C++'s, so
  4 of 10 tests (04, 05, 07, 09) print a different valid array. `checker.py` accepts
  spec.cp on all 10 and on 60/60 generated inputs.
- **Repro:** `3 1 / 1 2 3 5` has answers `YES 5 5 5` and `YES 1 5 6`; the suite accepts
  only the reference's.
- **General fix:** run `checker.py input output answer` when it exists. Already
  routed to spec (btoi15p3, aac5p4); not filed again.
- **Status:** open.

## `.adjacent` of a domain is E0501 in `cpl run`

- **Where:** `for (p, q) in P.adjacent if ..` over the compressed pins. spec/core/04
  says a domain is a `Seq<V>` of its positions, and `adjacent` is a `Seq` method.
- **Repro:** `domain P = 0..4` / `for (a, b) in P.adjacent:` / `    print(a, b)` is
  E0501 "`cpl run` and `cpl build` can't handle `.adjacent` of a domain as a value yet".
- **Workaround:** `[p for p in P].adjacent` (allocates a list of positions).
- **General fix:** lower a domain's `Seq` getters as for any other `Seq`.
  Routed to compiler.
- **Status:** open.

## A product domain of a domain and an enum is not its tuple

- `TwoSat<(P, Side)>` with `at_most_one([Lit((p, s), true) for ..])` is E0302
  "expected `List<Lit<Prod3>>`, found `List<Lit<(P, Side)>>`". Workaround: name it,
  `domain Var = (P, Side)`, and write `Var((p, s))` and `value[Var((p, Above))]`.
  Already in the compiler worklog (2026-11-22, corpus migration); not filed again.
