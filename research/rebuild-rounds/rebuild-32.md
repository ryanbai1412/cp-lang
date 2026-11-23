# Rebuild round 32: holes

Cold read of `spec/` and root `std/` only. Statements and samples from the Luogu
mirror (Codeforces returned a challenge page). The cold version (this file and
every `.cp`) was committed before any tool ran; the verification section below
was added afterwards.

## Problems

| # | Problem | Rating | Slot | Main forms |
|---|---|---|---|---|
| 01 | cf1077f2 Pictures with Kittens (hard) | 2100 | window DP (max) | `Window(max)`, `[0..=n]Int` layers, `max(gen)!` |
| 02 | cf1195e OpenStreetMap | 2100 | window (min), 2D | `Window(min)` row then column pass, `[R, C]0..1e9` |
| 03 | cf1249e By Elevator or Stairs? | 1700 | (vertex, state) shortest path | `dijkstra((v, k) => moves(v, k), from: [((s, m), 0)])`, `p.dist[(v, m)]` |
| 04 | cf1326d2 Prefix-Suffix Palindrome (hard) | 1800 | string + `s.rev` | `prefix_function(Str(chain(mid, "#", mid.rev)))`, multitest |
| 05 | cf1548b Integers Have Friends | 1800 | two pointers | `d.maximal(state: Window(gcd), ok: ...)`, multitest |
| 06 | cf1200d White Lines | 1900 | grid rows/cols + generator max | `g.rows.keys`, `g[r]`, `g.cols.keys`, `g[.., c]`, `max(max(... for j)! for i)!` |
| 07 | cf1059d Nature Reserve | 2200 | float, F64 sentinel | `var best: F64 = 1e18`, `best min= f`, ternary search, `print(x, precision: 9)` |
| 08 | cf1475g Strange Beauty | 1900 | multitest + shared precomputation | divisor lists above `@!multitest`, `cnt`/`dp` arrays below |
| 09 | cf1092f Tree with Maximum Cost | 1900 | tree DP, rerooting | `Rooted(g, root:)`, `t.reroot(Monoid.of(each(...)), lift:, finish:)` |
| 10 | cf1312d Count the Arrays | 1700 | combinatorics `Mod<P>` | `type Z = Mod<998244353>`, `Comb<Z>(m)`, `Z(2).pow(e)` |

Every slot was filled cold. After the tools ran, two slots are only partly filled in
the requested form: 05 hand-rolls the two pointers (FL9, FL10), and 06 reads cells
as `g[r, c]` with no `g.rows` / `g.cols` or row views (FL6, FL7). Skip list checked: none of the ten IDs appears in a file or
folder name under `plan/domains/spec/delta/` or `research/rebuild-rounds/`.

## Counts

| Class | Count | Inside M1 subset |
|---|---|---|
| silent | 12 | 12 |
| contradiction | 3 | 3 |
| missing-stdlib | 1 | 0 |
| found-late | 16 | 15 (all but FL13) |

M1: every `spec/SUBSET.md` section touched (R01-R07, R10, std-window, std-seq,
std-strings, std-graph, std-tree, std-grid, std-mod) is marked supported, so every
hole except H16 (no row covers F64 math functions) is inside the subset.

## Holes

"Guess held" was filled in after the tools ran. Silent guesses: 7 held (H5-H8, H12, H14, H15), 4 failed (H4, H9, H10, H13), 1 partly (H11).

| ID | Class | M1 | Where | Hole | Cold guess | Guess held |
|---|---|---|---|---|---|---|
| H1 | contradiction | in (R07.3, R04b) | 04 | R07.5.3/R04b say a reversed view is not indexable, so `prefix_function(s.rev)` (needs `Indexed`) cannot type; R04b's helper section says a `[K]Int` helper "works unchanged on `a`, on `a[l..=r]` and on `a.rev`". | `.rev` is not `Indexed`; materialize with `Str(chain(mid, "#", mid.rev))`. | No: `prefix_function(s.rev)` compiles and runs (`"abcab".rev` gives `0 0 0 1 2`); R07.5.3 is stale. The copy in 04 was unnecessary. |
| H2 | contradiction | in (std-seq) | 05 | `maximal` doc: "debug traps when `ok` fails on a single element after a shrink"; its body has no such trap and lets the window empty. cf1548b has single differences of 1 (gcd 1), so the doc reading would trap on valid input. | No trap; an empty window is `ok` (`(w.all ?? 0) != 1`) and its piece has `len` 0. | Untested: `Window(gcd)` (FL9) and `maximal` over any `Window` (FL10) fail first. |
| H3 | contradiction | in (std-window) | 01, 02 | `Window` doc and `spec/lib/sequence-patterns.md` say selective ops (`min`, `max`) keep a monotone deque; the struct fields (`q`, `front`, `back`) read like a two-stack queue for every op. | Either way `push`/`pop`/`all` are O(1) amortized; correctness unaffected. | Contradiction confirmed: generated `Window_Int_Op_Max` holds `q`, `front`, `back` (two stacks), no deque. Speed fine (01 max input 0.04 s). |
| H4 | silent | in (std-window) | 05 | Whether `Window(gcd)` accepts a plain function value (no operator, no `Monoid`) as `op`. | Accepted like `Window(min)`. | No (FL9). |
| H5 | silent | in (R04b.1.12) | 02, 06 | Element indexing of `[R, C]T`: spec shows `g[r]` and `g[.., c]` but not the cell form. | `g[r, c]`. | Yes. |
| H6 | silent | in (R04.4) | 01 | `var prev: [0..=n]Int = NEG` (anonymous range array, scalar fill, `Int` keys). | Allowed, keys checked. | Yes. |
| H7 | silent | in (R02.8) | 05 | Array builder over an anonymous range: `[0..n - 1] i => abs(a[i + 1] - a[i])`, then `.maximal` on it. | Builds `[0..n-1]Int`; `maximal` takes `[D]T`. | Yes. |
| H8 | silent | in (R10) | 03 | Printing an array value built inline: `print([V] v => ...)`. | One line, space separated. | Yes. |
| H9 | silent | in (R03.1) | 07 | `F64(int)` conversion, `-1e7` / `1e18` as F64 initializers, `2.0` literal. | All allowed. | No (FL8). |
| H10 | silent | in (R02.8) | 08 | `var divs: [X]List<X> = List()` fills every key with its own empty list. | Each key gets a fresh list. | No (FL11). |
| H11 | silent | in (std-mod) | 10 | `Comb<Z>(m)` positional limit; `Z(2).pow(e)` method form; `if n == 2: 0 else: <Z>` with expected type `Z`. | All type-check; the `0` converts to `Z`. | Partly: `Comb<Z>(m)` and the `0` to `Z` held; `.pow` did not (FL12). |
| H12 | silent | in (R06.1.7) | 03 | Top-level `fn moves` reading top-level `let`s `a`, `b`, `c`. | Allowed (captures must be initialized first). | Yes. |
| H13 | silent | in (R07) | 04 | `Str(chain(mid, "#", mid.rev))`: `chain` over a `Str` piece, a literal and a reversed view, collected into `Str`. | Works; O(n). | No for 3-argument `chain` (FL5); `chain(chain(mid, "#"), mid.rev)` works. |
| H14 | silent | in (R01.7, R02.5) | 06 | `if let pi = i.prev and let pj = j.prev:` and tuple destructuring `for (i1, i2, j1, j2) in rects`. | Both allowed. | Yes. |
| H15 | silent | in (std-tree) | 09 | `Rooted(g, root:)` with a `Graph` (examples use `[V]List<V>`); `max(r.cost for r in at)!` over `[V]` records. | Both accepted. | Yes. |
| H16 | missing-stdlib | out (no row) | 07 | No F64 `sqrt` (or other float math) in root `std/`; R09.3.6 mentions "`sqrt` of a negative" but nothing defines it. Avoided by squaring the formula. | n/a (avoided) | Confirmed. |

## Top five risks

After the tools ran:

1. FL7: row and column views key by 0-based `Int`, so natural grid code is silently off by one on a `1..=n` grid.
2. FL10: no `Window` works with `maximal` / `minimal`, the spec's headline two-pointer example.
3. FL4: the documented `dijkstra((v, k) => ...)` product-state form does not type.
4. FL8: the brief's `var best: F64 = 1e18` sentinel is rejected.
5. FL13: `cpl test` cannot judge float output without a hand-written `@check`.

Cold (before any tool ran):

1. H1: the spec contradicts itself on whether `.rev` can feed `Indexed` code; the
   requested "read `s.rev` in a prefix-function" shape forces a copy.
2. H2: the `maximal` trap doc would make the natural gcd two-pointer trap on valid input.
3. H4: non-operator function values as `Window` ops are unspecified.
4. H5: the 2D cell index form is never shown next to `g[r]` / `g[.., c]`.
5. H3: if `Window(min)` is a two-stack queue, it costs about 2-3x a monotone deque on hot loops (01: 25M pushes).

## Revision test

| File | Change | Lines changed | Chunks | Hesitation / duplicated fact / unrelated rewrite |
|---|---|---|---|---|
| 01-1077F2-rev.cp (DP) | "exactly x reposts" becomes "at most x" | 3 (+1 `var best`, +1 `best max=` in the layer loop, -1 final `let best`), plus the comment | 1 (the answer moves into the layer loop) | No hesitation; the window code is untouched. |
| 03-1249E-rev.cp (graph) | elevator overhead `c` paid only on the first boarding | 5 (`domain Mode = 0..3`, one extra `else if` yield arm, the weight select flips to `k == Mode(1)`, the print takes a `min` over two modes), plus the comment | 2 (state domain + moves fn; output) | Hesitation on the mode numbering: `Mode(0)`/`Mode(1)` are magic numbers repeated in `moves` and `print`; an enum would name them, but `dijkstra` over `(V, Enum)` products is not shown in the spec, so I kept a numeric domain (duplicated fact). |

## Found late (after the cold commit)

Every one needed a change to the committed cold code or to the test procedure. Repros are one line unless noted.

| ID | M1 | Files | Failure | Minimal repro | Workaround |
|---|---|---|---|---|---|
| FL1 | in (R01.3) | 01, 06, 08 | E0208: an uppercase `let` reads as a type | `let NEG = 1` | `const NEG` / lowercase name |
| FL2 | in (R07) | 01, 05-09 | E0303: there is no one-argument `max(seq)`; the brief's `max(e for c in Col)!` form does not exist | `print(max(x for x in [1, 2])!)` | `seq.max()!` / `.map(...).max()!` |
| FL3 | in (R01.7) | 03-rev | E0503: `else if` is spelled `elif` | `if a: f() else if b: g()` | `elif` |
| FL4 | in (known gap `core-06/lambda-unpacks-tuple-vertex.cp`) | 03, 03-rev | E0302: `dijkstra((v, k) => f(v, k), ...)`, the form `spec/lib/graphs-trees.md` shows, does not type; the spec's own conformance file fails the same way | `cpl run spec/conformance/core-06/lambda-unpacks-tuple-vertex.cp` | `dijkstra(s => moves(s.0, s.1), ...)` |
| FL5 | in (R07) | 04 | E0303: `chain` takes exactly 2 sequences | `chain(a, b, c)` | `chain(chain(a, b), c)` |
| FL6 | in (known gaps `core-04/grid-rows-*`) | 06 | E0203: `[R, C]T` has no `rows` / `cols` (R04b.1.12) | `domain R = 0..2; domain C = 0..3; let g: [R, C]Int = 0; print(g.rows.len)` | loop over `R` / `C` |
| FL7 | in (R04b.1.12) | 06 | Silent wrong answer: `g[r].keys` and `g[.., c].keys` are 0-based `Int`, not `C` / `R`, so `Int(key)` is off by one on a `1..=n` grid (sample 1 printed 2, expected 4); `g[r][c]` with `c: C` is E0302 | `domain R = 1..=2; domain C = 1..=3; let g = read<[R, C]Char>(); print(g[R.first!].keys.find(c => g[R.first!][c] == 'B') ?? -1)` on `WBW/BWW` prints `1`, not `2` | read cells as `g[r, c]` over `C.find` / `R.find` |
| FL8 | in (R03.1) | 07 | E0302: `var best: F64 = 1e18` (the brief's exact sentinel) is rejected, `1e18` and `1.0e18` are `Int`; `1e7.0` parses as a member access | `var best: F64 = 1e18` | `F64(1e18)` |
| FL9 | in (std-window) | 05 | E0201: `Window(gcd)` fails inside std.window ("doesn't know a fn named `gcd`"); `Window(Monoid<Int>.of(gcd))` and `Window(Monoid(id: 0, op: ...))` fail with "`op`" | `var w = Window(gcd); w.push(4)` | hand-rolled distinct-gcd list per right end |
| FL10 | in (std-seq, std-window) | 05 | E0303: `a.maximal(state: Window(+), ...)`, the headline example of `spec/lib/sequence-patterns.md`, fails: `Window.pop()` takes 0 arguments, `Track.pop(x)` 1, so `Window` is not a `Track` | `[1, 2, 3, 4].maximal(state: Window(+), ok: w => (w.all ?? 0) <= 3)` | hand-rolled (same as FL9) |
| FL11 | in (R02.8) | 08 | E0302: a list-valued fill `var d: [X]List<X> = List()` | `domain X = 1..=3; var d: [X]List<X> = List()` | `[X] _ => List<X>()` |
| FL12 | in (std-mod) | 10 | E0203: `z.pow(e)` is rejected ("power uses `**`") though `std/mod.cp` declares `fn pow` | `type Z = Mod<7>; print(Z(2).pow(3))` | `Z(2) ** (n - 3)` |
| FL13 | out (tooling) | 07 | `cpl test` compares floats token-exactly: `0.625` vs `0.625000000` fails, 1/3 samples | `cpl test 07-1059D.cp` | judged samples and random tests with a 1e-6 checker outside `cpl test` |
| FL14 | in (explain) | all | `cpl explain` reports a phantom `1:1 explicit copy -> std::vector<Char>` and `1:1 reverse view` in every file (none of 01-03, 05-10 has a string), and does not report 01's per-layer `var cur: [0..=n]Int` heap allocation inside the `for _ in 1..=x` loop | `cpl explain 01-1077F2.cp` | read the C++ |
| FL15 | in (interp) | 08 | The interpreter takes about 25 s per tiny test because the shared 2e5 divisor lists are rebuilt each run (native: 0.14 s on the max input) | `echo "1 1 1" \| cpl run 08-1475G.cp` | random tests sharded 8 ways |
| FL16 | in (R05) | 05 | E0402: `swap(a, b)` needs `swap(mut a, mut b)` (spec rule, missed cold) | `var a = [1]; var b = [2]; swap(a, b)` | `mut` at the call |

## Verification

| Check | Result |
|---|---|
| `cpl check` (12 files) | clean after FL1-FL12, FL16 |
| samples (interpreter, native `-O2`, sanitizer build) | all pass: 01 3/3, 02 1/1, 03 2/2, 04 1/1 (checker: any valid palindrome), 05 1/1, 06 5/5, 07 3/3 (1e-6), 08 1/1, 09 2/2, 10 4/4; both revs pass their samples against the revised brute |
| `cpl test` | 9/10 files pass every case; 07 1/3 (FL13) |
| `cpl build` + `g++ -std=c++17 -O2` | all 12 build; max inputs: every file under 0.2 s and 62 MB |
| random, 200 per file, vs Python brute, interpreter = native | all 12 files 200/200 (08 run in 8 shards of 25 because of FL15) |
| `g++ -std=c++17 -O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG` on samples and every random input | no reports |
| `cpl explain` | no heap copy in a loop reported; FL14; dynamic `std::function` in 03 (Dijkstra heap comparator, 2) and 09 (3) |
