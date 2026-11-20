# Scout batch 50: hard NOI and NOIP classics, part 2

Ten accepted C++ submissions from the `noi*` / `noip*` prefix bucket that had
no port anywhere under `proto/ports/`, re-solved against the locked sheet
(`docs/one-page-cheat-sheet.md`, which is at v2.1 on `main`).

```
noip02p4   5096459.cpp   cover n points with at most 3 disjoint rectangles (recursive cuts)
noi03p4    5032732.cpp   weighted tree: diameter ends plus the best third node
noi04p5    3744270.cpp   rod-cutting DP on one-decimal prices, done exactly in tenths
noip08p4   5185830.cpp   twin-stack sorting: clash graph, 2-colouring, least script
noi17p1    5690633.cpp   huge integer under x += a * 2^b with bit queries
noi20p5    3803796.cpp   "surreal trees": a 4-way trie over tree spines
noi22p1    4800882.cpp   strict majority over joined sequences (push, pop, join)
noi22p4    4802121.cpp   rooted tree isomorphism after at most 5 leaf deletions
noi23p1    5723056.cpp   count cells under horizontal, vertical and diagonal strokes
noip22p2   5445681.cpp   constructive: clear paired cards through n stacks
```

## Layout

- `<problem>.cp` is the v2.1-spec port: what the solution should read like.
  These files are **not compiled**; the prototype does not implement all of
  the sheet.
- `today/<problem>.cp` is the same algorithm in the subset `proto/cpc` builds
  now. These are the files that are verified.
- `check.py` builds every `today/` file (on a copy), compiles the emitted
  C++ with ASan/UBSan and diffs it against the original submission.
- `loc.py` prints the dual LOC table.

```
python3 proto/ports/scout_batch_50/check.py            # all ten, 150 cases each
python3 proto/ports/scout_batch_50/check.py noi17p1 --cases 40
python3 proto/ports/scout_batch_50/loc.py
```

## What the check covers

Last run: 1500 cases pass (150 per problem).

Inputs are random and small to medium (roughly two thirds small, one third
medium; the sizes are in each `gen_*` function). Besides the diff against the
original, seven problems are also checked against an independent exhaustive
solver in `check.py` whenever the input is small enough: `noip02p4`,
`noi03p4`, `noip08p4`, `noi17p1`, `noi22p1`, `noi22p4`, `noi23p1` (`noi17p1` and
`noi22p1` on every case).

Three problems are not a plain diff:

- `noip08p4`: the original's script is legal but not always the
  lexicographically least one. On `5 / 3 2 4 1 5` it prints
  `a a c a b b b d a b`; `a a c a b b b a d b` is smaller and also sorts.
  The check therefore requires the two programs to agree on "0 or not",
  validates the port's script by simulation, and holds the port (not the
  original) to the exhaustive least script.
- `noip22p2`: any valid script is accepted, so both outputs go through a
  simulator that checks every card is dealt, every stack ends empty and the
  operation count is within `m..=2m`.
- `noi22p1`: the original answers by 20 random probes; the port is exact
  (per-bit majority plus one count lookup). They agreed on every case.

## Not tested

- Nothing at full constraints: no time or memory measurement, and the
  prototype warns that the declared loop nests of `noi04p5`, `noi20p5`,
  `noi22p1` and `noip22p2` are large at the maximum bounds.
- `noi04p5` and `noi20p5` have no brute force, only the diff. `noi20p5`
  trees have at most 9 nodes, `noi04p5` prices at most 50.9.
- `noi03p4` inputs are always trees (`m = n - 1`), as in the real data.
- `noi22p4` keeps `|G| - |H| <= k`: the original ignores `k` and uses a fixed
  5, which is equivalent only under that guarantee.
- `noip22p2` uses `k = 2n - 2` or `2n - 1` with `n <= 12`, and decks where
  every value appears an even number of times.
- `noi17p1` keeps the number non-negative, as the statement promises.
- The v2.1 sketches themselves are unverified by construction.
