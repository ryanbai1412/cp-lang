# Friction report: ioi25_triples

## Dream

```cp
for (g, group) in at.group(by: x => h[x] - x):
    if group.len <= isqrt(h.len):
        for a in 0..group.len:
            for b in a + 1..group.len: crossed(group[a], group[b], group[a] + h[group[b]])
    else:
        for k in at if (k - g - h[k]) % 2 == 0:
            crossed((k - g - h[k]) / 2, (k - g + h[k]) / 2, k)
```

Part I counts the triples by which position holds the largest distance, k - i. If an end
holds it, or the middle holds it with h[i] = j - i, one position fixes the triple. The
remaining middle case pairs positions with equal h[x] - x: all pairs in groups of at most
sqrt(n), and one solve per k in the larger groups. Part II puts peaks at the midpoints of a
random even point set and keeps the best of 8e5 / M tries. This is the official model
solution's algorithm. The port and the dream share the counting code, and both are stdin
programs that read the sample grader's input. The dream as first written failed `cpl check`
on the names below. The errors quoted come from that dream (commit 44d032e0f) on a freshly
built `cpl`.

## Std names and syntax the dream wanted that don't exist
- **Where:** `dream.cp` as first written. Each name, its error and what the dream uses now:

| Wanted | Error | Used instead |
|---|---|---|
| `let (i, j, k) = sort3(x, y, z)` | ``error[E0201]: no `sort3` in scope`` | a local `fn sort3(a, b, c) -> (Int, Int)` giving the min and middle |
| `(i, j, k) all in I` | ``error[E0103]: expected `:` after the condition, found a name`` | `i < j < k and i in at and k in at` (only the ends need the check) |
| `let found = Bag<(I, I, I)>()` with `.add` | ``error[E0201]: no `Bag` in scope, and not in the stdlib (yet)`` | `var found = 0`, and each candidate names the position `top` that holds k - i, so every triple counts once |
| `Rng(seed)` with no import | ``error[E0201]: no `Rng` in scope`` | `use std.rand.{Rng}` and `Rng.new(seed)` |
| `.max_by(h => count_triples(h))` | ``error[E0203]: a `List<List<Int>>` has no method `max_by` `` | `.map(h => (count_triples(h), h)).max(by: p => p.0)!.1` |

- **What felt wrong:** `Bag` was there to deduplicate. The same triple can be found from two
  positions, and a set of triples is the obvious way to count it once. Without it, the dream
  must prove a canonical finder for every triple. The first version without the set counted
  some triples twice: it disagreed with orig.cpp on gen.py seeds 51, 77 and 383 (7 vs 8, 29
  vs 30, 52 vs 53). The `top` argument fixes this. `all in` reads naturally next to `x in I`.
  `sort3` and `max_by` are names a C++ or Python user reaches for first. `max_by` is the same
  friction as research/frictions/ioi25_worldmap.md.
- **Goal it hurts:** express, hard-to-get-wrong.
- **Ideal code:** `let found = Bag<(I, I, I)>()` with `found.add(ok(...))` and
  `found.len`, `(i, j, k) all in I`, and `(x, y, z).sorted()` on a homogeneous tuple.
- **General fix:** in std, `Bag` (or `Set.extend` over an optional, the form
  corpus/gold/ioi25_triples.cp writes), and `sorted` on homogeneous tuples. In the parser,
  either accept `xs all in R` or give E0103 a help line naming `[..].all(x => x in R)`.
  For a missing `max_by`, E0203 could offer `max(by:)`. Missing types such as `Rng` could
  get a help line naming the `use`. Owner: stdlib lead (Content), diagnostics.
- **Severity:** high (the double count is a silent wrong answer), low (the rest).

## The `@grader` signatures are dropped
- **Where:** the dream as first written, `@grader fn count_triples(h: List<1..2e5>) -> Int`
  and `@grader fn construct_range(m: 3..=2e5, k: Int) -> List<Int>`. Run without a grader,
  a `@grader fn` program stops with `cpl: the judge's grader calls ..., and ... has no
  `main` without it: run it with `--grader grader.cpp`, or put grader.cpp beside its tests`.
  The dream now ends with the sample grader's I/O loop (`part`, then `N, H` or `M K`).
- **What felt wrong:** the program verified is not the program submitted. The judge's
  interface is two typed functions, and the dream copies the grader instead. The official
  grader in the task archive carries the judge's secret tokens, so it can't be committed.
  research/frictions/ioi25_worldmap.md links the public sample grader from `dream.tests/`
  instead. This package doesn't do that yet.
- **Goal it hurts:** evidence, submission.
- **Ideal code:** the two `@grader fn`s and nothing else.
- **General fix (tools):** the same as the `@grader` section of
  research/frictions/ioi25_souvenirs.md: a package commits the sample grader, and
  `corpus.py` passes `--grader` for the dream. The port could then share it.
- **Severity:** medium.

## `for (i, j) in group.pairs()` builds every pair first
- **Where:** `dream.cp` as first written, `for (i, j) in group.pairs(): crossed(i, j, i +
  h[j])`, now nested index loops (see the Dream section). `cpl build` turns that loop into
  `std::vector<std::tuple<ll, ll>> t7 = m2_pairs_Lst_Int_Int_131(group);`. That function
  first copies `group` (`self.collect()`), then `push_back`s all |group|^2 / 2 pairs before
  the loop starts. In the same program, `for (x, y) in s.pairs() if ...` is emitted as a
  streaming `cpl_each_m2_pairs_...` callback. `pairs`'s doc (std/seq.cp) says "O(1) per
  pair". It doesn't say that a plain `for` allocates them all.
- **Measured:** these are `cpl build` plus `g++ -O2`, best of 5. The Part I `--max` case is
  N = 200000. The Part II case is M = 200000 with five tries.

| Inner loop | Part I `--max` | Part II M = 200000 |
|---|---|---|
| `group.pairs()` (first dream) | 0.29 s | 1.42 s |
| index loops | 0.20 s | 0.97 s |
| `for s in [-1, 1]` unrolled into two cases, still `pairs()` | 0.30 s | 1.42 s |

  Under `cpl test` with the 2000 ms limit, the first dream took 2221 ms on Part II (M =
  200000), a TLE. It also tripped the `slow --max` gate against orig.cpp. Unrolling the
  two-element list literal had no measurable effect, so the slowness is `pairs()` alone.
  `dream.cp` still has the unrolled form, and `for s in [-1, 1]` could come back at no cost.
  The tries went from 1e6 / M to 8e5 / M (four at M = 200000) for headroom. The dream now
  takes 1486 ms there.
- **What felt wrong:** `pairs()` is the obvious way to write "every pair in a small group",
  but in a hot loop it costs a third of the runtime. The cost line doesn't show this, so the
  obvious way isn't the fast way. `cpl lint` also pushes back to the slow form: on the index
  loops it warns ``warning[L0543]: `for b in a + 1..group.len` walks each pair `a < b` by
  hand`` with `help: write `for (x, y) in group.pairs()``. The port keeps that warning.
- **Goal it hurts:** performance.
- **Ideal code:** `for (i, j) in group.pairs(): crossed(i, j, i + h[j])`, as fast as the
  index loops.
- **General fix (codegen):** emit a `for` over a `yields` fn as the streaming
  `cpl_each_` callback whenever the body allows it (it already does for the filtered form).
  Have `pairs` borrow `self` when it is a `List`, not `collect()` it. This helps every
  `pairs`/`windows`-style generator in a hot loop. Owner: codegen.
- **Severity:** medium.

## Mapping a closure's local table copies it
- **Where:** `port.cp` and `dream.cp`, `attempt`: `h.map(v => v ?? rng.int(1, 2))` on the
  `var h: [0..m]?Int` local to the nested fn. `cpl explain` gave
  `[copy] 76:9  h -> a `let` inside a closure owns its value`. Repro:
  `fn f(m: 1..=10) -> List<Int>:` / `fn g(seed: Int) -> List<Int>:` /
  `var h: [0..m]?Int = None` / `h[seed % m] = seed` / `h.map(v => v ?? 0)` / `g(1)`, then
  `print(f(read<1..=10>()))`: `cpl explain` reports the same copy at `h`.
- **What felt wrong:** `h` is dead after the map, so the map can read it in place. The port now
  writes `var h: [0..m]Int = m` with `m` as a "no peak yet" sentinel, fills the sentinels in a
  loop and returns `List(h)`: a sentinel where `?Int` said the same thing exactly.
- **Goal it hurts:** zero-cost safety, costs are visible.
- **Ideal code:** the dream's `h.map(v => v ?? rng.int(1, 2))` with no copy.
- **General fix (codegen):** a closure-local table read for the last time is a borrow (or a
  move) into the map, as it is outside a closure.
- **Severity:** medium (one O(M) copy per try; 8e5 / M tries).
- **Status:** no longer reproduces on 63669f09c: `cpl explain` reports no copy for the repro,
  and the dream's `h.map(v => v ?? rng.int(1, 2))` makes none (read_only_copies).

## `max(by:)!.1` copies the winning element
- **Where:** `construct_range`,
  `(0..8e5 / m).map(attempt).map(h => (count_triples(h), h)).max(by: p => p.0)!.1`.
  `cpl explain` gave two reasons on it: "an optional holds its own value" and "the name is
  moved later, so it owns its value". Repro:
  `let m: 1..=10 = read()` /
  `print((0..m).map(s => [s, s + 1]).map(h => (h.len + h[0], h)).max(by: p => p.0)!.1)`.
- **What felt wrong:** the pipeline is the editorial's "keep the best try", one line. Taking a
  field out of the unwrapped optional copies the list instead of moving it, so the port spells
  the argmax as `var (best, most)` and an `if count > most` loop (first maximum, like
  `Ties.First`).
- **Goal it hurts:** the obvious way is the correct and fast way.
- **Ideal code:** the dream line, with `!` and `.1` moving out of the temporary.
- **General fix (codegen):** unwrapping a temporary optional and projecting a field moves the
  field; the same helps `min(by:)`, `first(...)` and `find` results that are taken apart.
- **Severity:** medium.
- **Status:** no longer reproduces on 63669f09c: `cpl explain` reports no copy for the repro.
  The dream now writes `(0..8e5 / m).map(attempt).max(by: count_triples)!`, with no `.1` at all.

## Comparing sorted three-element lists allocates per candidate
- **Where:** the dream's `mythical` as first rewritten:
  `[h[i], h[j], h[k]].sorted() == [j - i, k - j, k - i].sorted()`.
- **What:** each side is a heap `std::vector` built, sorted by `cpl::sort_seq` and freed for
  every candidate (several per position), so on case 08 (M = 200000) the dream took 8.89 s
  CPU, with `mythical`, `malloc`/`free` and the insertion sort at about 60% of samples. The
  dream and the port now say the same fact as arithmetic: with `h[top] = k - i`, the other two
  heights are `j - i` and `k - j` exactly when the three sum to `2 (k - i)` and one of the
  other two is either. That takes the dream to 0.83 s and cut the port's instruction count
  from 8.51e9 to 3.00e9 on case 08 (callgrind; orig.cpp 9.44e9), since the old `sort3`
  comparison was two branchy min/max chains per candidate.
- **General fix (codegen):** keep a list literal of known small length that does not escape
  on the stack (a `std::array` and an in-register sort), so the multiset comparison is free.
- **Severity:** medium.

## The `1..n` height bound drops at `count_triples`
- **Where:** `port.cp:89-90`, `dream.cp:70`.
- **What:** Part I reads `h: [n]1..n` and Part II builds a `[0..m]Int`, and one
  `count_triples(h: List<Int>)` serves both, so the statement's `1..=N - 1` and the position
  domain are gone inside it. The `.map(x => Int(x))` both programs had was not needed (a
  `[n]1..n` passes as `List<Int>` as is, no copy) and is removed. A generic
  `fn count_triples<P>(h: [P]Int)` compiles and takes both tables, but every position here
  comes from arithmetic (`x + h[x]`, `(k - g - h[k]) / 2`), so each read would need a checked
  `P` conversion, which reads worse than the range test `y in at` it replaces.
- **General fix (check, stdlib):** let a parameter state its element bound in terms of its
  own length (`h: List<1..h.len>`), so both callers keep the bound without a domain.
- **Severity:** low.

## Measurements

- **Tokens** (scripts/ports-page.py tokenizers): port 823, dream 788, original 1171 (1132
  after `strip_cpp_library`; it includes the sample grader's `main`).
- **Runtime and memory** (`-O2`): on the `gen.py --max` input (Part I, N = 200000), the port
  takes 0.20 s and 11.3 MB, and orig.cpp takes 0.22 s and 12.5 MB. On Part II with M =
  200000, the port took 0.84 s and 21.1 MB against orig.cpp's 1.13 s and 14.4 MB (1.47x
  the memory) before the two copies above were removed, then 1.65 s and 14.7 MB against
  orig.cpp's 1.83 s and 14.4 MB. With the arithmetic `mythical` test above, case 08 (M =
  200000, K = 12000000, which only the fourth try reaches) takes the port 0.64 s min, 0.85 s
  median of 7 runs (CPU, 14.9 MB), the dream 0.85 s / 1.07 s (19.9 MB), and orig.cpp 1.38 s /
  1.49 s (14.4 MB). The Part I `--max` case takes the port 0.17 s and 9.6 MB against 0.30 s
  and 12.5 MB. The port also returns the first try that reaches K (no saving on case 08).
  Under `cpl test` the program is built with `-DLOCAL`, which turns on the overflow check in
  every `iadd`/`isub`/`imul`; `count_triples` does a dozen per candidate, so that build took
  1.04-1.93 s on case 08 against 0.66-0.89 s without it, and the harness charged 1.77-1.96 s
  with one 2010 ms TLE in four runs. The crossed rule now checks what is left once i and j
  share h[x] − x (k = i + h[j], then h[k] = j − i: three checked operations per pair, not a
  dozen). With it, 7 runs each under g++ 14 `-O2` on case 08: the port takes 0.56-0.75 s
  (14.8 MB), the dream 1.03-1.36 s (20.0 MB) and orig.cpp 1.61-2.49 s (14.4 MB). Under
  `cpl test` (CPU time, the `-DLOCAL` build): the port 0.76-0.99 s, the dream 1.23-1.83 s,
  orig.cpp 1.95 s.
- **Copies:** neither the port nor the dream makes any (read_only_copies, on 63669f09c).
- **Emitted C++:** port 27548 bytes, dream 28576 bytes (`cpl build`, judge `generic`, on 63669f09c).
- **Agreement:** the dream and port match orig.cpp on all 29 package cases, and the dream
  matches it on 1500 gen.py seeds (sizes 10, 40 and 300). orig.cpp itself agrees with the
  archive's O(n^3) and O(n^2) correct solutions.
