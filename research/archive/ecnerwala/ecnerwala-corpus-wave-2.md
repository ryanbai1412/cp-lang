# ecnerwala corpus, wave 2

The second pass over <https://github.com/ecnerwala/contest-programming>,
continuing newest first from where `ecnerwala-corpus-wave-1.md` stopped. The
ports and their gate are in `proto/ports/ecnerwala/`. Everything here is
CP Lang 0.2.2 against the Rust `cpl` built from `main`.

## What was covered

Wave 2 starts with Universal Cup 5 Stage 5 B, which wave 1 left as "portable
today", and then walks back through upstream history: Universal Cup 5 Stage 3
Gorkovskaya (2026-10-02), AtCoder AGC 078 (09-27), Codeforces 2268 and
Universal Cup 5 Stage 2 EPS (09-26), Universal Cup 5 Stage 1 Nizhny Novgorod
(09-18), Codeforces 2262 (09-12), and Codeforces 2255 and 2245 (August).
Twenty-one submissions are ported and pass every gate.

Each port below passed `cpl check`, `cpl build` compiled and run under ASan
and UBSan, and `cpl run`, with both outputs equal to the original C++ on the
upstream samples (where upstream has any) and on random inputs. "Lines" are
non-blank, non-comment lines: the port, then the original's `main` alone (as
if C++ already had a perfect library), then the whole original file.

    port        lines   main   file   samples   random   what it exercises
    ucup5s05b      48    161   1234      0        200    Euclidean `%` for mirror folding, tuple state, event sweep
    ucup5s03f      17     32    367      0         50    Mod<P>, Comb, in-place difference passes
    ucup5s02e      14     33     38      0         60    domain-typed permutations, nested arrays
    ucup5s01a      31     57    557      3         60    Mod<P> polynomial and a 2D DP table
    cf2268a         6     18     23      1         60    two `sum` comprehensions
    cf2268b        12     27     32      1         60    popcount, printing a list
    cf2268c        33     90    238      1         60    SparseTable over a domain, record stack, sort by tuple
    cf2268d        35    112    138      1         60    monotone stack, next-greater forest
    cf2262a1       11     26     32      1         60    difference array
    cf2262a2       25     47    383      1         60    Mod<P> sliding-window DP
    cf2262c        20     42    383      1         60    Comb.fact, `dedup`, `all`
    cf2262d        57     84     91      1        100    `?Int` links, a function returning an optional record
    cf2255a        10     18     24      1         60    circular Str indexing with `%`
    cf2255b        14     34    276      1         60    `runs` over a Str, Comb.choose
    cf2255c        24     59     68      2        100    input above `@!multitest`, `[0..n]Str`, `first`
    cf2255d        33     53     60      1         60    bit_width buckets, a helper mutating top-level state
    cf2245a         7     18     23      1         60    `count` over a Str
    cf2245b         5     16     21      1         60    `sorted` plus two sums
    cf2245c        19     41     49      1         60    constructive permutation
    cf2245d2       29     58     64      1         60    record edges, peeling queue
    cf2245e        18     55     63      1         60    Dsu over one domain, `.groups()`

In total: 468 port lines against 1,081 lines of original `main` (57% fewer)
and 4,164 lines of whole original files (89% fewer).

Reproduce any row with `proto/ports/ecnerwala/gate.sh NAME CASES`.

Two notes on the originals. `ucup5s05b_orig.cpp` carries a 13-line
`wala::Point` shim at the top (marked as not upstream): the archived file
uses that type without including its definition and does not compile as
committed. `ucup5s05b` has no upstream sample, so it rests on the 200 random
cases alone; the generator mixes tiny boxes with 10^9-sized ones and biases
queries toward lines that actually meet the front.

## Not ported, and why

**Blocked by a compiler bug (see finding 1): four submissions.**
Universal Cup 5 Stage 2 F, H and D and Stage 1 H all need 128-bit integers
(products of two values near 10^18, or exact fraction comparison). Stage 2 F
was fully ported; the port is quoted under finding 1 because it is the
reproduction.

**Needs formal power series over NTT: four.** Universal Cup 5 Stage 3 A,
Codeforces 2268 E, 2262 E and 2255 F. Same gap wave 1 reported.

**Nothing to port: five.** AGC 078 A and C are exploratory game-value
enumerators that read no input and print only to stderr; AGC 078 B,
Codeforces 2268 F and 2262 F are empty stubs.

**Deferred, not attempted in this wave: the rest.** Universal Cup 5 Stage 2
A and B (best-K-transactions queries on a roughly 500-line bespoke
time-indexed segment tree outside `main`), Stage 3 H (Prufer enumeration over
a min-cost max-flow; `MinCostFlow` exists only in `examples/local-lib`),
Stage 1 C (Boruvka over a lazy segment tree on floating-point logs),
Codeforces 2262 B (ordered map and set walking), 2255 E1/E2 and 2245 F, G, H.
None of these was tried, so nothing is claimed about them.

## Findings

Numbered for reference. Items already in the wave 1 list are only mentioned
when wave 2 adds something.

### Compiler

1. **`I128` is accepted and then silently compiled as 64-bit.** `cpl check`
   and the IR lowering both map `I128` to plain `Int`
   (`crates/cpl-check/src/prelude.rs` and `crates/cpl-ir/src/lower/mod.rs`
   list it beside `I32`/`U64`), and `cpl build` emits `ll`. The interpreter
   computes the right answer, so `cpl run` and the emitted C++ disagree, and
   UBSan reports the overflow:

       fn covered(t: I128, k: I128, v: Int, p: List<Int>) -> Bool:
           var a: I128 = 0
           for j in 0..p.len / 2:
               if a >= k: break
               let lo = 6 * k * p[2 * j] - t * v
               let hi = 6 * k * p[2 * j + 1] - t * v
               if lo > 6 * t * a: return false
               if hi >= 6 * t * a: a = hi / (6 * t) + 1
           a >= k

       out.cpp:121: runtime error: signed integer overflow:
       2625999328587073698 * 6 cannot be represented in type 'long long int'

   The emitted signature is `bool covered(ll t, ll k, ll v, ...)`. This is a
   check-accepts-what-codegen-drops case, and it blocks every submission that
   uses `__int128` (four in this wave alone). Until it is implemented,
   `I128` should be a compile error rather than an alias.

2. **A tuple `read()` cannot use an earlier component as a bound.**
   `let (n, k): (1..=2e5, 1..=n) = read()` fails with E0201 ("doesn't know
   the name `n` here"); it has to be split into two `let`s (`cf2268a`).

3. **A list literal does not coerce to a fixed-size array inside a record.**
   With `type Ends = (lo: [0..2]Int, hi: [0..2]Int)`, the literal
   `([INF, INF], [-INF, -INF])` is rejected as `(List<Int>, List<Int>)`
   (E0302). `cf2268c` declares the fields as `List<Int>` instead.

4. **Scalar fill works for one array dimension only.**
   `var cnt: [0..n][0..2]Int = 0` is rejected (E0302), although
   `var freq: [0..n]Int = 0` is the standard idiom. `cf2245d2` writes
   `[[0, 0] for _ in 0..n]`.

5. **Arrays that do not start at 0 are still rejected (E0501).** Wave 1
   reported this; wave 2 hit it three more times (`cf2262a1`, `cf2262a2`,
   `ucup5s01a`), each leaving `a[i - 1]` noise in a problem that is stated
   1-based.

6. **`loop:` is not syntax.** The natural spelling for "retry until return"
   is `while true:`. Minor, but it was the first thing typed.

### Standard library

7. **`Mod ** Int` is rejected (E0309)** although the cheat sheet lists `**`
   for `Mod<P>`. `.pow(e)` works (`ucup5s03f`).

8. **`sum` rejects `Mod` values** ("`sum` adds numbers, not `Mod`").
   `sum(dp[i][0] / Z(i) for i in 1..=n + m)` became a two-line loop in
   `ucup5s01a`. `product` is still missing (wave 1); `Comb.fact` covered the
   two places that wanted it.

9. **No list concatenation.** `xs ++ ys` is a parse error and there is no
   `extend`. `cf2262c` and `cf2245c` both build one list and push the other
   element by element.

10. **No `zip`.** `all(x == y for (x, y) in zip(sorted(merged), a))` became
    an index loop in `cf2262c`.

11. **No `.enumerate` on `List`** (wave 1 reported arrays). No `.count(x)`
    on arrays either (`good.count(true)` was the first thing typed in `ucup5s02e`).

12. **`count` only takes a Bool comprehension.** `count(i for i in 0..k if
    s[i] != 'R')` fails with "expected `Bool`, found `Int`", which does not
    point at the fix (`count(s[i] != 'R' for i in 0..k)`). Either accept the
    filter form or say so in the message.

13. **`dedup` is neither in the prelude nor an expression.** It needs
    `use std.seq.{dedup}`, a `var`, and `dedup(mut xs)`; the port wanted
    `for x in distinct(sorted([...]))`.

14. **`Dsu<V>` over two kinds of node needs manual offsets.** `cf2245e`
    unions tree edges with vertices, so it declares one domain
    `0..2 * n - 1` and writes `X(n + e)` and `x.ord >= n` by hand. A sum
    domain, or `Dsu` over a tagged pair, would remove that.

### What worked well

- Euclidean `%` removed the "fix a negative remainder" lines from every
  folding and circular-index helper (`ucup5s05b`, `cf2255a`, `cf2255c`).
- Input read above `@!multitest` (`let op: Str = read()` in `cf2255c`) made
  a two-mode communication problem read top to bottom with no wrapper.
- `runs(s)` over a `Str`, `first(...)!`, `?? -1`, `print(list)` for
  space-joined output (including the empty line for an empty list),
  `Comb<MOD>()`, `SparseTable<I, Int>(max, a)` and `Dsu.groups()` each
  replaced several lines of the original with one.
- UBSan on the emitted C++ caught a real overflow in a first draft
  (`6 * k > t` with `k` near 10^18), which is exactly what the sanitizer gate
  is for.

## Not tested

- Maximum-size inputs, running time and peak memory. Random cases are small
  (tens of elements, a few with 10^9-sized coordinates). `cf2268c` sorts
  where the original uses a hash map and counting sort, and `ucup5s01a`
  allocates a fresh table per step, so both deserve a max-N timing run.
- Constructive problems (`cf2262d`, `cf2245c`, `cf2245d2`, `cf2255c`) are
  compared byte for byte with the original's construction, not through a
  checker, so they prove equality with upstream rather than validity.
- The submissions listed as deferred.
