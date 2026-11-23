# cco20p4 friction

## The road table can't be read as `[Pair<V>]Char`

- **Where:** `spec.cp` reads `rows = read<[n - 1]Str>()` and `is_red(u, v)` picks the row of
  the later town by hand (`rows[hi.ord - 1][lo.ord]`).
- **What felt wrong:** the input is exactly a lower triangle, which `std/pair.cp` documents as
  `read<[Pair<V>]T>()` with `road[u, v] == road[v, u]`; today that is E0204 "an array is
  indexed by a domain or a length".
- **Goal it hurts:** express (one fact, the symmetric road colour, needs index bookkeeping).
- **Ideal code:** `let road = read<[Pair<V>]Char>()` then `road[u, v] == 'R'`.
- **Repro:** `domain V = 0..4` / `let a: [Pair<V>]Int = read()` -> E0204 (also
  `spec/conformance/matrix/index/pair--read.cp`).
- **General fix:** already owned: `Domain` as a trait any type implements, arrays packed by
  `ord` (compiler `PLAN.md`, `std/GAPS.md` pair entries).
- **Severity:** low (a two-line accessor).

## The corpus tests took a partial-score reference

- **Where:** `scripts/corpus-tests.py` `accepted_cpp` takes the first submission marked `AC`,
  here 4801875, which prints the 2n - 2 Euler walk; `checker.py` (route length n) rejects it
  on every test. The other three AC submissions print length n.
- **Fix here:** `tests/*.ans` are now 6416816's answers, and `spec.cp` follows its insertion
  order so the exact token compare passes.
- **General fix:** let `meta.json` pin the reference submission (a hand-written field the
  generator keeps), and have `make`/`verify` run `checker.py` on the reference answers.
- **Severity:** medium (a test set that rewards a wrong answer).
