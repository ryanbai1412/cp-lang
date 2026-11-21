# Writable-from-docs evaluation

Measures the `AGENTS.md` design rule *"Writable from the docs: a model or a person should write correct code from the documentation alone."*

## Harness

- Script: `scripts/writable-from-docs.py` (offline tests in `scripts/test_writable_from_docs.py`).
- Pinned manifest (`research/writable-from-docs-set.json`, v2): **55 unseen unique deterministic-output mixed-difficulty problems** (34 Codeforces Div2 B–E, 17 AtCoder ABC D–E, 4 AtCoder ARC A–B) with zero overlap with `proto/ports/`, excluding interactive problems, multi-answer constructive problems, and redirected duplicate CF statements.
- Sample comparator supports `1e-6` relative/absolute float tolerance (matching online judges).
- Ratchet (`research/writable-from-docs.json`): enforced offline in `scripts/scoreboard.py --fast --check` and refreshed weekly by `.github/workflows/writable-from-docs.yml`.

## Iteration curve (55 unseen problems, `gpt-5.4`)

| Round | Docs / harness delta | Compile (`cpl check` + `cpl build`) | Full pass (all samples) |
| --- | --- | --- | --- |
| **0 (baseline)** | Initial 8-file bundle (v1 set) | 40/55 (72.7%) | 14/55 (25.4%) |
| **1** | Added `idioms.md` + upgraded `cpp-guide.md` + 6 R0 fixes | 46/55 (83.6%) | 19/55 (34.5%) |
| **2** | Fixed `Dsu<V>` & `v.next`; caught `if c then` typo | 40/55 (72.7%) | 18/55 (32.7%) |
| **3** | Fixed `if cond: a else: b` & `arr.swap(i, j)` | 38/55 (69.1%) | 17/55 (30.9%) |
| **4** | Fixed `Sieve.spf[x]` & `HashMap` `m[k]` vs `m.get(k)` | 46/55 (83.6%) | 17/55 (30.9%) |
| **5** | Cleaned 15 un-evaluable v1 problems (2 interactive, 5 CF redirect dups, 8 multi-answer) + `1e-6` float tolerance + `a.reverse()` docs | 46/55 (83.6%) | 25/55 (45.5%) |
| **6** | Added `v + 1`/`V.last - 1` `?V` docs + `mut` call-site docs; routed 3 language/stdlib frictions | 48/55 (87.3%) | 29/55 (52.7%) |
| **7** | Enforced canonical spellings across all docs & site examples (`if` filters, `**`, parenless getters, `@!multitest`, method-form folds, `t.Pre`); exposed `Mod<P>.v` vs `.raw` & lent `Str` slices | 44/55 (80.0%) | 25/55 (45.5%) |
| **8** | Fixed `Mod<P>.v`, lent `s[l..r].to_str()`, `[R, C]T`, `Yes`/`No`, no trailing `=`/`=>`; enabled `reasoning_effort="medium"` (16k max completion tokens) | 44/55 (80.0%) | 43/55 (78.2%) |
| **9** | Fixed `'a' <= c and c <= 'z'` (ranges are `Int`-only), multi-line `(...)` parens, `(m[k] ??= init)` parens, `mut` params vs local copy, `let` for `[domain S]Char`, and `300s` HTTP timeout | **52/55 (94.5%)** | **48/55 (87.3%)** |
| **10 (current)** | Fixed `isqrt` prelude vs `std.math` (`cf2020b`, `cf2035e`), `[R, C]T = 0` vs `[R][C]T` (`cf2020e`), unnamed `(Row, Col)` for `.adj4` (`abc384e`), `Bitset.of(xs)`/`Bitset.from(a)` & `course-dp.md` methods, `Range.halves()` (`cf2069f`), and SSE streaming + 24k completion budget (`cf2013e`) | **51/55 (92.7%)** | **50/55 (90.9%)** |

## Failure classification (Round 10: 5 failing / 55 total)

- **Model syntax / line-break slip:** 2
  - `cf2036c` (`E0102`: unwrapped multi-line `and` at end of line).
  - `cf2036d` (`E0103`: unwrapped multi-line `if` condition).
- **Model domain / `mut` slip (caught by typechecker):** 2
  - `cf2035d` (`E0402`: omitted `mut` at call site `split(a[v])`).
  - `abc384e` (`E0301`: indexed `[Row, Col]` with domain `P` instead of `Row`/`Col`).
- **Model algorithm wrong:** 1
  - `cf2040d` (compiled; sample 1 WA on constructive tree labeling).

## Failure classification (Round 9: 7 failing / 55 total)

- **Docs gaps (fixed post-Round 9):** 2
  - `abc380d` (compiled, WA: `[D]Char` printed contiguously as `aBAb...` instead of spaced `a B A b ...`; documented `print(chars, sep: " ")` in `io-and-formatting.md` and `idioms.md`).
  - `abc385d` (compiled, `R0001`: `m[k]` trapped on absent key because `stdlib.md` contradicted `one-page-cheat-sheet.md` by saying `m[k]` auto-inserted; unified `stdlib.md` and `idioms.md` to match runtime `R0001`).
- **Model syntax / output slip:** 3
  - `cf2004b` (`E0102`: trailing `=` at end of line).
  - `cf2013e`, `cf2040d` (exhausted completion budget in reasoning before emitting fenced `cp` block).
- **Model algorithm wrong:** 2
  - `cf2004d` (sample 1 WA on portal shortest path).
  - `abc382e` (sample 2 WA on expected packs DP).

## Historical failure classification (Round 6: 26 failing / 55 total)

- **Docs gap:** 1 (`cf2035d`: `Mod<P>.val` vs `.raw`)
- **Language / stdlib gap:** 2 (`cf2007e`, `abc381e`: multi-statement `match =>` arm `E0102`; plus `bsearch_last` `?Int` inference from Round 5 — all routed in `research/frictions/writable-from-docs-gaps.md`)
- **Model's algorithm / domain slip:** 23 (17 runtime WAs on compiled programs, 2 runtime off-by-one bounds traps, 4 one-off domain/type slips)

| Problem | Compile | Classification | Evidence |
| --- | --- | --- | --- |
| `cf2003b` | ok | model's algorithm wrong | sample 1 WA (expected '2\n1\n2\n2\n7', got '1\n1\n2\n2\n5') |
| `cf2004e` | FAIL | model's algorithm wrong | Compile/domain slip (`E0203`): error[E0203]: a `Int` has no member `to_idx` that `cpl run` knows |
| `cf2005d` | FAIL | model's algorithm wrong | Compile/domain slip (`E0302,E0309`): error[E0309]: `+` can't be applied to `Int` and `V` |
| `cf2007e` | FAIL | language/stdlib gap | Multi-statement `match =>` arm (`E0102`); routed in `research/frictions/writable-from-docs-gaps.md` #2. |
| `cf2013b` | ok | model's algorithm wrong | sample 1 WA (expected '-1\n8\n2\n7\n8', got '-1\n8\n8\n13\n12') |
| `cf2013d` | ok | model's algorithm wrong | sample 1 WA (expected '0\n2\n1\n1\n3', got '1\n2\n4\n4\n10') |
| `cf2013e` | ok | model's algorithm wrong | sample 1 WA (expected '6\n6\n9\n14\n51', got '6\n6\n9\n16\n46') |
| `cf2020d` | ok | model's algorithm wrong | sample 1 WA (expected '2\n96\n61', got '2\n86\n60') |
| `cf2025d` | ok | model's algorithm wrong | sample 1 runtime error: error[R0001]: index 5 is outside an array dimension of length 5 |
| `cf2025e` | ok | model's algorithm wrong | sample 1 WA (expected '2', got '16') |
| `cf2035b` | ok | model's algorithm wrong | sample 1 WA (expected '-1\n66\n-1\n3366\n36366\n3336366', got '-1\n666\n-1\n36366\n-1\n-1') |
| `cf2035d` | FAIL | docs gap | Used `.val` on `Mod<P>` (`E0203`); `stdlib-api.html` documents `.raw`. |
| `cf2035e` | ok | model's algorithm wrong | sample 1 WA (expected '12\n190\n280\n160', got '13\n260\n280\n2404') |
| `cf2039d` | ok | model's algorithm wrong | sample 1 WA (expected '6 4 4 3 4 3\n1\n-1', got '4 4 4 6 6 6\n1\n-1') |
| `cf2040b` | ok | model's algorithm wrong | sample 1 WA (expected '1\n2\n2\n4', got '1\n2\n3\n5') |
| `cf2040c` | ok | model's algorithm wrong | sample 1 WA (expected '1 3 2\n2 3 1\n-1\n2 4 3 1\n-1\n2 3 4 5 7 6 1', got '1 3 2\n3 1 2\n-1\n4 1 3 2 |
| `cf2040d` | ok | model's algorithm wrong | sample 1 WA (expected '2 10 1 6 5\n8 7 12 1 4 6 3', got '2 1 4 6 3\n2 1 3 4 6 8 10') |
| `abc381e` | FAIL | language/stdlib gap | Multi-statement `match =>` arm (`E0102`); routed in `research/frictions/writable-from-docs-gaps.md` #2. |
| `abc382e` | ok | model's algorithm wrong | sample 1 runtime error: error[R0001]: index 2 is outside an array dimension of length 2 |
| `abc386d` | ok | model's algorithm wrong | sample 1 WA (expected 'Yes', got 'No') |
| `abc388d` | FAIL | model's algorithm wrong | Compile/domain slip (`E0301`): error[E0301]: `diff` is indexed by 1..=n + 2, but `i` is a V |
| `abc388e` | FAIL | model's algorithm wrong | Compile/domain slip (`E0301`): error[E0301]: `a` is indexed by I, but `t` is a 0..k |
| `arc180a` | ok | model's algorithm wrong | sample 1 WA (expected '2', got '3') |
| `arc181a` | ok | model's algorithm wrong | sample 1 WA (expected '1\n0\n2', got '2\n0\n2') |
| `arc181b` | ok | model's algorithm wrong | sample 1 WA (expected 'Yes\nNo\nNo', got 'Yes\nYes\nYes') |
| `arc182a` | ok | model's algorithm wrong | sample 1 WA (expected '1', got '4') |
