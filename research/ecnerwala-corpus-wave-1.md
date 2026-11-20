# ecnerwala corpus, wave 1

A first pass over <https://github.com/ecnerwala/contest-programming>, newest
submissions first, as a second external corpus beside DMOJ. The ports and
their gate are in `proto/ports/ecnerwala/`.

## What was covered

The wave is every submission upstream dated 2026-10-17 or later: AtCoder
AGC 079 (2026-11-08), Universal Cup 5 Stage 6 Hyakumanben (11-06) and Stage 5
Chengdu (10-31 and 11-01), and Codeforces round 2261 (10-17). That is 19
files. Thirteen are ported and pass every gate, three are not ported yet, and
three have nothing to port.

Each port below passed `cpl check`, `cpl build` under ASan and UBSan, and
`cpl run`, with both outputs equal to the original C++ on the upstream sample
and on random inputs. "Lines" are non-blank, non-comment lines: the port, then
the original's `main` alone (as if C++ already had a perfect library), then
the whole original file.

    port        lines   main   file   random cases   what it exercises
    agc079a        15     28   1075        50        Mod<P>, a stack of records
    agc079c        76    154    178       150        linked runs on a stack, Deque, early `return` in @!multitest
    cf2261a         5     27     34        50        Str, comprehension
    cf2261b         8     17     22        50        bit tricks, `.adjacent`
    cf2261c         7     27     39        50        counting
    cf2261d1        8     18     23        50        constructive output
    cf2261d2       26     61     68        50        xor hashing with `rand`, `scan`, sort by key
    cf2261e        14     49     54        50        stack plus `bsearch_first`
    cf2261f1       21     73     87        40        output only; builds `Str` from a generator
    cf2261g        39     60     69        40        binary trie in a List of records, recursive merge
    cf2261h        35     98    109        60        Hamming code on Str, two-run (encode/decode) problem
    ucup5s05e      47    114    950       100        std.geometry, convex hull, `bsearch_first`
    ucup5s06g      18     69     75        50        Graph, peeling by in-degree, `dijkstra` with a 0/1 weight fn

Over the thirteen, the ports are 319 lines against 795 lines of `main`
(60% fewer) and 2,783 lines of file (89% fewer).

`agc079c` is the hardest of these (a 150-line state machine with no library
help) and its port passed all 150 random cases on the first run.

## Not ported, and why

- `ACM/UCup5/Stage06-Hyakumanben/A` and `ACM/UCup5/Stage05-Chengdu/F` need
  formal power series over NTT (log, inverse, middle product; F also a
  Cartesian tree over the LCP array). The stdlib has no NTT or power-series
  module, so these wait on one. This is the largest gap the corpus shows:
  upstream's own library is mostly this.
- `ACM/UCup5/Stage05-Chengdu/B` (reflections on a grid, about 130 lines of
  integer geometry) is portable today and was left for the next wave.
- `CF/2261/F2` is byte-for-byte the same program as F1, so `cf2261f1` covers it.
- `AtCoder/AGC079/B` is unfinished upstream (the solving lambda never
  returns) and `AtCoder/AGC079/D` is an empty `main`.

## Friction

Every item is something the first draft of a port wrote and `cpl` rejected,
or a line that is clumsier than it should be. Nothing was worked around
silently.

1. `product(...)` does not exist. `agc079a` wanted
   `print(product(Z(i) for i in 1..n) / den)` and has a three-line loop
   instead. Already on the wave1 blocked list.

2. Arrays whose index range does not start at 0 are rejected by `cpl run` and
   `cpl build` (E0501). `agc079a` reads `p_2..p_n`; the natural
   `let p: [1..n]1..=n = read()` had to become `[0..n - 1]` with a `- 1` in
   the loop body, which is exactly the off-by-one the range type is for.

3. `.enumerate` is missing on arrays (E0203 on `[Int]Int`), so
   `for (i, lo) in p.enumerate` became `for i in 0..n - 1` and `p[i]`.

4. `sorted(0..n, by: i => ...)` is rejected: "`sorted` takes a sequence, not
   a `Range<Int>`". Sorting indices by a key is the standard idiom
   (`cf2261d2`), and it needs `sorted([i for i in 0..n], by: ...)` today.

5. No way to print without a newline and no `join`. `print(x, sep: "")`,
   `"".join(...)` and `Str.split` are all rejected by the Rust `cpl`, though
   older ports under `proto/ports/scout_batch_*` use `" ".join(...)`. Output
   built a character at a time (`cf2261f1`, `cf2261h`) goes through
   `Str(c for ...)`, which works and reads well once found, but nothing
   points to it. Printing a list on one line works as `print(ids)`.

6. `and` cannot be a field name: `(and: Int, root: Int)` fails with
   "expected a value after `(`, found `and`". Fair for a keyword, but the
   message should say that `and` is a keyword and cannot name a field.

7. `else if` is rejected in favour of `elif`. The error says so and the fix
   is instant, but it is a first-draft slip every C++ writer will make.

8. A record on top of a stack cannot be updated through `.last!`:
   `runs.last!.head = x` fails with E0401 ("only a variable, a field, or an
   element can be assigned to"), so `agc079c` writes
   `runs[runs.len - 1].head = ...`. An assignable `.last` would remove the
   most repeated clumsy expression in the stack-heavy ports.

9. Reading values with a shift has no spelling. `agc079c` reads a 1-based
   permutation and needs it 0-based: `let perm: [0..n]1..=n = read()` then
   `let a = [x - 1 for x in perm]`. With domains (`domain V = 1..=n`) this
   goes away, but for plain "values are positions" input it is two lines and
   two names.

10. `ucup5s05e` reads `[0..n](x: Int, y: Int)` and rebuilds `Pt(...)` for
    each point. `let p: [0..n]Pt = read()` does work in `cpl run` (checked
    after the port was written); the port's first draft did not expect it to.

11. Lexicographic point order is spelled by hand twice in `ucup5s05e`
    (`by: s => (s.p.x, s.p.y)` and a tuple comparison in the search); `Pt` has
    no `Ord`. `std.geometry` also has no convex hull, which is the first thing
    a geometry problem reaches for.

12. A missing import is a good error (`bit_len`, `bsearch_first`, `scan`,
    `Deque` each named the exact `use` line to add), but four of thirteen
    ports hit it. `bsearch_first` and `scan` are common enough to question
    whether they belong in the prelude.

13. Sentinel-heavy originals translate with mixed results. `?Int` children in
    the `cf2261g` trie are clearer than `-1`, but cost a `!` at each use
    (`nodes[t!].kid`, `x!`, `y!`). In `agc079c` the port kept `-1` for
    "no link" because `?Int` made the dense state machine noisier.

## What worked without a second thought

`@!multitest` with a top-level `return` to end one test (`agc079c`,
`cf2261d1`); bounded reads; `Mod<P>` with `/`; `Graph.directed` and
`Graph.undirected` over one edge list, and `dijkstra` taking the weight as a
fn of the arc (`ucup5s06g` turns "forward free, backward costs one" into one
line); `.adjacent`; `bsearch_first` replacing a hand-written binary search;
`Deque`; list slices and `+`.

## Not tested

- No port was run at the problem's maximum size, so there are no timings or
  memory numbers against `g++ -O2` here.
- `ucup5s05e` has no upstream sample, and its constraints were inferred from
  the original (distinct points, not all on one line; coordinates small in
  the generator).
- `cf2261h` in decode mode is tested only on valid encodings with one node
  flipped. On arbitrary strings the original writes one past the end of its
  string; the port traps there instead.
- `cf2261d2` depends on random 64-bit keys in both programs; outputs agree
  unless a hash collides, and none did.
- The Python prototype (`proto/cpc`) was not run on these ports.
