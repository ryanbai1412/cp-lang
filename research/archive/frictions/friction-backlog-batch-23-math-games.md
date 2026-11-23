# Friction Backlog — Scout Batch 23: Hard Combinatorics, Game Theory, Expected Value & Polynomials/NTT

This backlog records the language, standard-library, and tooling friction surfaced by porting 10 unported hard problems from `corpus/submissions/` statement-first under `proto/ports/scout_batch_23/`, then verifying runnable `today/` counterparts against the compiled C++ originals (200/200 randomized and edge-case differential tests passing on `proto/cp.py lib-run`).

Across the 10 problems (`noi18p2`, `ioi08p3`, `noi09p5`, `noimps22p4`, `noi19p5`, `dpi`, `noi07p1`, `egoi22p2`, `ioi17p3`, `coci21c1p2`), raw C++ totals 1,718 LOC and library-stripped C++ core totals 621 LOC. Ideal CP Lang drops that to 378 LOC (78.0% raw / 39.1% library-stripped reduction), while today's runnable `proto/cpi` code takes 499 LOC (71.0% raw / 19.6% library-stripped reduction). Almost every line in that 121-line gap comes from the six recurring friction clusters below.

---

## 1. `Mod<P>`, `Comb<P>`, and Cross-`@multitest` Precomputation

`std.mod.{Mod, Comb}` is genuinely clean inside a single test case: `Mod<1024523>` works seamlessly for non-standard primes (`noi09p5`), `Comb<998244353>.choose(n, k)` already returns `Mod<P>(0)` when `k < 0 or k > n`, and `Mod<P>` integrates with `Sum<Mod<P>>` and `Prod<Mod<P>>` in `Fenwick` and `SegTree`. Three sharp rough edges showed up immediately.

**No persistent precomputation across `@multitest` (`noi18p2`).** In NOI '18 P2, a file has multiple test cases with `sum(n) <= 6e5`, and you must build `Comb<998244353>(2 * MAX_N)` once for the whole program. Writing `static let` inside an `@multitest` file fails to parse in both `proto/cpi` and `proto/cpc`:

```cp
@multitest
use std.mod.{Mod, Comb}

type Z = Mod<998244353>
static let comb = Comb<998244353>(1200005)
// error: line 18:8: expected end of line (at 'let')
```

Without a persistent precomputation construct (either top-level `static let` / `@once`, or making `Comb<P>` lazily grow its internal factorial table on demand), the programmer has to give up `@multitest`, manually read all test cases into `List<List<Int>>`, scan for `max_n`, build `Comb`, and loop over cases — 15 lines of pure plumbing in `today/noi18p2.cp`. A lazily-growing `Comb<P>` (or `@once` evaluated before the `@multitest` loop) eliminates both the boilerplate and the classic contest bug of guessing `MAX_N` wrong or accidentally rebuilding `Comb` inside the test loop (`O(T * MAX_N)` TLE).

**`proto/cpc` cannot compile any `std.mod` or `Float` program yet.** Running `proto/cp.py build` on our modular ports (`egoi22p2.cp`, `noi19p5.cp`, `noi09p5.cp`, `ioi08p3.cp`) immediately aborts with:

```text
error: line 14: std.mod.Mod is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)
```

And running `proto/cp.py build` on `dpi.cp` fails with `error: line 13: unknown type Float`. Eight of the ten problems in this batch cannot reach `cpl build` or `g++ -O2` until `Mod<P>`, `ModR`, `Comb<P>`, and `Float` lower cleanly through the compiler pipeline.

**Hot-loop modular division has a silent `O(log P)` trap (`noi19p5`).** In `noi19p5`, every one of the `5e5` shuffles divides by `n`, `n - 1`, `n - 2`, and `2`. Writing `Z(cut) / Z(n)` is natural and idiomatic, but in `stdlib/mod.cp` `/` computes `ext_gcd` or `pow(P - 2)` on every call. To hit the 1.0s time limit, the port had to hoist four manual inverse variables (`inv_n`, `inv_n1`, `inv_n2`, `inv_2`) and multiply by them. Emitting compile-time inverses for constant divisors (like `Z(2)`) and flagging loop-invariant `/` in the LSP would catch the TLE before submission.

---

## 2. `ModR` vs `Mod<P>` Asymmetry and the `SegTree` + `Monoid` Collision

IOI '08 P3 (`ioi08p3`) computes answers modulo a runtime, possibly composite integer `M <= 30000` using a point-update range-product segment tree. Because `Mod<P>` is a real value type implementing `Prod<Mod<P>>`, a static-modulus product tree is one clean line:

```cp
type Z = Mod<998244353>
var st = SegTree<Gem, Z>.new(Prod<Z>)
st.set(g, Z(eaten[g] + 1))
let pref = st.fold(Gem.first..t)
```

Switch to a runtime modulus `m_val = read()` and everything fractures at once. `ModR` in `stdlib/mod.cp` is an operation struct, not a value type, so `ioi08p3` degrades to:

```cp
let mod = ModR(m_val)
let mul_m = Monoid<Int>(1 % m_val, (a, b) => mod.mul(a, b))
var st = SegTree<Gem, Int>.of(mul_m, _ => 1 % m_val)
...
st.set(Gem(g), (cnt[g] + 1) % m_val)
let term1 = mod.mul(ways_small, cnt[r] % m_val)
let term2 = mod.mul(ways_small, ways_hi)
ans = mod.add(ans, mod.add(term1, term2))
```

This creates a **three-way multi-feature collision** between `ModR`, `Monoid<T>`, and `SegTree<D, T>`:

1. **No algebraic trait conformance.** `ModR` values are raw `Int`s, so `Prod<ModR>` does not exist, operators `+`, `-`, `*` disappear, and every leaf write or identity needs a manual `% m_val` (forgetting `1 % m_val` gives a WA when `m_val == 1`).
2. **Closure-captured runtime state inside `Monoid<Int>`.** `mul_m` captures `mod` inside its lambda. In C++17 lowering, a stateless `Prod<Mod<P>>` monoid has zero size and inlines cleanly, whereas a capturing `Monoid` either bloats every `SegTree` node/call or requires monomorphization over the closure type.
3. **Unification opportunity with runtime-frozen `let`.** CP Lang already freezes runtime sizes for `domain` (`let K = n`). Extending that exact same mechanism to a scoped runtime modulus — `let mod M = m_val` (or `type Z = Mod<M>` backed by a single immutable runtime `M` or thread-local Barrett/Lemire reducer) — would let `ioi08p3` use `SegTree<Gem, Z>.new(Prod<Z>)`, normal `+`/`*` operators, automatic reduction on `Z(cnt[g] + 1)`, and `Z.inv -> ?Z` for composite moduli, while keeping the language core small by reusing the same runtime-constant binder as `domain`.

---

## 3. Integer / Modular Promotion and `sum` Over Non-`Int` Types

Mixing `Mod<P>` with plain integers is the single highest-frequency paper cut across all five `Mod<P>` problems (`noi18p2`, `noi09p5`, `noimps22p4`, `noi19p5`, `egoi22p2`). Reproducing directly on `proto/cpi`:

```cp
use std.mod.{Mod}
type Z = Mod<998244353>
let a = Z(3)
print(a + 2)
// error: `+` needs two Mod<998244353> values, got 2

let xs = [Z(1), Z(2), Z(3)]
print(sum(xs))
// error: no `+` for 0 and 1
```

In real contest formulas (`noi19p5`, `noimps22p4`, `noi09p5`), this turns every coefficient, comparison, and constant into `Z(...)` noise:

```cp
// What you have to write today (noi19p5, noimps22p4, noi09p5):
b = d1 - Z(3) * a
prod *= pow2[cb] - Z(1)
if cur == Z(0): continue
let p = Z(cut) * inv_n

// What you expect to write:
b = d1 - 3 * a
prod *= pow2[cb] - 1
if cur == 0: continue
```

Even worse, look at the `sum(xs)` error message: `no '+' for 0 and 1`. Because `Mod<P>.show` formats `Z(1)` as `"1"`, the diagnostic makes it look like adding `0` and `1` failed. Two minimal fixes eliminate almost all of this:

- Allow untyped integer literals (`0`, `1`, `2`, `3`) — or symmetric `Add`/`Sub`/`Mul`/`Eq` with `Int` when the other side is `Mod<P>` — to convert to `Mod<P>` automatically without widening to arbitrary implicit conversions elsewhere.
- Have `sum(xs)` seed from `T.zero` (via `Additive` / `Sum<T>.id`) rather than hardcoded `Int(0)`, and print type names in binary-operator type errors (`Int` and `Mod<998244353>` instead of `0` and `1`).

A parallel promotion paper cut appears with `Float` in `noi07p1`: writing `Float(ways[s, v] * ways[v, t]) / Float(ways[s, t])` multiplies in `Int` first (which overflows `Int64` if `ways > 3e9`), whereas `Float(ways[s, v]) * Float(ways[v, t]) / Float(ways[s, t])` is safe. Requiring explicit `Float(...)` casts is defensible (no silent precision loss), *provided* the compiler warns when an integer product is immediately cast to `Float` or when `Int / Int` is assigned to a `Float` variable.

---

## 4. Polynomials, Formal Power Series, NTT, and Bitwise Transforms

**Corpus reality check.** Scanning all unported AC submissions in `corpus/submissions/` confirms that literal NTT/FFT (`998244353` butterfly loops) does not appear in the remaining unported corpus dump; however, four problems in Batch 23 expose the exact mathematical operations that `std.poly` and `std.bits` need to cover.

**Formal power series inversion over non-NTT primes (`egoi22p2`).** EGOI '22 P2 asks for solid walls modulo `10^9 + 7` with `w * h <= 250000`. Formally, `S(x) = 1 - T(x)^(-1) mod x^(w+1)`. Because C++ has no built-in FPS inversion and `10^9 + 7` is not NTT-friendly, the corpus submission `5199888.cpp` had to write **two completely different algorithms** — an `O(w^2)` naive convolution for `w <= 10^4`, plus an `O(w * h^2)` column-profile DP with a Pascal triangle for `w > 10^4` (and a hardcoded `if (m == 2 && n >= 10)` branch). With a userland `std.poly` module that dispatches NTT for NTT-friendly primes (like `998244353`) and 3-prime / split-base MTT for arbitrary primes (like `10^9 + 7`), both regimes collapse into two lines:

```cp
use std.poly.{Poly}
let solid = Poly<Z>.one - Poly<Z>(total).inv(w + 1)
print(solid[w])
```

Designing `std.poly` only around `Mod<998244353>` would miss `10^9 + 7` problems like `egoi22p2`. `Poly<Mod<P>>` should work for *any* prime `P`, choosing fast NTT when `(P - 1)` has enough factors of `2` and automatically falling back to split-base MTT (or `O(n^2)` for small degree) otherwise.

**Small-degree polynomial interpolation (`noi19p5`) and polynomial products (`dpi`).** In `noi19p5`, every shuffle maps a degree-2 polynomial to another degree-2 polynomial, evaluated at 3 points `(1, g1)`, `(2, g2)`, `(n, gn)` and reconstructed. Without `Poly<Z>.interpolate`, `noi19p5.cp` had to hand-derive Newton finite differences with four cached inverses (12 error-prone lines), while Ryan's C++ `5652798.cpp` wrote a custom `struct Quadratic` + `interpolate(y0, y1, y2)` + a 14-iteration bitmask loop. Having `Poly<Z>.interpolate(pts)` and `p.eval(x)` in `std.poly` turns reconstruction into:

```cp
f = Poly<Z>.interpolate([(Z(1), g1), (Z(2), g2), (Z(n), gn)])
```

Similarly, `dpi` computes `prod_{i=1..N} ((1 - p_i) + p_i * x)`; in `O(N^2)` it is a 6-line reverse loop, while a `Poly<Float>.product` helper (divide-and-conquer FFT) immediately scales the same pattern to `N = 2e5` when needed.

**SOS (zeta/Möbius) transform and submask iteration (`noimps22p4`).** In `noimps22p4`, we run a 13-bit Sum-Over-Subsets transform over `total_sub` and over each active big prime's frequency table, then iterate over all submasks of `sm_req` (`sub = (sub - 1) & sm_req`) and branch on `popcount(sub) % 2`. Today, none of these exist in `stdlib`:

- `popcount` is missing on `Int` in `proto/cpi`, forcing an 8-line manual bit-shifting loop in `today/noimps22p4.cp`.
- Submask iteration has no iterator helper, forcing the classic off-by-one-prone `while true: ... if sub == 0: break; sub = (sub - 1) & mask` loop.
- SOS / zeta / Möbius transforms (and FWHT for XOR/AND/OR convolution) are the bitwise dual of `std.poly` convolution. Adding `Int.popcount`, `submasks(mask)`, and `sos_add` / `sos_sub` in `std.bits` removes ~25 lines of boilerplate from `noimps22p4` alone.

---

## 5. Probability / Expected Value Floats, Formatting, and Game-Theory DP

**Silent float format-specifier bug in `proto/cpi` (`dpi`, `noi07p1`).** Both `dpi` (precision `1e-9`) and `noi07p1` (exact 3 decimal places) require fixed-precision floating-point output. Probing `proto/cp.py lib-run` directly reveals that format specifiers inside interpolated strings are silently ignored:

```cp
let x = 1.5
print("{x:.3f}")
// actual stdout from proto/cpi: 1.5   (expected: 1.500)
```

On exact-decimal judges like NOI '07 P1 (`0.000` vs `0.0`), silently dropping `:.3f` produces an instant Wrong Answer. Even for tolerance judges like `dpi`, raw float printing switches to scientific notation (`1e-05`) on small probabilities, which older judges can reject. Format specifiers (`{x:.3f}`, `{x:.10f}`) need first-class support in both `proto/cpi` and C++17 lowering (`printf("%.3f\n", x)` / `std::fixed << std::setprecision(3)`).

**`@memo` over 3D DP and rolling 2D arrays (`coci21c1p2`, `noi09p5`).** In `coci21c1p2`, valid states satisfy `0 <= l <= r < n` and `my_red < k`. Because `@memo` does not exist in `proto/cpi` yet, `today/coci21c1p2.cp` had to replace an 8-line `@memo fn win(l, r, my_red) -> Bool` with a manual flat `List.filled((n+1)*(n+1)*(k+1), -1)` and `fn cell(l, r, my_red) -> Int`. Meanwhile, Ryan's C++ `4080842.cpp` allocated `bool dp[n][n][k + 1]` as a 43 MB VLA on the stack, which segfaults on a standard 8 MB Linux stack. `@memo` over bounded domains should heap-allocate flat dense tables (with compact `U8` / tri-state packing for `Bool` return types), turning `coci21c1p2` from 45 lines of C++ (with a stack-overflow bomb) into 15 lines of safe CP Lang. In `noi09p5`, rolling `var dp: [I, I]Z` over `k = 0..n+m` cut memory from Ryan's 500 MB 3D `memo[505][505][505]` down to `~2 MB` (`501^2 * 4` bytes).

**MinPlus + path-count semiring (`noi07p1`).** Counting shortest paths in Floyd-Warshall (`if cand < dist[i, j]: ... elif cand == dist[i, j]: ways += ...`) is just Floyd-Warshall over the `MinCount` semiring `(dist: Int, ways: Int)`. Notice how `5032737.cpp` spent 91 core lines writing two custom structs (`SPG` Dijkstra + `PCNT` Kahn topo-sort), whereas Floyd-Warshall with `(dist, ways)` is 42 lines — and with a stdlib `MinCount` semiring + `floyd_warshall` it would be ~15 lines.

**Retrograde games and output formatting (`ioi17p3`).** In `ioi17p3`, the ideal solution wrote `print([if alive[v]: 1 else: 0 for v in V])`, and `today/ioi17p3.cp` wrote `print(out)`, which prints `[1, 0]` with brackets and commas instead of `1 0`. Almost every CP problem prints space-separated lists; needing a manual `join` or loop every time is recurring friction (and `ioi17p3` also raises the IOI grader question: DMOJ wraps `who_wins` in stdin/stdout, while official IOI graders link a C++ function signature).

---

## 6. Non-Typeable Stress Constraints for `cpl stress`

Every hard problem in Batch 23 has a structural input precondition that scalar range annotations (`1..=N`) cannot express — and almost every one of them **breaks naive rejection sampling** if `cpl stress` only treats `where` clauses as post-hoc filters:

- **Permutation (`noi18p2`):** `let q: [Pos]Val = read() where is_perm(q)`. Naive rejection succeeds with probability `n! / n^n ~ e^(-n)` (0% for `n >= 20`). Needs constructive `gen.perm(n)`.
- **Surjection (`ioi08p3`):** Every gem `1..=k` must appear on at least one of the `F` fish (`F >= K`, `distinct(gem) == k`). Missing this constraint triggers an empty-list / bounds bug in `by_gem[t][eaten[t]]`. Needs constructive `gen.surjection(f, k)`.
- **Coupled product bound (`egoi22p2`):** `let (w, h): (1..=250000, 1..=250000) = read() where w * h <= 250000`. Rejection sampling on `[1..250000]^2` accepts with probability `ln(250000) / 250000 ~ 0.00005`. Needs coupled generation (`pick w, then h in 1..=(250000 / w)`).
- **Prime-only queries with total-length budget (`noimps22p4`):** Every query element `c_i` must be prime (`c_i <= 2000`), and `sum(m) <= 18000` across all `q` queries. Needs `gen.subset(primes_up_to(2000))` + budget-partitioned `gen.partition(total_m, q)`.
- **Parity + float range (`dpi`):** `n` must be odd (`n % 2 == 1`) and every `p[i]` lies in `[0.0, 1.0]` (no float range type `0.0..=1.0` exists yet).
- **Fixed-alphabet strings (`noi09p5`, `coci21c1p2`):** `noi09p5` needs strings over `{'A', 'B'}` of exact lengths `n` and `m`; `coci21c1p2` needs strings over `{'C', 'P'}` with `count('C') >= 2 * k - 1` (naive random strings fail whenever `2 * k - 1 > n / 2` and trigger the `l == r` empty-board edge case).
- **Connected simple undirected weighted graph (`noi07p1`):** No self-loops, no parallel edges, connected.
- **Directed graph with min out-degree >= 1 (`ioi17p3`):** Every station has at least one outgoing track, no duplicate edges. Without `out_deg >= 1`, the attractor game semantics are undefined.

**Design takeaway:** `where` predicates on `read()` are valuable for documentation and debug-mode `assert`s, and `cpl stress` should pair them with a small composable library of constructive generators (`std.gen.{perm, surjection, bounded_product, simple_graph, dir_graph, str_of}`) rather than relying on rejection sampling.

---

## 7. Multi-Feature Collisions and Reproduced `proto/cpi` Traps

Beyond the `ModR × Monoid × SegTree` collision (§2) and `@multitest × Comb<P>` (§1), porting these 10 files exposed five concrete syntax/semantics traps directly reproduced on `proto/cpi`:

1. **Silent scalar-vs-array binding on `[D]T = val` (`ioi08p3`, `noi09p5`).** Writing `var max_len: [RawGem]Int = 0` or `var dp: [I, I]Z = Z(0)` did not reject the type mismatch OR broadcast the scalar into an array — `proto/cpi` silently bound the variable to scalar `0` and crashed later with `cannot index 0`. Either `[D]T = expr` should broadcast `expr` across `D` (matching the concise syntax used across earlier scout batches) or it must be a hard compile-time type error.
2. **`if` and `if let` expression asymmetry (`coci21c1p2`, `noimps22p4`).** `let x = if cond: a else: b` works as the entire RHS of `let`, yet embedding it inside an arithmetic expression (`pref[i] + if s[i] == 'C': 1 else: 0`) fails with `expected an expression (at 'if')`, and `let cb = if let arr = big_sub.get(b): arr[comp] else: 0` fails with `expected an expression (at 'let')`. Making `if` and `if let` first-class expressions everywhere removes both surprises.
3. **2D array row lending into `mut` functions (`noimps22p4`).** Passing one row of a 2D array `big_sub: [Big, Mask]Int` into `fn sos(mut a: [Mask]Int)` via `sos(mut big_sub[b])` needs partial slice lending (`lends mut`) on multi-dimensional arrays; without it, `today/noimps22p4.cp` fell back to `Map<Int, List<Int>>` and duplicated the 5-line SOS loop inline.
4. **Collection literal & method-name inconsistencies (`noimps22p4`, `ioi08p3`).** `Map()` works, while `{}` fails (`expected an expression (at '}')`) and `Map<Int, Int>.new()` fails (`Map<Int, Int> has no method 'new'`). `List.sort()` works, while `List.sort_by(...)` does not exist yet. `SegTree` uses `.fold(l..=r)` rather than `.query` or `st[l..=r]` (`cannot slice SegTree(...) with D[0..3]`). Aligning slice indexing `st[l..=r]` (or keeping `.fold` surfaced clearly in LSP completions) prevents guesswork under contest pressure.
5. **Corpus C++ edge-case bugs & stack-VLA crashes caught by differential testing.** Running randomized + small edge-case inputs exposed `1 << (__lg(n - 1) + 2)` UB in `5032789.cpp` (`ioi08p3`) when `F == 1` or `K == 1`, `assert(y == 1)` crash in `5652798.cpp` (`noi19p5`) when `n <= 2`, and 9.5 MB / 43 MB stack-overflowing VLAs in `4547602.cpp` (`noimps22p4`) and `4080842.cpp` (`coci21c1p2`) that segfault under standard 8 MB Linux stack limits. (Separately, on `egoi22p2`, our initial statement-first recall remembered `1x2`/`1x3` bricks instead of `1x1`/`1x2` bricks, caught and fixed immediately upon inspecting `5199888.cpp` before differential testing.)

---

## Prioritized Action Items for the Language Architect

1. **P0 — Value-type runtime modulus (`let mod M = m_val` / `Mod<M>`):** Unify runtime modulus with `domain`'s runtime-frozen `let` so `ModR` gets `+ - *`, `Prod<Z>`, and zero-ceremony `SegTree<D, Z>` integration (`ioi08p3`).
2. **P0 — Integer-literal promotion for `Mod<P>` and `T.zero`-seeded `sum`:** Eliminate `Z(0)`/`Z(1)`/`Z(3)` noise across every modular problem and fix `sum([Z(1), Z(2)])` (`noi19p5`, `noimps22p4`, `noi09p5`, `egoi22p2`).
3. **P0 — Fixed-precision float formatting (`{x:.3f}`, `{x:.10f}`) and `proto/cpc` lowering for `std.mod` and `Float`:** Unblock `dpi`/`noi07p1` exact decimal output and native C++17 builds for 8/10 problems in this batch.
4. **P1 — `std.poly` (`Poly<Mod<P>>` with NTT + arbitrary-prime MTT `inv`/`mul`/`interpolate`) and `std.bits` (`popcount`, `submasks`, `sos_add`/`sos_sub`):** Collapses `egoi22p2`, `noi19p5`, `dpi`, and `noimps22p4` by 50+ lines combined.
5. **P1 — Persistent precomputation across `@multitest` (`static let` or lazily-growing `Comb<P>`) and `@memo` flat dense allocation:** Eliminates `noi18p2` and `coci21c1p2` boilerplate and prevents stack-VLA crashes.
6. **P1 — Constructive `std.gen` combinators for non-typeable `cpl stress` constraints (`perm`, `surjection`, `bounded_product`, `simple_graph`, `dir_graph`, `str_of`)** plus fixing the silent `var a: [D]T = val` scalar binding and `if` / `if let` expression grammar gaps.
