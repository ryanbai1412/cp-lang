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

Every slot is filled. Skip list checked: none of the ten IDs appears in a file or
folder name under `plan/domains/spec/delta/` or `research/rebuild-rounds/`.

## Counts (cold)

| Class | Count | Inside M1 subset |
|---|---|---|
| silent | 12 | 12 |
| contradiction | 3 | 3 |
| missing-stdlib | 1 | 0 |
| found-late | 0 (filled in after the tools ran, below) | - |

M1: every `spec/SUBSET.md` section touched (R01-R07, R10, std-window, std-seq,
std-strings, std-graph, std-tree, std-grid, std-mod) is marked supported, so every
hole except H16 (no row covers F64 math functions) is inside the subset.

## Holes

"Guess held" is filled in after the tools ran.

| ID | Class | M1 | Where | Hole | Cold guess | Guess held |
|---|---|---|---|---|---|---|
| H1 | contradiction | in (R07.3, R04b) | 04 | R07.5.3/R04b say a reversed view is not indexable, so `prefix_function(s.rev)` (needs `Indexed`) cannot type; R04b's helper section says a `[K]Int` helper "works unchanged on `a`, on `a[l..=r]` and on `a.rev`". | `.rev` is not `Indexed`; materialize with `Str(chain(mid, "#", mid.rev))`. | |
| H2 | contradiction | in (std-seq) | 05 | `maximal` doc: "debug traps when `ok` fails on a single element after a shrink"; its body has no such trap and lets the window empty. cf1548b has single differences of 1 (gcd 1), so the doc reading would trap on valid input. | No trap; an empty window is `ok` (`(w.all ?? 0) != 1`) and its piece has `len` 0. | |
| H3 | contradiction | in (std-window) | 01, 02 | `Window` doc and `spec/lib/sequence-patterns.md` say selective ops (`min`, `max`) keep a monotone deque; the struct fields (`q`, `front`, `back`) read like a two-stack queue for every op. | Either way `push`/`pop`/`all` are O(1) amortized; correctness unaffected. | |
| H4 | silent | in (std-window) | 05 | Whether `Window(gcd)` accepts a plain function value (no operator, no `Monoid`) as `op`. | Accepted like `Window(min)`. | |
| H5 | silent | in (R04b.1.12) | 02, 06 | Element indexing of `[R, C]T`: spec shows `g[r]` and `g[.., c]` but not the cell form. | `g[r, c]`. | |
| H6 | silent | in (R04.4) | 01 | `var prev: [0..=n]Int = NEG` (anonymous range array, scalar fill, `Int` keys). | Allowed, keys checked. | |
| H7 | silent | in (R02.8) | 05 | Array builder over an anonymous range: `[0..n - 1] i => abs(a[i + 1] - a[i])`, then `.maximal` on it. | Builds `[0..n-1]Int`; `maximal` takes `[D]T`. | |
| H8 | silent | in (R10) | 03 | Printing an array value built inline: `print([V] v => ...)`. | One line, space separated. | |
| H9 | silent | in (R03.1) | 07 | `F64(int)` conversion, `-1e7` / `1e18` as F64 initializers, `2.0` literal. | All allowed. | |
| H10 | silent | in (R02.8) | 08 | `var divs: [X]List<X> = List()` fills every key with its own empty list. | Each key gets a fresh list. | |
| H11 | silent | in (std-mod) | 10 | `Comb<Z>(m)` positional limit; `Z(2).pow(e)` method form; `if n == 2: 0 else: <Z>` with expected type `Z`. | All type-check; the `0` converts to `Z`. | |
| H12 | silent | in (R06.1.7) | 03 | Top-level `fn moves` reading top-level `let`s `a`, `b`, `c`. | Allowed (captures must be initialized first). | |
| H13 | silent | in (R07) | 04 | `Str(chain(mid, "#", mid.rev))`: `chain` over a `Str` piece, a literal and a reversed view, collected into `Str`. | Works; O(n). | |
| H14 | silent | in (R01.7, R02.5) | 06 | `if let pi = i.prev and let pj = j.prev:` and tuple destructuring `for (i1, i2, j1, j2) in rects`. | Both allowed. | |
| H15 | silent | in (std-tree) | 09 | `Rooted(g, root:)` with a `Graph` (examples use `[V]List<V>`); `max(r.cost for r in at)!` over `[V]` records. | Both accepted. | |
| H16 | missing-stdlib | out (no row) | 07 | No F64 `sqrt` (or other float math) in root `std/`; R09.3.6 mentions "`sqrt` of a negative" but nothing defines it. Avoided by squaring the formula. | n/a (avoided) | |

## Top five risks

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
