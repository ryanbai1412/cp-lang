# Recommendations for decisions 1-4

Status: proposals from the research in NOTES.md, not yet reviewed. The
numbers come from the 3,462 C++ files in `corpus/` and from the benchmarks
in `bench/`.

## 1. Borrow rules (decisions 5.3, 5.4)

**Evidence.** Most closures in the corpus are local helpers: 2,421
`[&]` lambdas in 37% of files, of which only 24 are passed as arguments and
7 are returned. Structs that point into outside data are rare: pointer fields
in 1.3% of files and reference fields in 0.6%.

**Recommendation.**

- **No stored borrows in structs.** A segment tree over `a` or a DFS helper
  over `adj` becomes a top-level fn that reads the file's `let`s, or a
  method that takes the array. Supporting the 2% case would bring back
  lifetimes for everyone.
- **Fields are borrowed separately, indices are not.** `f(mut s.left,
  s.right)` is fine. `f(mut a[i], a[j])` is an error, with `swap` or
  `a.pair_mut(i, j)` as the fix.
- **Two-phase borrows**, so `a.push(a.len)` works.
- **Inline closures can reuse the caller's `mut` borrow.** A lambda passed to
  a parameter marked `inline` is checked as part of the caller's body. That
  makes `undo(mut st): ...` an ordinary library fn.
- **Values lent by a generator** live for one loop iteration. Mutating the
  source inside the loop body is an error.

## 2. Generators (decisions 4.2, 4.4)

**Evidence.** For non-recursive generators, the pull state machine ran as
fast as the hand-written loop (69 vs 74 ms). Recursive generators compiled
push-style matched hand-written recursion (80 vs 81 ms), while an explicit
stack was 8% slower. Zipping two pull generators was 5-10% slower; pushing
the outer one and pulling the inner one matched hand code.

**Recommendation.** The compiler chooses the strategy per use (compiler
chapter 12):

- **Plain `for` loop:** push.
- **`zip`, or storing as `Iterator<T>`:** pull. This changes 4.2: a
  non-recursive generator can be stored.
- **Recursive generator where pull is needed:** an error suggesting
  `List(g)` or a stdlib traversal iterator. There are no hidden explicit
  stacks.

## 3. Domain strictness (decisions 2.8, 2.11)

**Evidence.** `x[i ± 1]` patterns in the corpus:

- **Prefix sums:** 12.2% of files.
- **DP:** 12.2%.
- **Neighbour in the input:** 12.5%.
- **Index through a mapping array** (`tin`, `pos`, `id`): 1.9%.

Each maps to a named operation:

- **Prefix sums** → `V.bounds`.
- **Neighbours** → `.next` / `.prev`.
- **DP over `0..=n`** → a `[V.bounds]` table, or a domain declared over `0..=n`.
- **Mapping arrays** → typed maps like `[V]Pre`.

**Recommendation.** Keep the strict defaults:

- **`Sub<V>` → `V`:** the only implicit conversion.
- **All other conversions:** named methods (`v.before`, `pre.vertex`, `V(x)`).
- **Escape hatch:** `a.raw[i]`, as already agreed.

The next corpus audit should check that the "other" 27% of `±1` uses fit one
of these forms. Any that don't are a gap.

## 4. Runtime checks and overflow (decisions 7.1, 7.2)

**Evidence.** Synthetic kernels, no checks / index checks / overflow checks
/ both, in ms:

- **prefix:** 48 / 50 / 49 / 50
- **segment tree:** 85 / 92 / 88 / 95
- **knapsack:** 68 / 101 / 101 / 126
- **matmul:** 25 / 34 / 27 / 50

Checks are cheap in scalar code and expensive when they block
vectorization.

**Recommendation.**

- **Domain indexing:** never checked; it's proven by the type system.
- **`List` and `a.raw[i]` indexing:** checked in both builds, but checks are
  removed when range analysis proves them and hoisted out of loops where
  possible.
- **Overflow:**
  - **Proven safe:** no check.
  - **Provably reachable:** a warning that shows the bound.
  - **Unknown:** trapped in local builds, unchecked in the judge build.
- **Input validation:** local only.
- **`assert`:** kept in both builds.
- **New: `unchecked:` blocks.** Inside one, the remaining index checks are
  dropped in the judge build. It's greppable and listed by `cp explain`, for
  hot loops where hoisting fails.
