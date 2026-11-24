# The standard library stubs

Every public module of the standard library, written in CP Lang and built into `cpl`: signatures,
doc comments, costs and bodies. The library chapters in `spec/lib/` give the model and the
examples; these files are the contract an implementation is held to.

## What every public fn documents

Two people implementing the same stub must build the same function. A doc comment starts with a
summary in prose, then a blank `///` line, then labelled lines, one fact each:

```
/// The key of the least element: `[3, 1, 2].argmin()` is `Some(1)`.
///
/// cost: O(n)
/// empty: `None`
/// ties: the first in key order
/// traps: never
/// example: `[3, 1, 2].argmin() == Some(1)`
/// example: `[5, 5].argmin() == Some(0)`
pub fn argmin<S: Keyed, K, T: Ord>(self: S) -> ?K = todo
```

| Label | Required | Says |
|---|---|---|
| `cost:` | always | time, and extra space when it is not O(1); per step for generators |
| `traps:` | always | every input that traps, or `never`. A violated precondition is an `assert` in the body and reports R0401; a core trap (an index, overflow, `x!`) keeps its own code |
| `example:` | always, one or more | an expression `==` its value, so it can become a test |
| ```` ```cp ```` block | for anything that needs setup (a graph, a tree, a table) | a multi-line program; its last line is asserted unless it is a statement (`let`, `print`, ...). Lead with what the item computes (its defining equation or a brute force), then a recognisable contest example |
| `empty:` | when an input is a sequence, range, domain or container | the result, or the trap, on empty input |
| `ties:` | when the fn chooses among equal candidates (min, max, sort, search, first, nearest) | which one wins, and whether a sort is stable |
| `order:` | when it returns a sequence | the order of the output, and whose keys it carries |
| `see:` | optional | the neighbour to use instead (`sort` for `sorted`) |

Struct methods document the same way; a struct's own doc says its invariant and its layout.
`scripts/spec-stub.py --audit` lists every entry missing a required label; `--check` fails when
a file has more such entries than `doc-baseline.json` records, so the count only goes down.
Lower the baseline in the same commit that fixes entries (`--audit --write-baseline`).

## Surface index (migrated from plan/STDLIB.md)

# The standard library: the complete design

Status: the design the stdlib lane builds toward. `plan/20-stdlib.md`
keeps the catalogue of what contests need and the reasoning behind the
v1 scope; this file is the surface: every module, every public item with
its signature and cost, the naming rules they follow, and the core
features each one relies on. `plan/domains/stdlib/` says where we
stand against it. The 0.3 spec chapters 20 to 29 and 40 were the seed;
where this file departs from them it says why, and `decisions.md` logs
the call.

Every item is marked:

- **[have]** exists in `stdlib/` today with tests;
- **[new]** designed here, not written yet;
- **[rename]** exists under another name, which keeps working as a
  deprecated alias until a codemod moves the ports;
- **[local]** lives in `examples/local-lib/` today and is promoted here.

## Who owns what

Ten area owners each own their part of section 4, write its design in
`spec/lib/<area>.md` and build it; the coordinator owns sections 1
to 3 (principles, naming, core features), the prelude budget (4.1), `std.spec`
(4.15) and section 5, reviews every area design before it lands against the
checklist in `spec/lib/rationale/README.md`, and routes each stdlib friction
in `research/frictions/` to the owner below. An item that spans two
areas belongs to the area of the type it returns; a new intrinsic or
core feature goes through section 3 and the core lane.

| Area | Owns | Done and next |
|---|---|---|
| sequences | 4.2 except the searches; the walk folds' library side; a `Seq`/`Indexed` trait | done: `_by` twins to `by:` with L0510, `.items` as the one enumerate, `Range` span traps that keep `bsearch_*` exact at any width, user `impl Ord` in the interpreter's `sorted`/`min`/`max`, `.len` typed so `0..xs.len` infers (decision 60), method `inverse` in 5 ports, `range:` on `reverse`/`rotate_*` (debt until mutable slice views, decision 76), `scan_in_place` with the mutator-vs-copy naming rule (decision 76.2), `argmax`/`prefix_n` in 5 ports, one `prefix(a)` with L0513 (decision 80), writable-from-docs fixes. `distinct`, `fold(f)`, `sort(cmp:)`, `reach(while:, cyclic:)` (`cf1284e`), `[D]T.map` keeping the domain (decision 104). Next: corpus audit for hand-rolled `argmax`/runs/compress/`next_permutation` with codemods, `by:` on `distinct`/`min`/`max` |
| DP/search | 4.2 searches (`spec/lib/rationale/search.md`); `std.dp` | done: `monge_path` and generic `aliens` in `cf2183h` (decisions 56, 65), online LARSCH and filled `val`/`prev` (decision 74, `cf2183h` 0.9x to 1.4x the original), `std.slope` concave/convex slope trick (decision 81, `cf2084g2`), lint `L0520` hand-rolled answer search with `cpl fix` (51 corpus loops, 0 false positives, 42 port sites moved), `List.reserve` on the LARSCH level stacks (`cf2183h` seed 2 0.40 to 0.29 s), `if let` payloads read in place, dense enum and `?enum` `@memo` keys, `D.cuts` as a range of cuts, matched C++17 benches for every entry point (`bash bench/dp/run.sh`: p50 0.85x, max 1.20x), 48-byte `LazyTreap` nodes (decision 122) and in-place tag pushes (`std.slope` 1.37x to 1.24x), a 4-byte `knuth` split table (1.24x to 1.11x). Next: 4-byte `?(0..2^31)` child links once flows' bounded optional lands, a gated port for Li Chao or Knuth when one appears |
| number theory/combinatorics | 4.3 to 4.7 (`math`, `mod`, `ntt`, `matrix`, `rand`, hashing) | done: `Comb<M>` (decision 55), 32-bit `Mod<P>` by ranged-field narrowing (plus `U32`-field non-negativity proving raw `%` in `Mod::mul`), `cf2245f` 7.3x to 0.64x. `std.poly` (decision 69) at 1.06x matched C++, L0415 loop-invariant `Mod` divisor lint (decision 75). `eval`, `div_mod`, `prod_linear`, `taylor_shift`, `eval_all` (0.78x), `compose`, negative `pow`; `lib.math.poly` retired from ports. Generic `fwht`/`convolve_xor`/`convolve_subset`, `lagrange_at`/`interpolate`, one `factor` for every `Int` (decision 92), range-typed store checks (decision 100), 32-bit ranged array/2D/nested/tuple elements (decisions 106, 115), CSR `Adj` from pair/triple graphs (decision 110: `root_bench` 0.76x time, 1.14x memory), `divmod`, and `bench/math` matched C++17 suite (`p50=1.09x`, `max=1.17x`, 4.7–16.0 KB emitted; codegen-only notes in `research/frictions/bench-math.md`) |
| containers | 4.8, 4.9 (`bitset`, `bag`, `deque`, `heap`, `dsu`, `marks`, `window`), `List` methods | done: heaps (`min_heap` 0.92x), potential DSU, 4 B/vertex single-array `Dsu` (`dsu` 1.15x wall / 1.10x user), `FlatMap` (0.95x), inline bounded `Bitset` (0.96x, decision 62), open-addressing `HashMap`/`HashSet` (0.64x–0.79x `gp_hash_table`, decision 85), `Map.keys`/`.values` O(1) views, `SortedSet` + `IntervalMap` in prelude (decisions 113, 117), `Set`/`Map.range(r)` zero-alloc `Iter::TreeRange` (`ordered_map_set` 0.96x), packed `BagNode` single-pass `SortedBag`/`SortedSet` (`sorted_range` 0.61x, `sorted_set` 1.26x vs PBDS `rb_tree_tag`), `drain(after:/before:, while:)` across `Set`/`SortedSet`/`SortedBag` via external parameter labels (`cf2262b`), single-iterator `IntervalMap.assign` (`interval_map` 0.56x, `cf2237h` 0.514 s / 0.321 s, decision 105), raw `List<T>` `Deque` + `Deque.reserve` (`deque` 1.14x), `List.reserve`, constant-length `std::array` lowering, `TimeTree.run`/`offline_over` (`cf2069f` RSS 356 to 146 MB), `Bitset.subset_sums` (`cf2187d` 4.3x faster), `Dsu.groups()`/`roots`/`values` (`cf2245e` 0.50 to 0.13 s); 13 matched C++17 benches (`bash bench/containers/run.sh`: p50 0.95x, max 1.26x) |
| range structures | 4.10 | done: `Fenwick.first_ge` (decision 57), constant-init `LazySegTree`, `LazyTreap` (decision 68, matched 1.06x), `Flip`/`flip_action` with devirtualized combinators (decision 73), `LazySegTree` over `Cut<D>` (`cf2207f`), `cf2057d` 2.01x to 1.12x. Next: the 5 open high frictions, all core/codegen-side (`?Struct` niche, `MinHeap.pop`) |
| graphs | 4.11 `std.graph`, 4.15 `std.grid` | done: `until:` early exit (46.11), hole-based heap sift, one `Paths.back` witness array (decision 78; matched `dijkstra` 1.44x to 1.18x), `cf2135c` on a named component domain. Next: `fixed_cost_paths(until:)` and CSR (held for a green rebase), then `cf2178h`; the rest of `dijkstra` (dead-field elimination in codegen); `divmod` and named edge tuples (cf2035f dream) |
| trees | 4.11 `std.tree` | done: `LinkCut` (1.01x matched), `cf2237h` on `IntervalMap`. vertex loops as `for v in t.at` / `t.at.rev` with L0511 (decision 63). `Rooted.below` (`cf2183f` 0.44x tokens). `JumpTree` for trees grown online (`cf2152f`). `PermTree` (`cf2187f1`/`f2`, one construction instead of two copies). Label-ordered parent arrays stay loops (`cf2138c1`, `cf2187c`, `cf2089e`; design §5). `Rooted` builds breadth-first with flat child ranges (1.1x matched, decision 86). `LazyLinkCut` with `apply_path` (at most 1.0x matched). `Rooted.virtual` with its own domain (decision 108, 0.78x–1.06x matched), 7 hand-rolled ports moved onto `Rooted.from_parents`/`centroid`/`virtual`. `t.up(+, finish:)` (cf2035f), `Tree` in the prelude (decision 119), L0208 for a `use` the prelude covers. Tree dreams cf2035f, cf2124i, cf2183d2, cf2183f, cf2237i1 check clean. Next: cf2135f waits on `PMultiset`; PathPaint waits for a second problem |
| flows/matching | 4.12 | done: weighted blossom `max_weight_general_matching` (1.27x matched), unchecked hot loops in `std.flow` (min-cost flow 1.14x). Next: matched C++ benches for closure, bounded flow, Gomory–Hu and global cut |
| strings | 4.13; O(1) slice and reverse views | done: slice/reverse views (decision 61), `.rev` views, full `Indexed`/`Seq` generality (`List<T>`, `[D]T`, `ListView`, `Str`, `StrView`), `lyndon_runs`, `SuffixAutomaton.first_pos`, `L0516`–`L0519` hand-rolled algorithm lints, all 10 matched C++17 benches (`bash bench/strings/run.sh`, p50 0.87x, max 1.11x; `Hash2D` 32-bit tables 0.88x, `RollingHash` 0.58x), 0 open strings dream gaps, 0 high strings frictions |
| geometry | 4.14 | done: `Location`, `Vec.rot(by:)`, Shamos–Hoey, 3D hull, heterogeneous `Mul`/`Div` (`cf2138f`), `min_bounding_box`, `circle_tangents`, `circle_circle_tangents`, `circles_union_area`, `sqrt` negative trap, 16 matched C++17 benches all <=1.09x (`locate` 2.43x to 0.97x), 0 geometry dream gaps; lines continue after a trailing binary operator (docs-written compile 94.5%). Next: open geometry frictions; `cf2183f`'s fill-order sums are `Prefix.filling` (ranges) |

## 1. Principles

The stdlib is ordinary CP Lang. A module may use only what a user library
may use: the public language, the documented intrinsics (section 3), and
other stdlib modules. If a module needs a power the language lacks, the
power becomes a core feature for everyone or the module waits.

What belongs here, in the end, is everything ever asked at the hardest
contests (ICPC World Finals, IOI, AtCoder WTF, Codeforces Div. 1), each
as fast as the best known C++ library implementation on a matched
benchmark. The order of arrival follows the ports: `examples/local-lib/`
is the staging area, written on top of this library the way a
contestant's own would be, and a module is promoted when it meets the
bar here (documented costs, tests, a port or example, the naming rules)
and the ports reach for it.

Each public item has a `///` comment saying what it does, its cost in
time and, where it matters, in memory. `cpl doc` prints them; the cost is
part of the contract, and a change that makes a cost worse is a breaking
change.

The obvious call is the correct and fast one. A function that looks O(1)
is O(1); anything O(n) or allocating says so in its name or its type
(`sorted` returns a new list, `sort` sorts in place).

Nothing is chosen secretly. `Bitset<D>` is a type you pick, never what a
`[D]Bool` silently becomes; `convolve` names NTT in its doc and its cost,
and a different algorithm is a different function.

## 2. Naming conventions

One name for one idea, everywhere. These rules decide every name below;
a name that breaks one is a bug in this file.

**Types are nouns in `UpperCamel`, functions and methods verbs or nouns
in `snake_case`.** A type's constructor is its name called like a
function: `Sieve(n)`, `Dsu<V>()`, `Rng(seed)`. `T.new(...)` is the
declaration form and the explicit spelling.

**A fold or sort of one sequence is a method.** `xs.sum()`, `xs.max()`,
`xs.count(p)`, `xs.all(p)`, `xs.first(p)`, `xs.sorted(by: k)`,
`xs.sort(by: k)`. The free form is for math: a generator
(`sum(f(x) for x in xs)`) or two or more values (`min(a, b)`). `find(x)`
is the index of a value; the element a test holds for is `first(p)`.
Lint L0509 (`free_fold`) rewrites the rest (decisions.md §53).

**Options are named arguments, not `_by` variants.** `runs(a, by: key)`,
`a.sort(by: key)`, `r.first_true(ok)`. Today's `runs_by`,
`first_ge_by` and `last_le_by` are [rename]s to `runs(a, by:)`,
`first_ge(xs, x, by:)` and `last_le(xs, x, by:)`.

**`r.first_true(ok)`/`r.last_true(ok)` are the O(log n) monotone form of
`first`/`last`** (spec R04.8.3). `first(xs, p)` scans: O(n) calls of any `p`.
`r.first_true(ok)` asks O(log n) questions and needs `ok` false then
true along `r` (`last_true`: true then false); both answer `?` a
position of the space searched. Reach for the scan unless the predicate
is monotone and the range is large.

**Fallible operations return `?T`; the trapping twin has the short
name.** `m[k]` traps when `k` is absent and `m.get(k)` is `?V`;
`x.inv` traps on a non-invertible residue and `x.try_inv()` is
`?Mod<P>`. A fallible twin is named `try_<verb>` unless a established
name exists (`get`, `pop`, `first`, `find`).

**Mutator vs copy twins.** Where mutation is the default, the mutator
takes the bare verb and the copy takes the participle (`a.sort()` /
`a.sorted()`, `reverse(mut a)` / `a.rev`). Where the pure copy is the
established default verb (`scan`, `suffix_scan`), the zero-allocation
mutating twin uses `_in_place` (`a.scan_in_place(f)`,
`a.suffix_scan_in_place(f)`).

**Collection verbs.** Every container speaks the same verbs:

```
s.add(x)        // insert into a set-like container; returns whether it was new
s.remove(x)     // returns whether it was there (maps: the removed ?V)
s.toggle(x)     // flips membership
x in s          // membership; s.contains(x) is the method form
s.len           // size, a getter, never len()
s.is_empty
s.clear()       // O(1) or O(capacity), documented per type
s.take()        // moves the contents out and leaves s empty
```

Sequences `push` and `pop` at the back, `push_front` and `pop_front` at
the front; `first` and `last` read the ends as `?T`.

**Range structures.** `t.get(i)`, `t.set(i, x)`, `t.update(i, f)` for a
point; `t.fold(r)` for a range under the structure's monoid, addition
included (`Fenwick`, `Prefix` and their 2D forms fold by `+`); `t.apply(r, f)` for a lazy action; `t.all` for
the fold of everything; `t.max_right(from, ok)` and `t.min_left(to, ok)`
for searches, returning boundaries.

**Sorted lookup.** One four-name family everywhere a sorted order exists
(sorted lists, `[D]T`, `Map`, `SortedBag`, `Compressed`):

```
first_ge(x)    // first element >= x
first_gt(x)    // first element >  x
last_le(x)     // last element  <= x
last_lt(x)     // last element  <  x
```

Sequences return the position, ordered collections the element. The 0.3
spec spells these `first(from: x)`, `first(after: x)`, `last(upto: x)`
and `last(before: x)`; we keep the four names because they read the
same in every container, are greppable, and need no overloading on
argument labels, which the core does not have (decision 26.1).

**Number-theory and algorithm names are the textbook names.**
`gcd`, `ext_gcd`, `crt`, `pow_mod`, `is_prime`, `factor`, `phi`,
`convolve`, `z_function`, `prefix_function`, `suffix_array`,
`manacher`, `dijkstra`, `scc`, `max_flow`, `convex_hull`. A contestant
types the name from the editorial.

**Costs in the name when two costs exist.** `factor(x)` is trial
division, O(sqrt x); `Sieve(n).factor(x)` is O(log x) after an O(n)
sieve; `factor_big(x)` is Pollard's rho for x up to 4e18. Each doc says
which to reach for.

**Costs are written one way.** A trailing `// O(...)` on the signature
here and the same in the `///` comment: time first, then memory when
it is not O(1) extra (`// O(n log n), O(n) memory`). Amortized and
expected bounds say so in words after the bound (`// O(alpha(n)) amortized`,
`// O(n) expected`); `n` is the structure's size unless the comment
names another variable.

**Index spaces.** A function over positions takes and returns the
domain (`?D`, `Cut<D>`, `Range<D>`), never a raw `Int`, when its
input is a `[D]T`; over a `List<T>` it uses `Int`. A function that
creates a new index space returns it (`scc` returns `domain C`). `D.cuts` is
the value `Range<D.cuts>`, the cuts `0..=D.K`: pass it where a
`Range` or `List` of cuts is expected; `D.all` is the positions.

## 3. Core features the stdlib relies on

The stdlib needs no compiler knowledge beyond the following, each a
public feature user libraries get too.

Intrinsics (functions not written in the language): `popcount`, `ctz`,
`bit_width`, `bit_ceil`, `isqrt`, `sqrt`, `atan2`, `sin`, `cos`, `tan`, `asin`, `acos`, `exp`, `ln`, `hypot`, `floor`, `trunc`,
`ceil`, `round`, `rand`, `flush`, `read`, `print`, `debug`, `sort`,
`sorted`, `swap`, `min`, `max`, `abs`, `sum`, `prod`, `count`, `all`,
`any`, `first`. The folds take a walk, `count(x > 2 for x in xs)`; the
four that fold a test (`count`, `all`, `any`, `first`) also take one
after the sequence, `count(xs, p)` and the method `xs.count(p)`, which
is the same walk with `p`'s body as its body (an inline lambda costs
nothing, a named function is one call per element). `sum`, `prod`,
`min` and `max` of a function stay `sum(f(x) for x in xs)`: one
spelling per idea (spec 22.2). The 0.3 spec adds
`mulhi`, `checked_*`, `wrapping_*`, `overflowing_*` and `clock`
[new, core lane].

Language features, with the modules that need them:

- generics with type parameters that are values (`Mod<P>` with a
  constant or a read `P`): `mod`, `ntt`, `matrix`, `spec`;
- operator traits (`Add`, `Sub`, `Mul`, `Div`, `Neg`, `Show`) and the
  associated constants `T.zero`, `T.one`: `mod`, `matrix`, `fenwick`,
  `prefix`, `geometry`;
- operation values (`Monoid`, `Action`, functions as values): `segtree`,
  `lazy`, `sparse`, `algebra`;
- domains, `Cut<D>`, `Range<D>` and fresh domains returned from a
  function (existential returns): `graph`, `tree`, `seq.compress`,
  `strings.suffix_array` [0.3: fresh `Rank`];
- generators (`yields`): `bitset.each`, `graph.adj`, `tree.tour`,
  `seq.windows`;
- interior mutability in a `let` value (`Comb`), `lends mut`
  projections (`matrix.at`) and their write-back (`SegTree.at`,
  `LazySegTree.at`: `st.at(i) += x` pulls the ancestors), `@get` getters;
- `@test` functions run by `cpl test stdlib`, and `laws_hold` for
  algebraic laws.

Still missing from the core and wanted here, owned by the core lane:
variadic `print`/`read` for the `Read`/`Show` derives, a niche trait so
user optionals are 8 bytes, and overloading on argument labels (would let the
four-name family become `first(after:)` if we ever want it).

## 4. The modules

Each module below opens with what it is for, then its full public
surface as declared (the [have] block is generated from `stdlib/*.cp`
by reading each `pub` line and the `O(...)` phrases in its `///`
comment, so it matches the source at the commit that last touched this
file), then the [new] and [rename] items it still owes.

### 4.1 `std.prelude`

The names every program sees without a `use`: the 80-name budget is
checked by `scripts/check.sh` (decision 21.5). A name earns a slot when
ports and dreams write it without a `use`; a name only library authors
write (`Compressed`, `Reversible`) gives its slot up, and a name joins
only when another leaves. Everything else is one `use` away, and the compiler's missing-name hint names the module (it is read
off the files by `build.rs`, so a new `pub fn` needs no second edit).

### 4.2 `std.seq`: sequences, sorted lookup, search on answers

Free functions over `List<T>`, `[D]T`, and `Str`, the four-name sorted-lookup
family, prefix/suffix/exclusive scans, permutations, monotone-stack and
sliding-window helpers, coordinate compression, and binary/ternary searches.
Full design and contest survey in `spec/lib/rationale/seq.md`. The searches return
the boundary, not a found flag: `(lo..hi).first_true(ok)` is the first `i`
with `ok(i)` for a monotone `ok`, or `None`.

```cp
type Perm<V> = [V]V
fn scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T        // O(n)
fn bsearch_first<I>(r: Range<I>, ok: I -> Bool) -> ?I  // O(log n)
fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I   // O(log n)
fn min_by<S, T, K>(it: S, key: T -> K) -> ?T
fn max_by<S, T, K>(it: S, key: T -> K) -> ?T
fn reverse<T>(mut a: List<T>)
fn dedup<T>(mut a: List<T>)
fn deduped<T>(self: List<T>) -> List<T>
fn distinct<S: Seq, T>(self: S) -> List<T>  // O(n) expected, O(n) space
fn partition<T>(self: List<T>, keep: T -> Bool) -> (List<T>, List<T>)
fn windows<T>(self: List<T>, k: Int) -> yields List<T>  // O(k)
fn pairs<T>(self: List<T>) -> yields (T, T)
fn with_prev<T>(self: List<T>) -> yields (?T, T)
fn cyclic_pairs<T>(self: List<T>) -> yields (T, T)
fn zip<A, B>(a: List<A>, b: List<B>) -> List<(A, B)>
fn get_or_insert<K, V>(mut self: Map<K, V>, k: K, v: V) -> V  // O(log n)
// O(log n)
fn get_or_insert_with<K, V>(mut self: Map<K, V>, k: K, f: fn() -> V) -> V
fn update<K, V>(mut self: Map<K, V>, k: K, init: V, f: fn(V) -> V)  // O(log n)
fn rotate_left<T>(mut a: List<T>, k: Int)
fn next_permutation<T>(mut a: List<T>) -> Bool  // O(n), O(1)
fn prev_permutation<T>(mut a: List<T>) -> Bool  // O(n), O(1)
fn runs<T>(a: List<T>) -> List<(val: T, len: Int)>
fn runs_by<T, K>(a: List<T>, by: T -> K) -> List<(key: K, items: List<T>)>
fn excl_fold<T, M>(self: List<T>, m: M) -> yields (x: T, rest: T)  // O(n)
struct Compressed<D, T>(vals: [D]T)
    fn get<K>(self, i: K)                   // O(1), O(r.len)
    fn slice(self, r: Range<D>) -> List<T>  // O(r.len)
    @get fn len(self) -> Int
    fn at(self, val: T) -> D         // O(log n)
    fn first_ge(self, val: T) -> ?D  // O(log n)
    fn last_le(self, val: T) -> ?D   // O(log n)
fn compress<T>(all: List<T>)         // O(n log n)
@get fn l<D>(self: Range<D>) -> Cut<D>
@get fn r<D>(self: Range<D>) -> Cut<D>
@get fn ends<D>(self: Range<D>) -> (Int, Int)
fn first_ge_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int  // O(log n)
fn last_le_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int   // O(log n)
fn swap_at<D, T>(mut self: [D]T, i: D, j: D)
fn resized<T>(self: List<T>, n: Int, fill: T) -> List<T>  // O(n)
fn take<T>(mut self: List<T>) -> List<T>                  // O(1)
fn insert<T>(mut self: List<T>, i: Int, x: T)             // O(n)
fn remove<T>(mut self: List<T>, i: Int) -> T              // O(n)
@get fn argmax<D, T>(self: [D]T) -> ?D                    // O(n)
@get fn argmin<D, T>(self: [D]T) -> ?D                    // O(n)
```

[new] Designed in `spec/lib/rationale/seq.md`:

```cp
// Reductions and predicates
fn prod<S, T>(it: S) -> T                                 // O(n)
fn is_sorted<D, T>(self: [D]T) -> Bool                    // O(n)
fn is_sorted_by<D, T, K>(self: [D]T, by: T -> K) -> Bool  // O(n)
fn argmax_by<D, T, K>(self: [D]T, by: T -> K) -> ?D       // O(n)
fn argmin_by<D, T, K>(self: [D]T, by: T -> K) -> ?D       // O(n)

// Selection, merging, and O(1)-space rotation
fn nth_element<T>(mut a: List<T>, k: Int)                // expected O(n), O(1) space
fn merge_sorted<T>(a: List<T>, b: List<T>) -> List<T>    // O(n + m)
fn rotate_right<T>(mut self: List<T>, k: Int)            // O(n), O(1) space
fn rotated_left<S: Seq, T>(self: S, k: Int) -> List<T>   // O(n)
fn rotated_right<S: Seq, T>(self: S, k: Int) -> List<T>  // O(n)

// Zero-allocation runs/chunks and domain bucketing
fn chunk_by<D, T>(self: [D]T, same: (T, T) -> Bool) -> yields Range<D>  // O(n), O(1) space
fn run_ranges<D, T>(self: [D]T) -> yields Range<D>                      // O(n), O(1) space
fn run_ranges_by<D, T, K>(self: [D]T, by: T -> K) -> yields Range<D>    // O(n), O(1) space
fn buckets<D, K>(key: [D]K) -> [K]List<D>                               // O(D.count + K.count)
fn chunks<T>(self: List<T>, k: Int) -> yields List<T>                   // O(n)

// Prefix and suffix scans over elements (cut-indexed prefix_fold/suffix_fold live in std.prefix)
fn suffix_scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T         // O(n)
fn scan_in_place<T>(mut self: List<T>, f: (T, T) -> T)         // O(n), O(1) extra space
fn suffix_scan_in_place<T>(mut self: List<T>, f: (T, T) -> T)  // O(n), O(1) extra space

// Coordinate compression completeness
fn Compressed.contains(self, val: T) -> Bool    // O(log n)
fn Compressed.try_index_of(self, val: T) -> ?D  // O(log n)
fn Compressed.first_gt(self, val: T) -> ?D      // O(log n)
fn Compressed.last_lt(self, val: T) -> ?D       // O(log n)

// Permutations
fn inverse<I, V>(p: [I]V) -> [V]I              // O(n)
fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C  // O(n)
fn perm_cycles<V>(p: [V]V) -> List<List<V>>    // O(n)
fn perm_pow<V>(p: [V]V, k: Int) -> [V]V        // O(n)
fn inversions<T>(a: List<T>) -> Int            // O(n log n)

// Binary/ternary search and four-name sorted lookup
// (search on answers: spec/lib/rationale/dp.md; each returns a position of its
// range's index space or none, never a sentinel)
fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64
// steps + 1 calls of ok; the least x in [lo, hi] with ok(x), to within
// (hi - lo) / 2^steps; none when not ok(hi)
fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I  // O(log n) calls of f
fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I  // O(log n) calls of f
// a position of the optimum of f on r, for f strictly rising then
// falling (plateaus only at the optimum); none for an empty r
fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
// steps + 2 calls of f (golden section); within (hi - lo) * 0.618^steps
fn parallel_bsearch<I>(
    queries: Int,
    r: Range<I>,
    reset: () -> (),
    apply: I -> (),
    ok: Int -> Bool,
) -> List<?I>
// O((r.len + queries) log r.len) calls; out[q] is the first t with
// ok(q) after apply on every t' <= t, ok monotone in the prefix
fn first_gt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D  // O(log n)
fn last_lt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D   // O(log n)

// Monotone stack: nearest smaller (or, with `by: x => -x`, greater) neighbour
fn prev_smaller<D, T, K>(self: [D]T, by: T -> K = x => x, strict: Bool = true) -> [D]?D  // O(n)
fn next_smaller<D, T, K>(self: [D]T, by: T -> K = x => x, strict: Bool = true) -> [D]?D  // O(n)
```
```

[rename] `runs_by(a, key)` is superseded in hot paths by zero-allocation
`a.run_ranges(by: key)` (and ultimately `runs(a, by: key)`);
`first_ge_by`, `last_le_by`, `argmax_by`, `argmin_by` collapse into
`first_ge(.., by:)`, `last_le(.., by:)`, `argmax(by:)`, `argmin(by:)` once
builtins and getters accept optional `by:`.

### 4.3 `std.math`: number theory

Integer helpers and number theory with every cost in the name or the
type. Full design and contest survey: `spec/lib/math.md`. Two
ways to factor: `Sieve(n).factor(x)` in O(log x) for many x up to n,
and `factor(x)` for any 64-bit `Int` (trial division for `x <= 1e6`,
Pollard's rho + Miller-Rabin above).

```cp
fn gcd(a: Int, b: Int) -> Int  // O(log min(a, b))
fn lcm(a: Int, b: Int) -> Int
fn fixed(num: Int, den: Int, k: Int) -> Str
fn choose(n: Int, k: Int) -> Int                        // O(min(k, n - k))
fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int)  // O(log min(a, b))
fn ceil_div(a: Int, b: Int) -> Int                      // O(1)
fn divmod(self: Int, d: Int) -> (Int, Int)              // O(1); Euclidean (self / d, self % d), 0 <= r < |d|, self == q * d + r; works as n.divmod(d) or divmod(n, d)
struct Sieve                                            // O(n)
    fn new(n: Int) -> Sieve
    fn is_prime(self, x: Int) -> Bool
    fn factor(self, x: Int) -> List<(Int, Int)>  // O(log x)
    fn phi(self, x: Int) -> Int                  // O(log x)
    fn mobius(self, x: Int) -> Int               // O(log x)
fn bit_len(x: Int) -> Int
fn digits(self: Int, base: Int = 10) -> List<Int>
@get fn sign(self: Int) -> Int
fn bit(self: Int, i: Int) -> Bool
@get fn is_pow2(self: Int) -> Bool
fn splitmix64(x: U64) -> U64
fn factor(x: Int) -> List<(Int, Int)>  // O(sqrt x) for x <= 1e6; O(x^(1/4) log x) expected above
fn divisors(x: Int) -> List<Int>       // O(sqrt x)
fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int)
fn mul_mod(a: Int, b: Int, m: Int) -> Int  // O(1), through I128 if needed
fn pow_mod(a: Int, e: Int, m: Int) -> Int  // O(log e); 1 <= m
fn is_prime(x: Int) -> Bool                // O(log^2 x); deterministic Miller-Rabin, every Int
fn phi(x: Int) -> Int                      // O(sqrt x) for x <= 1e6; O(x^(1/4) log x) expected above
fn primitive_root(p: Int) -> Int           // O(p^(1/4) log p) expected, prime p
fn floor_sum(n: Int, m: Int, a: Int, b: Int) -> Int
// O(log m); sum of (a*i + b) / m for i in 0..n, floor division
```

[new] Additions from `spec/lib/math.md`:

```cp
fn choose2(n: Int) -> Int                                     // O(1); n * (n - 1) / 2, 0 for n < 2
fn inv_mod(a: Int, m: Int) -> ?Int                            // O(log m); m >= 1, none if gcd(a, m) != 1
fn crt_all(eqs: List<(r: Int, m: Int)>) -> ?(r: Int, m: Int)  // O(k log M)
fn mod_log(a: Int, b: Int, m: Int) -> ?Int                    // O(sqrt m); BSGS discrete log
fn mod_sqrt(a: Int, p: Int) -> ?Int                           // O(log^2 p); Tonelli-Shanks, prime p
fn divisors_from_factors(fs: List<(Int, Int)>) -> List<Int>   // O(d(x) log d(x))
fn Sieve.divisors(self, x: Int) -> List<Int>                  // O(d(x) log d(x)), sorted
fn Sieve.phi_table(self) -> List<Int>                         // O(n)
fn Sieve.mobius_table(self) -> List<Int>                      // O(n)
fn sieve_multiplicative<T>(
    n: Int,
    one: T,
    mul: (T, T) -> T,
    prime_pow: (Int, Int, Int) -> T,
) -> List<T>  // O(n) time and memory
```

Relies on: `I128` (inside `mul_mod`), the builtin `isqrt`.

### 4.4 `std.mod`: modular arithmetic

`Mod<P>` is an integer reduced mod `P`, where `P` is a constant or a
value read at startup; `Comb<M>` is keyed on the `Mod` type (`type Z = Mod<P>;
let comb = Comb<Z>(limit)`) so the modulus is written once and factorial tables
grow on demand (decision 52).

```cp
struct Mod<P>(v: 0..P)  // O(1), O(log P)
    fn new(x: Int) -> Mod<P>
    // Power is `a ** e` via `impl Pow for Mod<P>` (O(log e))
    fn try_inv(self) -> ?Mod<P>  // O(log P)
    @get fn inv(self) -> Mod<P>
struct Comb<M>  // O(1)
    fn new(limit: Int = 0) -> Comb<M>
    fn fact(self, n: Int) -> M
    fn inv_fact(self, n: Int) -> M
    fn inv(self, i: Int) -> M  // O(1)
    fn choose(self, n: Int, k: Int) -> M
    fn arrange(self, n: Int, k: Int) -> M
    fn multichoose(self, n: Int, k: Int) -> M
    fn multinomial(self, ks: List<Int>) -> M  // O(ks.len)
    fn catalan(self, n: Int) -> M
```

### 4.5 `std.ntt` [new]: convolution

The number-theoretic transform, polynomial products (linear and cyclic),
and bitwise/SOS convolutions, promoted from
`examples/local-lib/math/poly.cp` (imported as `lib.math.poly` by nine
port files) and `examples/local-lib/math/sos.cp`. Full design:
`spec/lib/math.md`. The formal power-series operations stay in the
local library until wider corpus demand justifies promotion.

```cp
fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false, bit_reversed: Bool = false)
// O(n log n) in place; a.len a power of two dividing P - 1.
// With bit_reversed: true, output is in DIF/DIT bit-reversed frequency
// order and only an inverse transform taking the same flag may consume it.
fn convolve<P>(a: List<Mod<P>>, b: List<Mod<P>>, cyclic: Bool = false) -> List<Mod<P>>
// O((n + m) log(n + m)); P - 1 divisible by a power of two at least
// n + m - 1 (998244353, 167772161, 469762049, 754974721 qualify);
// schoolbook below 64 terms
fn convolve_mod(a: List<Int>, b: List<Int>, m: Int, cyclic: Bool = false) -> List<Int>
// three NTT primes and Garner CRT; any 1 <= m < 2^31
fn convolve_int(a: List<Int>, b: List<Int>, cyclic: Bool = false) -> List<Int>
// exact signed integer convolution when every output fits in about +-2^62
enum SosOver = Sub | Sup
fn sos<T>(mut a: List<T>, over: SosOver = Sub, inverse: Bool = false)
// O(n log n); subset (Sub) or superset (Sup) zeta/Mobius transform in place
fn fwht<T>(mut a: List<T>, inverse: Bool = false)
// O(n log n); in-place Walsh-Hadamard (XOR) transform on Int, Mod<P>, Float
fn convolve_or<T>(a: List<T>, b: List<T>) -> List<T>      // O(n log n)
fn convolve_and<T>(a: List<T>, b: List<T>) -> List<T>     // O(n log n)
fn convolve_xor<T>(a: List<T>, b: List<T>) -> List<T>     // O(n log n); Int, Mod<P>, Float
fn convolve_subset<T>(a: List<T>, b: List<T>) -> List<T>  // O(k^2 * 2^k) for n = 2^k
```

Relies on: `Mod<P>` with a type-parameter modulus, `primitive_root`,
`bit_ceil`, `ctz`.

#### `std.poly` [new]: formal power series over NTT

Operates on plain `List<Mod<P>>` (lowest degree first) over an NTT-friendly
prime `P`. Full cost breakdown and laws: `spec/lib/math.md` §2.5.

```cp
fn deriv<P>(a: List<Mod<P>>) -> List<Mod<P>>                                           // O(n)
fn integral<P>(a: List<Mod<P>>) -> List<Mod<P>>                                        // O(n); linear-time inverses
fn inv<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>                                     // O(n log n); a[0] != 0
fn log<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>                                     // O(n log n); a[0] == 1
fn exp<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>                                     // O(n log n); a[0] == 0
fn pow<P>(a: List<Mod<P>>, k: Int, n: Int) -> List<Mod<P>>                             // O(n log n); supports k < 0 when a[0] != 0
fn sqrt<P>(a: List<Mod<P>>, n: Int) -> ?List<Mod<P>>                                   // O(n log n + log^2 P)
fn eval<P>(a: List<Mod<P>>, x: Mod<P>) -> Mod<P>                                       // O(n); Horner
fn div_mod<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> (q: List<Mod<P>>, r: List<Mod<P>>)  // O(n log n)
fn prod_linear<P>(roots: List<Mod<P>>) -> List<Mod<P>>                                 // O(n log^2 n); prod (x - r)
fn taylor_shift<P>(a: List<Mod<P>>, c: Mod<P>) -> List<Mod<P>>                         // O(n log n); a(x + c)
fn eval_all<P>(a: List<Mod<P>>, xs: List<Mod<P>>) -> List<Mod<P>>                      // O((n + m) log^2 m)
fn compose<P>(f: List<Mod<P>>, g: List<Mod<P>>, n: Int) -> List<Mod<P>>                // O(n log^2 n); f(g(x)) mod x^n
fn lagrange_at<P>(ys: List<Mod<P>>, x: Mod<P>) -> Mod<P>                               // O(n); value at x of deg < n poly through (i, ys[i])
fn interpolate<P>(xs: List<Mod<P>>, ys: List<Mod<P>>) -> List<Mod<P>>                  // O(n log^2 n) (O(n^2) for n <= 64); unique deg < n poly
```

### 4.6 `std.matrix` [new]: matrices, Gaussian elimination and linear basis

`Mat<T>` is a dense flat row-major matrix over any `T` with `T.zero`,
`T.one`, `+`, `-`, `*` (`Int`, `Mod<P>`, `F64`). Exact field elimination
(`det`, `rank`, `solve`, `solve_space`, `inverse`) lives on `Mat<T>` for
exact fields such as `Mod<P>` with prime `P`; partial-pivoted `F64`
helpers (`det_real`, `rank_real`, `solve_real`, `inverse_real`) take a
named tolerance `eps: F64 = 1e-9`. `XorBasis` covers GF(2) linear bases
with a `Monoid` for segment trees. Full design: `spec/lib/math.md`.

```cp
struct Mat<T>(n: Int, m: Int, a: List<T>)
    fn zero(n: Int, m: Int = n) -> Mat<T>       // O(n m)
    fn identity(n: Int) -> Mat<T>               // O(n^2)
    fn of(rows: List<List<T>>) -> Mat<T>        // O(n m)
    fn get(self, i: Int, j: Int) -> T           // O(1)
    fn at(self, i: Int, j: Int) -> lends mut T  // O(1)
    // Power is `a ** e` via `impl Pow for Mat<T>` (O(n^3 log e))
    fn apply(self, v: List<T>) -> List<T>   // O(n m)
    fn transpose(self) -> Mat<T>            // O(n m)
    fn rows(self) -> List<List<T>>          // O(n m)
    fn det(self) -> T                       // O(n^3), T: Div
    fn rank(self) -> Int                    // O(n m min(n, m))
    fn solve(self, b: List<T>) -> ?List<T>  // O(n m min(n, m))
    fn solve_space(self, b: List<T>) -> ?(x0: List<T>, basis: List<List<T>>)
    fn inverse(self) -> ?Mat<T>                    // O(n^3)
    fn product(n: Int) -> Monoid<Mat<T>>           // for segment trees
fn det_real(a: Mat<F64>, eps: F64 = 1e-9) -> F64   // O(n^3)
fn rank_real(a: Mat<F64>, eps: F64 = 1e-9) -> Int  // O(n m min(n, m))
fn solve_real(a: Mat<F64>, b: List<F64>, eps: F64 = 1e-9) -> ?List<F64>
fn inverse_real(a: Mat<F64>, eps: F64 = 1e-9) -> ?Mat<F64>
struct XorBasis
    fn new() -> XorBasis                    // O(1)
    @get fn rank(self) -> Int               // O(1)
    fn add(mut self, x: Int) -> Bool        // O(log x)
    fn reduce(self, x: Int) -> Int          // O(62)
    fn contains(self, x: Int) -> Bool       // O(62)
    fn max_xor(self, init: Int = 0) -> Int  // O(62)
    fn min_xor(self, init: Int) -> Int      // O(62)
    fn kth(self, k: Int) -> ?Int            // O(62^2)
    fn merge(mut self, o: XorBasis)         // O(rank * 62)
    fn monoid() -> Monoid<XorBasis>         // for segment trees
```

Relies on: operator traits, `T.zero` and `T.one`, `lends mut`.

### 4.7 `std.rand` and hashing

A seeded SplitMix64 stream: deterministic, so a failing stress case
replays. Hashing of values is the language's (`Hash` on every key
type); string hashing is `std.strings.RollingHash`.

```cp
struct Rng(s: U64)  // O(1)
    fn new(seed: Int) -> Rng
    fn next(mut self) -> U64
    fn int(mut self, lo: Int, hi: Int) -> Int
    fn float(mut self) -> F64
    fn shuffle<T>(mut self, mut a: List<T>)  // O(n)
```

[new]:

```cp
fn pick<T>(mut self, xs: List<T>) -> T            // O(1); traps on empty
fn chance(mut self, p: F64) -> Bool               // O(1)
fn perm(mut self, n: Int) -> List<Int>            // O(n), uniform
fn sample(mut self, n: Int, k: Int) -> List<Int>  // O(k log k), distinct,
// sorted, from 0..n
fn hash_mix(h: U64, x: U64) -> U64  // O(1), splitmix finalizer
// over h ^ x; for keyed hashes
```

### 4.8 `std.bitset`: bitsets

`Bitset<D>` packs a set of positions of `D` into 64-bit words, the type
for O(n/64) set algebra and shift DP. It is chosen by name; a `[D]Bool`
is never silently packed. The 0.3 spec's open name `Bits<D>` would
clash with the existing `Bits` trait for integer widths, so `Bitset`
stays (decision 26.3).

```cp
struct Bitset<D>  // O(1), O(K / 64)
    fn new() -> Bitset<D>
    fn full() -> Bitset<D>
    fn from(a: [D]Bool) -> Bitset<D>
    fn contains(self, i: D) -> Bool
    fn add(mut self, i: D)
    fn remove(mut self, i: D)
    fn toggle(mut self, i: D)
    @get fn len(self) -> Int
    @get fn count(self) -> Int
    @get fn is_empty(self) -> Bool
    fn union(mut self, o: Bitset<D>)
    fn intersect(mut self, o: Bitset<D>)
    fn subtract(mut self, o: Bitset<D>)
    fn symmetric(mut self, o: Bitset<D>)
    fn bitor(self, o: Bitset<D>) -> Bitset<D>
    fn bitand(self, o: Bitset<D>) -> Bitset<D>
    fn bitxor(self, o: Bitset<D>) -> Bitset<D>
    fn meets(self, o: Bitset<D>) -> Bool
    fn shifted(self, k: Int) -> Bitset<D>
    fn each(self) -> yields D
    fn next(self, i: D) -> ?D
```

[new] (see `spec/lib/containers.md`):

```cp
fn with(self, i: D) -> Bitset<D>                 // O(K / 64), non-mutating copy
fn without(self, i: D) -> Bitset<D>              // O(K / 64), non-mutating copy
fn clear(mut self)                               // O(K / 64)
@get fn first(self) -> ?D                        // O(K / 64)
@get fn last(self) -> ?D                         // O(K / 64)
fn prev(self, i: D) -> ?D                        // O(K / 64); last member < i
fn count_below(self, i: D) -> Int                // O(K / 64); count < i
fn shift_up(mut self, k: Int)                    // O(K / 64) in place, k >= 0
fn shift_or(mut self, k: Int)                    // O(K / 64) in place: self |= self << k
fn find(self, pred: D -> Bool) -> ?D             // O(K / 64 + visited)
fn subset_sums(weights: List<Int>) -> Bitset<D>  // O(K sqrt(S) / 64) (associated on Bitset<D>)
```

### 4.9 Containers: `std.bag`, `std.interval`, `std.deque`, `std.heap`, `std.dsu`, `std.marks`, `std.hashmap`

See `spec/lib/containers.md` for the full pattern survey, laws, and
worked contest examples. `SortedBag<K>` is an ordered multiset with order
statistics, monotone search, recycled treap nodes, and O(1)-amortized
cursors (`BagCursor`, `entries`, `entries_rev`) and O(1) `.range(r)` views; `SortedSet<K>` is its
unique-key twin; `IntervalMap<D, T>` (`std.interval`) maintains a maximal-run
partition of domain `D` under range assignment; `Deque<T>`, `MonoDeque<T>`,
and `TwoStackFold<T>` cover ring buffers and sliding-window queries;
`MinHeap`/`MaxHeap`, `ErasableMinHeap`/`ErasableMaxHeap`, and
`IndexedMinHeap`/`IndexedMaxHeap` cover binary, lazy-delete, and domain-keyed
heaps; `Dsu<V>`, `DsuOf<V, T>`, `ParityDsu<V>`, `PotentialDsu<V>`, and
`RollbackArray<D, T>` share nominal `Mark` rollback; `Marks<D>`,
`MarkMap<D, T>`, and `Interner<K>` cover O(1)-clear epoch sets/maps and
online interning; `FlatMap<V>` (`std.hashmap`) provides open-addressing
salted hashing for `Int` keys.

The union-finds and `RollbackArray` step back in time the same way: `mark()`
returns a `Mark`, its own type, and `undo_to(mark)` undoes every change since,
values and parities included, O(1) per change. One mark serves several
returns to the same point (`cf2222f` undoes both halves of its divide and
conquer to one `snap`). `DsuOf`'s join is `join(value of a's set, value
of b's set)` for `unite(a, b)` whatever root union by size keeps, so
`(_, b) => b` is a directional "b wins" (`cf2237h`, skip-to-previous
bucket) and `(a, b) => a ?? b` keeps the first representative
(`cf2222f`). Scoped rollback, so an early `return` cannot skip
`undo_to`, belongs to the core's proposed `defer`/`undo` block (`plan/14`);
a closure helper would need the body to mutate the structure while it is
lent, which lexical borrowing rejects. Frictions:
`research/frictions/cf2222f.md`, `research/frictions/cf2237h.md`.

```cp
struct BagCursor(id: Int)
struct SortedBag<K> // O(log n), O(1), O(n)
    fn new() -> SortedBag<K>
    fn of(xs: List<K>) -> SortedBag<K> // O(n log n)
    @get fn len(self) -> Int // O(1)
    @get fn distinct(self) -> Int // O(1) [new]
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> SortedBag<K> // O(1)
    fn add(mut self, x: K) -> Bool // O(log n)
    fn add_many(mut self, x: K, c: Int) // O(log n) [new]
    fn remove(mut self, x: K) -> Bool // O(log n)
    fn remove_all(mut self, x: K) -> Int // O(log n) [new]
    fn count(self, x: K) -> Int // O(log n)
    fn contains(self, x: K) -> Bool // O(log n)
    @get fn first(self) -> ?K // O(log n)
    @get fn last(self) -> ?K // O(log n)
    fn pop_first(mut self) -> ?K // O(log n)
    fn pop_last(mut self) -> ?K // O(log n)
    fn range(self, r) -> SortedRange<K> // O(1) view; .first/.last/.len/.is_empty O(log n), iter/.rev O(log n + k)
    fn rank(self, k: K) -> Int // O(log n)
    fn nth(self, i: Int) -> ?K // O(log n)
    fn bsearch_first(self, ok: K -> Bool) -> ?K // O(log n), monotone ok (boundary-checked)
    fn bsearch_last(self, ok: K -> Bool) -> ?K // O(log n), monotone ok (boundary-checked)
    fn cursor_first(self) -> ?BagCursor // O(log n) [new]
    fn cursor_last(self) -> ?BagCursor // O(log n) [new]
    fn cursor_at(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_ge(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_gt(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_le(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_lt(self, k: K) -> ?BagCursor // O(log n) [new]
    fn key_of(self, c: BagCursor) -> K // O(1) [new]
    fn count_of(self, c: BagCursor) -> Int // O(1) [new]
    fn next_cursor(self, c: BagCursor) -> ?BagCursor // O(1) amort [new]
    fn prev_cursor(self, c: BagCursor) -> ?BagCursor // O(1) amort [new]
    fn entries(self) -> yields (key: K, count: Int) // O(n) [new]
    fn entries_rev(self) -> yields (key: K, count: Int) // O(n) [new]
    fn to_list(self) -> List<K> // O(n)
struct SortedSet<K> // [new] unique-key twin of SortedBag<K>
    fn new() -> SortedSet<K>
    fn of(xs: List<K>) -> SortedSet<K> // O(n log n)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> SortedSet<K> // O(1)
    fn add(mut self, x: K) -> Bool // O(log n)
    fn remove(mut self, x: K) -> Bool // O(log n)
    fn set(mut self, x: K, on: Bool) // O(log n)
    fn contains(self, x: K) -> Bool // O(log n)
    @get fn first(self) -> ?K // O(log n)
    @get fn last(self) -> ?K // O(log n)
    fn pop_first(mut self) -> ?K // O(log n)
    fn pop_last(mut self) -> ?K // O(log n)
    fn range(self, r) -> SortedRange<K> // O(1) view; .first/.last/.len/.is_empty O(log n), iter/.rev O(log n + k)
    fn rank(self, k: K) -> Int // O(log n)
    fn nth(self, i: Int) -> ?K // O(log n)
    fn bsearch_first(self, ok: K -> Bool) -> ?K // O(log n), monotone ok (boundary-checked)
    fn bsearch_last(self, ok: K -> Bool) -> ?K // O(log n), monotone ok (boundary-checked)
    fn each(self) -> yields K // O(n)
    fn each_rev(self) -> yields K // O(n)
    fn to_list(self) -> List<K> // O(n)
struct IntervalMap<D, T> // [new] std.interval; O(log runs) amortized
    fn new(init: T) -> IntervalMap<D, T> // O(1)
    fn from(a: [D]T) -> IntervalMap<D, T> // O(D.K)
    fn at(self, i: D) -> T // O(log runs)
    fn run_at(self, i: D) -> (range: Range<D>, val: T) // O(log runs)
    fn assign(mut self, r: Range<D>, val: T, on_old: (Range<D>, T) -> () = (_, _) => ())
    fn assign_point(mut self, i: D, val: T) -> T // O(log runs)
    fn each(self) -> yields (range: Range<D>, val: T) // O(runs)
    @get fn runs(self) -> Int // O(1)
struct Deque<T> // O(1)
    fn new() -> Deque<T>
    fn of(xs: List<T>) -> Deque<T> // O(n)
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> Deque<T> // O(1)
    fn push(mut self, x: T)
    fn push_front(mut self, x: T)
    fn pop(mut self) -> ?T
    fn pop_front(mut self) -> ?T
    @get fn first(self) -> ?T
    @get fn last(self, k: Int
    @get fn items(self) -> List<(Int, T)> // O(n)
    fn get(self, i: Int) -> T
    fn set(mut self, i: Int, x: T) // O(1) [new]
    fn to_list(self) -> List<T> // O(n) [new]
struct MonoDeque<T> // O(1) amortized sliding min/max (std.window)
    fn new(by: (T, T) -> T = min) -> MonoDeque<T> // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
    fn clear(mut self) // O(1)
    fn push(mut self, at: Int, val: T) // O(1) amortized
    fn drop_before(mut self, min_at: Int) // O(1) amortized
    fn drop_after(mut self, max_at: Int) // O(1) amortized
    @get fn best(self) -> ?T // O(1)
    @get fn best_at(self) -> ?Int // O(1)
struct TwoStackFold<T> // [new] O(1) amortized queue monoid fold
    fn new<M>(op: M) -> TwoStackFold<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
    @get fn fold(self) -> T
struct MinHeap<T> // O(log n), O(1)
    fn new() -> MinHeap<T>
    fn of(xs: List<T>) -> MinHeap<T> // O(n) [new]
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn peek(self) -> ?T
    fn clear(mut self)
    fn take(mut self) -> MinHeap<T> // O(1)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
struct MaxHeap<T>
    fn new() -> MaxHeap<T>
    fn of(xs: List<T>) -> MaxHeap<T> // O(n) [new]
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn peek(self) -> ?T
    fn clear(mut self)
    fn take(mut self) -> MaxHeap<T> // O(1)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
struct ErasableMinHeap<T> // [new] O(log n) amortized lazy-delete heap
    fn new() -> ErasableMinHeap<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> ErasableMinHeap<T> // O(1)
    fn push(mut self, x: T)
    fn remove(mut self, x: T)
    fn peek(mut self) -> ?T
    fn pop(mut self) -> ?T
struct ErasableMaxHeap<T> // [new] largest-first twin of ErasableMinHeap
struct IndexedMinHeap<D, T> // [new] O(log n) domain-keyed decrease/update-key
    fn new() -> IndexedMinHeap<D, T> // O(D.K)
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn contains(self, k: D) -> Bool // O(1)
    fn get(self, k: D) -> ?T // O(1)
    fn set(mut self, k: D, val: T) // O(log n)
    fn remove(mut self, k: D) -> ?T // O(log n)
    @get fn peek(self) -> ?(key: D, val: T) // O(1)
    fn pop(mut self) -> ?(key: D, val: T) // O(log n)
struct IndexedMaxHeap<D, T> // [new] largest-first twin of IndexedMinHeap
struct Mark(at: Int)
struct Dsu<V> // O(alpha(n)), O(log n)
    fn new() -> Dsu<V> // O(n)
    fn find(mut self, v: V) -> V
    fn mark(mut self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone unite
    fn unite(mut self, a: V, b: V) -> Bool
    fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V)
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    fn roots(self) -> yields V // O(n), zero allocations
    fn groups(mut self) -> List<List<V>> // O(n alpha(n))
struct DsuOf<V, T> // as Dsu, plus one join per merge; three [V] arrays
    fn new(init: (V) -> T, join: (T, T) -> T) -> DsuOf<V, T> // O(n)
    fn find(mut self, v: V) -> V
    fn get(mut self, v: V) -> T
    fn set(mut self, v: V, x: T)
    fn unite(mut self, a: V, b: V) -> Bool
    fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V)
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    fn roots(self) -> yields V // O(n), zero allocations
    fn values(self) -> yields T // O(n), zero allocations
    fn groups(mut self) -> List<List<V>> // O(n alpha(n)) [new]
    fn mark(mut self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone merge or set
struct ParityDsu<V> // O(log n), no path shortening; three [V] arrays
    fn new() -> ParityDsu<V> // O(n)
    fn find(self, v: V) -> (root: V, flip: Bool)
    fn unite(mut self, a: V, b: V, differ: Bool = true) -> Bool // false: contradiction
    fn diff(self, a: V, b: V) -> ?Bool
    fn same(self, a: V, b: V) -> Bool
    fn size(self, v: V) -> Int
    @get fn consistent(self) -> Bool
    fn mark(self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone unite
struct PotentialDsu<V> // [new] O(alpha(n)) / O(log n) after mark
    fn new() -> PotentialDsu<V> // O(n)
    fn find(mut self, v: V) -> (root: V, pot: Int)
    fn unite(mut self, a: V, b: V, diff: Int) -> Bool
    fn diff(mut self, a: V, b: V) -> ?Int
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    @get fn consistent(self) -> Bool
    fn mark(mut self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone unite
struct RollbackArray<D, T> // [new] O(1) get/set/mark, O(changes) undo
    fn new(init: [D]T) -> RollbackArray<D, T> // O(D.K)
    fn filled(val: T) -> RollbackArray<D, T> // O(D.K)
    fn get(self, i: D) -> T // O(1)
    fn set(mut self, i: D, val: T) // O(1)
    fn mark(self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone set
struct Marks<D> // O(1)
    fn new() -> Marks<D> // O(D.K), O(1)
    fn add(mut self, v: D) -> Bool // O(1)
    fn remove(mut self, v: D) -> Bool // O(1)
    fn contains(self, v: D) -> Bool // O(1)
    fn clear(mut self) // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool
struct MarkMap<D, T> // [new] O(1) epoch-stamped sparse array over D
    fn new(default: T) -> MarkMap<D, T> // O(D.K)
    fn get(self, v: D) -> T // O(1)
    fn set(mut self, v: D, x: T) // O(1)
    fn contains(self, v: D) -> Bool // O(1)
    fn remove(mut self, v: D) -> Bool // O(1)
    fn clear(mut self) // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
struct Interner<K> // [new] online key <-> 0..len id
    fn new() -> Interner<K> // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
    fn id(mut self, k: K) -> Int // O(log n)
    fn get(self, k: K) -> ?Int // O(log n)
    fn key(self, idx: Int) -> K // O(1)
    @get fn keys(self) -> List<K> // O(1)
struct FlatMap<V> // [new] std.hashmap; O(1) expected open-addressing Int map
    fn new(cap: Int = 16) -> FlatMap<V> // O(cap)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
    fn clear(mut self) // O(table_size)
    fn contains(self, k: Int) -> Bool // O(1) expected
    fn get(self, k: Int) -> ?V // O(1) expected
    fn set(mut self, k: Int, v: V) // O(1) expected
    fn remove(mut self, k: Int) -> ?V // O(1) expected
    fn each(self) -> yields (key: Int, val: V) // O(table_size)
```

### 4.10 Range structures

Modules `std.algebra`, `std.fenwick`, `std.prefix`, `std.segtree`,
`std.lazy`, `std.sparse`, plus [new] `std.window`, `std.mo`, and advanced
structures (`std.persistent`, `std.beats`; `LiChao` is in `std.dp`, `MergeSortTree` in `std.segtree`).
Full design, contest survey, laws and exemplars: `spec/lib/rationale/ranges.md`.

The algebra is values, not traits: a `Monoid<T>` is an identity and an
operation, an `Action<T, F>` a monoid of tags acting on one of values.
`laws_hold` checks the laws on samples in tests. The 0.3 spec moves
these to traits; that waits for trait-associated constants on
user-declared traits and stays a [rename] candidate (decision 26.4).

```cp
struct Monoid<T>(id: T, op: (T, T) -> T)
    fn of<M>(m: M) -> Monoid<T>
struct Action<T, F>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: Monoid<T>)
    fn new<M>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: M) -> Action<T, F>
struct Desc<T>(x: T)
fn desc<T>(x: T) -> Desc<T>
trait Reversible
fn fold<T, M>(m: M, xs: List<T>) -> T
struct MaxSub(has: Bool, best: Int, total: Int, pre: Int, suf: Int)  // [new] O(1), Reversible
    const zero: MaxSub
    fn of(v: Int) -> MaxSub
    fn join(a: MaxSub, b: MaxSub) -> MaxSub  // `+`
    fn scale(self, k: Int) -> MaxSub         // `*`, k >= 0
    @get fn try_best(self) -> ?Int
    fn monoid() -> Monoid<MaxSub>
struct Fenwick<D, T>  // O(n), O(log n)
    fn new(a: ?[D]T = none) -> Fenwick<D, T>
    fn add(mut self, i: D, x: T)
    fn prefix(self, b: Cut<D>) -> T
    fn fold(self, r: Range<D>) -> T
    fn first_ge(self, target: Int) -> ?Cut<D>  // O(log n)
    fn kth(self, k: Int) -> ?D                 // O(log n)
struct DualFenwick<D, T>                       // [new] range add, point get; O(n), O(log n)
    fn new(a: ?[D]T = none) -> DualFenwick<D, T>
    fn add(mut self, r: Range<D>, x: T)
    fn get(self, i: D) -> T
struct RangeFenwick<D>  // range add, range sum (Int); O(n), O(log n)
    fn new(a: ?[D]Int = none) -> RangeFenwick<D>
    fn add(mut self, r: Range<D>, x: Int)
    fn prefix(self, b: Cut<D>) -> Int
    fn fold(self, r: Range<D>) -> Int
    fn clear(mut self)   // O(n)
struct XorFenwick<D, T>  // O(n), O(log n)
    fn new(a: ?[D]T = none) -> XorFenwick<D, T>
    fn flip(mut self, i: D, x: T)
    fn prefix(self, b: Cut<D>) -> T
    fn xor(self, r: Range<D>) -> T
struct Fenwick2D<R, C, T>  // O(log R * log C)
    fn new() -> Fenwick2D<R, C, T>
    fn add(mut self, r: R, c: C, x: T)
    fn fold(self, rows: Range<R>, cols: Range<C>) -> T
enum FillOrder = Forward | Reverse
struct Prefix<D, T>                                                       // also Prefix<R, C, T> for 2D grids and Prefix<T> for List<T>
    fn new(a: [D]T) -> Prefix<D, T>                                       // O(|D|); also [R, C]T in O(|R|*|C|) and List<T> in O(n)
    fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<D, T>  // O(|D|) / O(|R|*|C|) / O(n) (takes n: Int for List<T>); append-only prefix/suffix table (`FillPrefix<D, T>`, `FillPrefix<R, C, T>`, `FillPrefix<T>`) with O(1) `.set`, `.prefix`, `.fold`
    fn prefix(self, b: Cut<D>) -> T                                       // O(1); (r: Cut<R>, c: Cut<C>) for 2D, (k: Int) for List<T>
    fn fold(self, r: Range<D>) -> T                                       // O(1); (rows: Range<R>, cols: Range<C>) for 2D, (r: Range<Int>) for List<T>
fn prefix(a)                                                              // prelude shape dispatch over [D]T, [R, C]T, and List<T>; L0513 + checker error migrate removed prefix1/prefix2/prefix_n/Prefix1/Prefix2/PrefixN/Diff1/Diff2
fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T                      // [new] O(n)
fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T                      // [new] O(n)
struct Diff<D, T>                                                         // offline 1D/2D difference array (`Diff<D, T>()` or `Diff<R, C, T>()`); O(1) add, O(|D|) / O(|R|*|C|) finish
    fn new() -> Diff<D, T>
    fn add(mut self, r: Range<D>, x: T)  // (rows: Range<R>, cols: Range<C>, x: T) for 2D
    fn finish(self) -> [D]T              // [R, C]T for 2D
// Matched C++17 benchmarks (`bench/prefix/`): p50 = 1.10x, max = 1.35x (prefix_1d 1.25x / 8.2 KB, prefix_2d 1.04x / 10.5 KB, diff_1d 1.15x / 8.4 KB, diff_2d 1.05x / 11.1 KB, fill_1d_fwd 0.88x / 11.4 KB, fill_1d_rev 0.91x / 11.6 KB, fill_2d_fwd 1.33x / 14.3 KB, fill_2d_rev 1.35x / 14.6 KB; codegen-only causes >1.1x in `research/frictions/prefix_bench.md`). `L0521` (`hand_rolled_prefix_sum`) flags manual 1D/2D prefix loops with `cpl fix` (9/9 = 100% precision on `proto/ports/`: 4 compiling live ports migrated, 5 pre-existing non-compiling draft/broken ports).
struct SegTree<D, T>(m: Monoid<T>)                                       // O(log n)
    fn new<M>(op: M, a: ?[D]T = none, fill: ?T = none) -> SegTree<D, T>  // O(n)
    fn of<M>(op: M, f: D -> T) -> SegTree<D, T>                          // O(n)
    fn refresh(mut self, i: D)                                           // O(log n)
    fn get(self, i: D) -> T
    fn set(mut self, i: D, x: T)
    fn at(mut self, i: D) -> lends mut T  // O(log n) per write: `st.at(i) += x`
    fn update(mut self, i: D, f: T -> T)  // O(log n)
    fn fold(self, r: Range<D>) -> T
    @get fn all(self) -> T  // O(1)
    @get fn monoid(self) -> Monoid<T>
    fn max_right(self, from: Cut<D>, ok: T -> Bool) -> Cut<D>
    fn min_left(self, to: Cut<D>, ok: T -> Bool) -> Cut<D>
struct LazySegTree<D, T, F>(act: Action<T, F>)                                           // O(log n), O(1)
    fn new(act: Action<T, F>, a: ?[D]T = none, fill: ?T = none) -> LazySegTree<D, T, F>  // O(n)
    fn set(mut self, i: D, x: T)
    fn update(mut self, i: D, f: T -> T)  // O(log n)
    fn get(mut self, i: D) -> T
    fn at(mut self, i: D) -> lends mut T  // O(log n) per write
    @get fn all(self) -> T                // O(1)
    @get fn monoid(self) -> Monoid<T>
    fn fold(mut self, r: Range<D>) -> T
    fn apply(mut self, r: Range<D>, f: F)
    fn max_right(mut self, from: Cut<D>, ok: T -> Bool) -> Cut<D>
    fn min_left(mut self, to: Cut<D>, ok: T -> Bool) -> Cut<D>
struct DualSegTree<D, T, F>  // [new] range apply, point get; no monoid or pulls; O(log n)
    fn new(
        id: F,
        compose: (F, F) -> F,
        apply: (F, T) -> T,
        a: ?[D]T = none,
        fill: ?T = none,
    ) -> DualSegTree<D, T, F>  // O(n)
    fn apply(mut self, r: Range<D>, f: F)
    fn get(mut self, i: D) -> T
    fn set(mut self, i: D, x: T)
    fn at(mut self, i: D) -> lends mut T
    fn update(mut self, i: D, f: T -> T)
struct LazyTreap<T, F>  // [new] randomized treap with subtree-size-aware lazy tags; O(log n)
    fn new(
        apply: (T, Int, F) -> T,
        compose: (F, F) -> F,
        split: (F, Int) -> (F, F) = (f, _) => (f, f),
        before: (T, T) -> Bool = (_, _) => false,
    ) -> LazyTreap<T, F>                              // O(1)
    @get fn len(self) -> Int                          // O(1)
    @get fn is_empty(self) -> Bool                    // O(1)
    fn insert(mut self, x: T)                         // O(log n) expected, by `before`
    fn push(mut self, x: T)                           // O(log n) expected
    fn insert_at(mut self, pos: Int, x: T)            // O(log n) expected
    fn apply_all(mut self, f: F)                      // O(1)
    fn apply_range(mut self, lo: Int, hi: Int, f: F)  // O(log n) expected
    fn get(mut self, pos: Int) -> T                   // O(log n) expected
    fn reserve(mut self, n: Int)                      // O(n); room for n elements, no regrowth
    fn take(self, k: Int) -> List<T>                  // O(k + log n); pending tags carried, not pushed
    fn to_list(self) -> List<T>                       // O(n)
const AddSum = Action<Int, Int>(0, +, (f, x, len) => x + f * len, over: +)
const AddMin = Action<Int, Int>(0, +, (f, x, len) => x + f, over: min)
const AddMax = Action<Int, Int>(0, +, (f, x, len) => x + f, over: max)
const AssignSum: Action<Int, ?Int>                   // assign over sum
const AssignMin: Action<Int, ?Int>                   // assign over min
const AssignMax: Action<Int, ?Int>                   // assign over max
const AffineSum: Action<Int, (Int, Int)>             // [new] (mul, add) over sum
fn affine_sum<T>() -> Action<T, (T, T)>              // [new] generic (Int, Mod<P>)
struct Flip<T>(now: T, flipped: T)                   // [new] paired aggregate for range toggle
    fn of(on: Bool, x: T, id: T) -> Flip<T>          // O(1)
fn flip_action<T, M>(m: M) -> Action<Flip<T>, Bool>  // [new] range-toggle action over `m`
struct SparseTable<D, T>(op: (T, T) -> T)            // O(n log n), O(1) idempotent
    fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>
    fn fold(self, r: Range<D>) -> T                // O(1)
struct DisjointSparseTable<D, T>(op: (T, T) -> T)  // [new] O(n log n), O(1) any semigroup
    fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T>
    fn fold(self, r: Range<D>) -> T  // O(1)
fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool
struct MonoDeque<T>  // [new] O(1) amortized sliding-window min/max
    fn new(pick: (T, T) -> T) -> MonoDeque<T>
    fn by(better: (T, T) -> Bool) -> MonoDeque<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn best(self) -> ?T
    fn clear(mut self)
    fn push(mut self, at: Int, val: T)
    fn drop_before(mut self, min_at: Int)
struct WindowFold<T>(m: Monoid<T>)  // [new] O(1) amortized inverse-free queue fold
    fn new<M>(op: M) -> WindowFold<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn fold(self) -> T
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
fn mo<D, R>(queries: List<Range<D>>, add: D -> (), remove: D -> (), answer: Int -> R) -> List<R>  // O((n + q) sqrt n)
fn mo_order<D>(queries: List<Range<D>>) -> List<Int>                                              // [new] O(q log q)
fn mo_steps<D>(from: Range<D>, to: Range<D>) -> yields (pos: D, entering: Bool)                   // [new]
```

Phase 2 additions (see `spec/lib/rationale/ranges.md` §3.8):
`MergeSortTree<D, T>` and `SparseSegTree<T>` (`std.segtree`),
`PersistentSegTree<D, T>` + `Version` + `RangeKth<D, T>` (`std.persistent`),
`BeatsSegTree<D>` (`std.beats`: `chmin`/`chmax`/`add`/`sum`/`min_val`/`max_val`),
`TimeTree<E>` + `TimeTree.run` + `offline_over` + `TimeStep` (`std.timetree`: offline time-interval divide-and-conquer),
and `LiChao` + `LiChaoTag<V>` (coordinated with the DP lane).

### 4.11 `std.graph`, `std.tree`: graphs and trees

`Graph<V>` is a compressed adjacency list over a vertex domain, built by
`undirected`/`directed`; `Rooted<V>` a rooted tree with parents, depths,
heavy-first preorder positions `t.Pre` in which every subtree and every heavy
chain is a range, so one range structure serves subtree and path queries;
`Centroids<V>` a centroid decomposition. The design, laws and worked problems
are in `spec/lib/rationale/trees.md`.

```cp
type Tree<V> = [0..V.len - 1](u: V, v: V)
type SimpleGraph<V, E> = [E](u: V, v: V)
type ConnectedGraph<V, E> = [E](u: V, v: V)
type Dag<V, E> = [E](u: V, v: V)
type Digraph<V, E> = [E](u: V, v: V)
trait Neighbours
trait Target
struct Graph<V, E>  // O(n + m)
    fn directed(edges: [E](u: V, v: V)) -> Graph<V, E>
    fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E>
    fn out(self, u: V) -> yields (to: V, e: E)
    fn get(self, u: V) -> List<(to: V, e: E)>
    fn deg(self, u: V) -> Int          // O(1)
    fn transpose(self) -> Graph<V, E>  // O(n + m)
struct Adj<V, A>                       // CSR adjacency built from pairs/triples, O(n + m)
    fn out(self, u: V) -> yields A
    fn get(self, u: V) -> List<A>
    fn deg(self, u: V) -> Int  // O(1)
// list-form hooks behind `Graph.directed(pairs)` etc., never called by name
fn directed_of_pairs<V>(edges: List<(V, V)>) -> Adj<V, V>                 // O(n + m)
fn undirected_of_pairs<V>(edges: List<(V, V)>) -> Adj<V, V>               // O(n + m)
fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> Adj<V, (V, W)>    // O(n + m)
fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> Adj<V, (V, W)>  // O(n + m)
struct Paths<V, W, A>                                                     // dist: [V]?W, back: [V]?(from: V, via: A), order: List<V>
    fn get(self, v: V) -> ?W                                              // O(1)
    fn reached(self, v: V) -> Bool                                        // O(1)
    fn path(self, t: V) -> ?List<V>                                       // O(len)
    fn arcs(self, t: V) -> ?List<A>                                       // O(len)
// every search takes `until: t`: stop once t is settled
fn bfs<V, N>(next: N, from: List<V>, until: ?V = none) -> Paths<V, Int, A>  // O(n + m)
// O(n + m) deque; traps unless every weight is 0 or 1
fn bfs01<V, N, A>(next: N, from: List<V>, w: A -> Int, until: ?V = none)
    -> Paths<V, Int, A>
// O((n + m) log m)
fn dijkstra<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W, until: ?V = none)
    -> Paths<V, W, A>
// O(|S| |moves|) time, O(|S|) memory: one cursor per move, no heap
fn fixed_cost_paths<S, M, W>(
    from: List<S>,
    moves: List<M>,
    cost: M -> W,
    step: (M, S) -> ?S,
    until: ?S = none,
) -> Paths<S, W, M>
// O(n m); none on a reachable negative cycle
fn bellman_ford<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W)
    -> ?Paths<V, W, A>
fn floyd_warshall<V, W, R>(mut d: [V, V]?W, over: R = Semiring(min, +)) -> Bool  // O(n^3); false on a negative cycle
fn topo<V, N>(next: N) -> ?List<V>                                               // O(n + m)
fn peel<V, N>(next: N) -> List<V>                                                // O(n + m)
fn find_cycle<V, N>(next: N) -> ?List<V>                                         // O(n + m)
fn components<V, N>(next: N) -> (domain C, comp: [V]C)                           // O(n + m)
fn scc<V, N>(next: N) -> (domain C, comp: [V]C)                                  // O(n + m), C topological
fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)>
fn two_edge_components<V, N>(next: N) -> (domain C, comp: [V]C)  // O(n + m)
fn bridges<V, E, N>(next: N) -> [E]Bool                          // O(n + m)
fn articulation_points<V, N>(next: N) -> [V]Bool                 // O(n + m)
fn biconnected_components<V, N>(next: N) -> List<List<V>>        // O(n + m)
fn two_color<V, N>(next: N) -> ?[V]Bool                          // O(n + m)
struct Walk<V, E>                                                // start: V, arcs: List<(to: V, e: E)>
    @get fn vertices(self) -> List<V>                            // O(len)
// O(n + m) Hierholzer
fn euler_walk<V, E>(edges: [E](u: V, v: V), directed: Bool) -> ?Walk<V, E>
struct Lit<X>(x: X, value: Bool)
    @get fn flip(self) -> Lit<X>
struct TwoSat<X>
    fn new() -> TwoSat<X>
    fn add_or(mut self, a: Lit<X>, b: Lit<X>)     // O(1)
    fn implies(mut self, a: Lit<X>, b: Lit<X>)    // O(1)
    fn must(mut self, a: Lit<X>)                  // O(1)
    fn at_most_one(mut self, lits: List<Lit<X>>)  // O(k)
    fn solve(self) -> ?[X]Bool                    // O(|X| + clauses)
struct Functional<V>                              // O(n log n)
    fn new(next: [V]V) -> Functional<V>
    fn on_ring(self, v: V) -> Bool       // O(1)
    fn ring_of(self, v: V) -> Int        // O(1)
    fn steps_to_ring(self, v: V) -> Int  // O(1)
    fn root(self, v: V) -> V             // O(1)
    fn jump(self, v: V, k: Int) -> V     // O(log n)
    fn dist(self, a: V, b: V) -> ?Int    // O(log n)
    fn orbit(self, v: V) -> Int          // O(1)
    fn jump_while(self, v: V, pred: V -> Bool, max_steps: Int = V.K)
        -> ?(V, Int)   // O(log n)
struct Rooted<V, Pre>  // O(n) time and memory; spec/lib/rationale/trees.md
    fn new<N>(adj: N, root: V)
    fn from_parents(par: [V]?V)  // O(n)
    fn weighted<N, A>(adj: N, root: V, len: A -> Int)
    fn forest<N>(adj: N, roots: List<V>)  // O(n)
    fn parent(self, v: V) -> ?V           // O(1), like every shape query
    fn children(self, v: V) -> List<V>    // in the order adj listed them
    heavy: [V]?V                          // O(1) field, like at and far
    fn light(self, v: V) -> yields V      // O(deg v)
    fn depth(self, v: V) -> Int
    far: [V]Int  // weighted depth
    fn size(self, v: V) -> Int
    fn root_of(self, v: V) -> V                          // O(depth)
    @get fn subtree_sizes(self) -> [V]Int                // O(n)
    fn pos(self, v: V) -> Pre                            // O(1)
    at: [Pre]V                                           // inverse of pos
    fn sub(self, v: V) -> Range<Pre>                     // O(1)
    fn below(self, v: V) -> Range<Pre>                   // O(1), sub(v) without v
    fn subtree(self, v: V) -> yields V                   // O(size v)
    fn reorder<T>(self, a: [V]T) -> [Pre]T               // O(n)
    @get fn tour(self) -> yields (v: V, entering: Bool)  // O(n)
    fn is_anc(self, a: V, b: V) -> Bool                  // O(1)
    fn lower(self, u: V, v: V) -> V                      // O(1)
    fn lca(self, a: V, b: V) -> V                        // O(log n)
    fn dist(self, a: V, b: V) -> Int                     // O(log n)
    fn ancestor(self, v: V, k: Int) -> ?V                // O(log n)
    fn step(self, a: V, b: V, k: Int) -> ?V              // O(log n)
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V     // O(log^2 n)
    fn path(self, a: V, b: V) -> List<V>                 // O(|path|)
    fn chains(                                           // O(log n)
        self,
        a: V,
        b: V,
        edges: Bool = false,
    ) -> yields (range: Range<Pre>, rising: Bool)
    // O(log n) range folds; path order when the monoid is Reversible
    fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T
    // O(log n) range applies
    fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false)
    fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T  // O(n)
    // O(n): rooted DP, and the same DP for every root; lift defaults to (x, c, p) => x
    fn up<T, E, M>(self, m: M, lift: (T, V, V) -> E, finish: (E, V) -> T) -> [V]T
    fn reroot<T, E, M>(self, m: M, lift: (T, V, V) -> E, finish: (E, V) -> T) -> [V]T
    // O(k)
    fn excl_fold<T, M>(self, v: V, m: M, f: V -> T) -> yields (c: V, rest: T)
    fn diameter(self) -> (len: Int, u: V, v: V)      // O(n)
    @get fn centroid(self) -> V                      // O(depth)
    @get fn centroids(self) -> List<V>               // O(depth)
    fn shapes(self) -> (domain Shape, of: [V]Shape)  // O(n log n)
    // O(k log k + k log n)
    fn virtual(self, keys: List<V>) -> VirtualTree<V, K>
    // fresh domain vt.K in preorder; at: [K]V, up: [K]?K, len: [K]Int, key: [K]Bool
struct Centroids<V>  // O(n log n) time and memory
    fn new<N>(adj: N) -> Centroids<V>
    fn parent(self, c: V) -> ?V                                  // O(1)
    fn depth(self, c: V) -> Int                                  // O(1), at most log2 n
    fn ancestors(self, v: V) -> yields (c: V, dist: Int)         // O(log n), v's own first
    fn piece(self, c: V) -> yields (v: V, dist: Int, branch: V)  // O(|piece|)
struct LinkCut<V, T>                                             // dynamic forest; amortized O(log n) per call
    fn new<M>(op: M, a: ?[V]T = none) -> LinkCut<V, T>           // O(n)
    fn link(mut self, u: V, v: V)
    fn cut(mut self, u: V, v: V)
    fn connected(mut self, u: V, v: V) -> Bool
    fn root_of(mut self, v: V) -> V
    fn make_root(mut self, v: V)
    fn fold_path(mut self, u: V, v: V) -> T  // path order; Reversible
    fn get(self, v: V) -> T                  // O(1)
    fn set(mut self, v: V, x: T)
    fn lca(mut self, u: V, v: V, root: V) -> ?V
struct LazyLinkCut<V, T, F>                                             // LinkCut's verbs plus path updates under an Action
    fn new(act: Action<T, F>, a: ?[V]T = none) -> LazyLinkCut<V, T, F>  // O(n)
    fn apply_path(mut self, u: V, v: V, f: F)                           // amortized O(log n)
    fn get(mut self, v: V) -> T                                         // amortized O(log n): pending tags
enum PermKind: Leaf, Incr, Decr, Partial, Full
struct PermTree<I, V, Node>              // binary permutation tree of a bijection [I]V
    fn of(p: [I]V) -> PermTree<I, V, _>  // O(n), fresh node domain of 2n - 1
    kind: [Node]PermKind; first, last: [Node]I; lo, hi: [Node]V; root: Node
    fn leaf(self, i: I) -> Node                       // O(1)
    fn left(self, v: Node) -> Node                    // O(1), traps on a leaf
    fn right(self, v: Node) -> Node                   // O(1), traps on a leaf
    fn span(self, v: Node) -> Range<I>                // O(1), positions
    fn vals(self, v: Node) -> Range<V>                // O(1), exact unless Partial
struct JumpTree<V>                                    // forest grown leaf by leaf; skew-binary jumps
    fn new() -> JumpTree<V>                           // O(n), every vertex a root
    fn add(mut self, v: V, p: V)                      // O(1), hang the lone vertex v below p
    fn parent(self, v: V) -> ?V                       // O(1)
    fn depth(self, v: V) -> Int                       // O(1)
    fn root_of(self, v: V) -> V                       // O(log n)
    fn ancestor(self, v: V, k: Int) -> ?V             // O(log n)
    fn lca(self, a: V, b: V) -> ?V                    // O(log n), none across trees
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V  // O(log n) calls
```

[new], promoted from `examples/local-lib/graph/`:

```cp
fn mst<V>(edges: List<(u: V, v: V, w: Int)>) -> (total: Int, used: List<Int>)
// O(m log m) Kruskal; indices of the chosen edges
```

Trees, [rename] (coherence pass, `design/stdlib/trees.md` "Coherence"; the
old spelling stays a deprecated alias until a codemod moves the ports):

```cp
// one constructor convention: T(args) is T.new(args), options are named
fn Rooted.new<N, A>(adj: N, root: V, w: A -> Int = a => 1)  // O(n); was Rooted.weighted(adj, root, len)
fn PermTree.new(p: [I]V) -> PermTree<I, V, _>               // O(n); was PermTree.of(p)
fn Rooted.jump(self, a: V, b: V, k: Int) -> ?V              // O(log n); was step, the word Functional.jump uses
fn Rooted.jump_while(self, v: V, ok: V -> Bool) -> ?V       // O(log^2 n); was lift_while (also JumpTree)
VirtualTree.parent: [K]?K                                   // was up, which is Rooted's bottom-up fold
VirtualTree.w: [K]Int                                       // was len, which is every collection's size; w as in graph edges
// the monoid argument is `op` everywhere in the area, as in SegTree and LinkCut (was m)
```

Trees, [new]: the rest of the surface. Every `Range<Pre>` argument also
takes a range of cuts `j..i` over `t.Pre.bounds` (decision 124);
`t.sub(v).l` and `.r` are cuts (decision 2.2).

```cp
struct Rooted<V, Pre>
    fn top(self, v: V) -> V                              // O(1), head of v's heavy chain
    fn chain(self, v: V) -> Range<Pre>                   // O(1), v's whole heavy chain, top first
    fn sub_at(self, v: V, root: V) -> yields Range<Pre>  // O(log n), at most 2 ranges: sub(v) were the tree rooted at root
    fn on_path(self, c: V, a: V, b: V) -> Bool           // O(log n)
    fn meet(self, a: V, b: V, c: V) -> V                 // O(log n), the vertex on all three pairwise paths
    // O(n): for v != root, the DP of the tree minus sub(v), rooted at parent(v); reroot's other half
    fn above<T, E, M>(
        self,
        op: M,
        lift: (T, V, V) -> E = (x, c, p) => x,
        finish: (E, V) -> T,
    ) -> [V]?T
    domain Bfs                                  // breadth-first positions, each depth layer in preorder
    by_depth: [Bfs]V                            // O(1) field
    fn layer(self, v: V, d: Int) -> Range<Bfs>  // O(log n), v's descendants at depth d
    domain Ev                                   // the 2n enter and exit events of tour, in order
    fn enter(self, v: V) -> Ev                  // O(1)
    fn exit(self, v: V) -> Ev                   // O(1)
    // O(log n), Mo on trees: vertices met an odd number of times in range, plus extra, are path(a, b)
    fn path_events(self, a: V, b: V) -> (range: Range<Ev>, extra: ?V)
    fn long_paths(self) -> LongPaths<V, L>  // O(n), fresh domain L of n slots
    // power 2: for (v, step) in t.subtree(r), step.prune() skips v's subtree; O(visited)
struct LongPaths<V, L>                // depth-indexed DP merged in O(n) total
    deep: [V]?V                       // O(1) field: the child with the tallest subtree
    height: [V]Int                    // O(1) field: edges on the longest downward path
    fn slots(self, v: V) -> Range<L>  // O(1), height(v) + 1 slots, depth offset 0 first; deep[v]'s start one later
    fn slot(self, v: V, d: Int) -> L  // O(1), traps unless 0 <= d <= height(v)
/// The farthest pair of a vertex set as a monoid, for any range structure or fold.
fn far_pairs<V>(dist: (V, V) -> Int) -> Monoid<?(u: V, v: V, len: Int)>  // O(1) dist calls per op
struct LcaTable<V>                                                       // Euler tour plus sparse table
    fn new<Pre>(t: Rooted<V, Pre>) -> LcaTable<V>                        // O(n log n) time and memory
    fn lca(self, a: V, b: V) -> V                                        // O(1)
    fn dist(self, a: V, b: V) -> Int                                     // O(1)
struct LinkCut<V, T>                                                     // also LazyLinkCut
    fn dist(mut self, a: V, b: V) -> ?Int                                // amortized O(log n), none across trees
    fn jump(mut self, a: V, b: V, k: Int) -> ?V                          // amortized O(log n), k-th vertex a -> b
struct JumpTree<V>
    fn dist(self, a: V, b: V) -> ?Int                         // O(log n), none across trees
struct EulerTourTree<V, T>                                    // dynamic forest with whole-tree and subtree folds; expected O(log n) per call
    fn new<M>(op: M, a: ?[V]T = none) -> EulerTourTree<V, T>  // O(n)
    fn link(mut self, u: V, v: V)
    fn cut(mut self, u: V, v: V)
    fn connected(mut self, a: V, b: V) -> Bool
    fn len(mut self, v: V) -> Int  // vertices in v's tree
    fn get(self, v: V) -> T        // O(1)
    fn set(mut self, v: V, x: T)
    fn fold_tree(mut self, v: V) -> T                                         // the whole tree holding v
    fn fold_sub(mut self, v: V, away: V) -> T                                 // v's side of the edge (v, away)
struct LazyEulerTourTree<V, T, F>                                             // EulerTourTree's verbs plus updates under an Action
    fn new(act: Action<T, F>, a: ?[V]T = none) -> LazyEulerTourTree<V, T, F>  // O(n)
    fn apply_tree(mut self, v: V, f: F)                                       // expected O(log n)
    fn apply_sub(mut self, v: V, away: V, f: F)                               // expected O(log n)
// derived trees
fn cartesian_tree<D, T>(a: [D]T, by: (T, T) -> Bool = (x, y) => x < y) -> [D]?D  // O(n), parent array; root the by-least
fn kruskal_tree<V, E, W>(edges: [E](u: V, v: V, w: W)) -> KruskalTree<V, N, W>   // O(m log m), fresh domain N
struct KruskalTree<V, N, W>                                                      // leaves are V, an internal node per merging edge
    leaf: [V]N                                                                   // O(1) field
    w: [N]?W                                                                     // O(1) field: the merging weight, none at a leaf
    tree: Rooted<N, Pre>                                                         // a forest when the graph is not connected
    fn reach(self, v: V, limit: W) -> N                                          // O(log^2 n), highest node over v with w <= limit
fn prufer<V>(edges: Tree<V>) -> List<V>                                          // O(n)
fn from_prufer<V>(code: List<V>) -> Tree<V>                                      // O(n), traps unless code.len == V.count - 2
```

### 4.12 `std.flow`: flows, cuts and matchings

The design and its reasons are in `spec/lib/flows.md`. Two networks
share `Arc` and one accessor vocabulary (`flow`, `cap`, `source`,
`target`, `rev`); everything whose input is a table or an edge list is a
function returning a value. Objectives are named (`max_`/`min_`), never
encoded by negating weights; options are named arguments (`limit:`,
`cap: none`). Sides, vertices and input edges are domains: a matching is
`[L]?R`, a bounded flow is `[E]Int`. All of it ships in `stdlib/flow.cp`, with
matched-C++ timings in `bench/flow/`.

```cp
struct Arc(id: 0..2_147_483_648)
    @get fn rev(self) -> Arc                            // O(1)
struct FlowGraph<V>                                     // Dinic
    fn new() -> FlowGraph<V>                            // O(V)
    fn add(mut self, u: V, v: V, cap: ?Int) -> Arc      // O(1)
    fn add_both(mut self, u: V, v: V, cap: Int) -> Arc  // O(1)
    fn flow(self, a: Arc) -> Int                        // O(1)
    fn cap(self, a: Arc) -> Int                         // O(1)
    fn source(self, a: Arc) -> V                        // O(1)
    fn target(self, a: Arc) -> V                        // O(1)
    fn max_flow(mut self, s: V, t: V, limit: ?Int = none) -> Int
    // O(V^2 E); O(E sqrt V) on unit networks
    fn push_relabel(mut self, s: V, t: V) -> Int
    // the same flow, O(V^2 sqrt E); highest label, gap
    fn min_cut(self, s: V) -> [V]Bool                          // O(V + E)
    fn reset(mut self)                                         // O(E)
struct CostFlow<V>                                             // successive shortest paths
    fn new() -> CostFlow<V>                                    // O(V)
    fn add(mut self, u: V, v: V, cap: ?Int, cost: Int) -> Arc  // O(1)
    fn flow(self, a: Arc) -> Int                               // O(1)
    fn cap(self, a: Arc) -> Int                                // O(1)
    fn cost(self, a: Arc) -> Int                               // O(1)
    fn source(self, a: Arc) -> V                               // O(1)
    fn target(self, a: Arc) -> V                               // O(1)
    fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = none)
        -> (flow: Int, cost: Int)
        // O(F E log V); plus O(V E) once when a cost is negative
    fn slope(mut self, s: V, t: V, limit: ?Int = none)
        -> List<(flow: Int, cost: Int)>  // same
fn max_closure<V, E>(profit: [V]Int, needs: [E](u: V, v: V))
    -> (value: Int, take: [V]Bool)  // one max flow
fn max_matching<L, R>(adj: [L]List<R>)
    -> (size: Int, mate: [L]?R, back: [R]?L)  // O(E sqrt V)
fn min_vertex_cover<L, R>(adj: [L]List<R>)
    -> (left: [L]Bool, right: [R]Bool)  // O(E sqrt V)
fn max_weight_matching<L, R>(weight: (L, R) -> Int)
    -> (total: Int, mate: [L]R)  // O(L^2 R), L <= R
fn min_weight_matching<L, R>(weight: (L, R) -> Int)
    -> (total: Int, mate: [L]R)  // O(L^2 R), L <= R
fn general_matching<V>(adj: [V]List<V>)
    -> (size: Int, mate: [V]?V)  // O(V^3) Edmonds
fn max_weight_general_matching<V, E>(edges: [E](u: V, v: V, w: Int))
    -> (total: Int, mate: [V]?V)  // O(V^3), O(V^2) memory
fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int
fn max_bounded_flow<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int), s: V, t: V)
    -> ?(value: Int, flow: [E]Int)
fn min_bounded_flow<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int), s: V, t: V)
    -> ?(value: Int, flow: [E]Int)
    // each two max flows on V + 2 vertices
fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int))
    -> List<(u: V, v: V, w: Int)>  // V - 1 max flows
fn global_min_cut<V, E>(edges: [E](u: V, v: V, w: Int))
    -> (w: Int, side: [V]Bool)  // Stoer-Wagner, O(V^3)
```

Staged (design section 7):
`min_cost_circulation`, `set_cap`, matroid intersection. Network nodes
built from several domains wait on sum domains (spec §21.11, core lane);
until then `max_closure`, the bounded flows and the matchings keep their
source and sink inside the library.

### 4.13 `std.strings`: strings

ASCII byte strings, KMP/prefix and Z functions, borders and Lyndon words,
palindromes (Manacher and palindromic tree), suffix array/LCP and suffix
automaton, tries and Aho-Corasick, and double-mod rolling hashing (1D and 2D).
Full design and exemplars in `spec/lib/strings.md`.

```cp
fn repeat(self: Str, n: Int) -> Str
fn starts_with(self: Str, p: Str) -> Bool // O(p.len)
fn ends_with(self: Str, p: Str) -> Bool // O(p.len)
fn find<P>(self: Str, p: P) -> ?Int // O(s.len + p.len)
fn count(self: Str, c: Char) -> Int // O(s.len)
fn rfind<P>(self: Str, p: P) -> ?Int // O(s.len + p.len)
@get fn lower(self: Char) -> Char
@get fn upper(self: Char) -> Char
@get fn digit(self: Char) -> Int
fn z_function<S: Indexed>(s: S) -> List<Int>
fn prefix_function<S: Indexed>(s: S) -> List<Int>
fn find_all<S: Indexed, P: Indexed>(self: S, pat: P) -> yields Int
fn common_prefix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int // O(min(|self|, |other|))
fn common_suffix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int // O(min(|self|, |other|))
fn period<S: Indexed>(s: S) -> Int // O(n)
fn suffix_array<S: Indexed>(s: S) -> List<Int>
fn lcp_array<S: Indexed>(s: S, sa: List<Int>) -> List<Int>
struct HashVal(h1: Int, h2: Int)
    @get fn hash_token(self) -> Int
trait Hashable
struct RollingHash // O(n), O(1)
    fn new<S>(seq: S, bases: ?(Int, Int)
    fn with_bases<S>(
    @get fn len(self) -> Int
    @get fn all(self) -> HashVal // O(1)
    fn slice(self, l: Int, r: Int) -> HashVal // O(1)
    fn get<R>(self, r: R) -> HashVal // O(1)
    fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal // O(1)
```

[new], designed in `spec/lib/strings.md` and promoted from
`examples/local-lib/str/`:

```cp
fn is_palindrome<S: Indexed>(s: S) -> Bool                               // O(n)
fn borders<S: Indexed>(s: S) -> yields Int                               // O(n)
fn prefix_matches<S: Indexed, P: Indexed>(self: S, pat: P) -> List<Int>  // O(|self| + |pat|)
fn overlap<A: Indexed, B: Indexed>(a: A, b: B) -> Int                    // O(|a| + |b|)
fn lyndon_factorization<S: Indexed>(s: S) -> List<Int>                   // O(n)
fn min_rotation<S: Indexed>(s: S) -> Int                                 // O(n)
fn lyndon_runs<S: Indexed>(s: S) -> List<(l: Int, r: Int, period: Int)>  // O(n log n)
struct Palindromes(odd: List<Int>, even: List<Int>)
    fn is_palindrome<R>(self, r: R) -> Bool    // O(1), Range<Int> or Range<D>
    @get fn count(self) -> Int                 // O(n)
    @get fn longest(self) -> (l: Int, r: Int)  // O(n)
fn manacher<S: Indexed>(s: S) -> Palindromes   // O(n)
struct PalindromicTree<A>
    fn new() -> PalindromicTree<A>                                        // O(|A|)
    fn of<S: Seq>(s: S) -> PalindromicTree<A>                             // O(n * |A|) mem, O(n)
    fn push(mut self, a: A) -> Int                                        // O(1) amortized
    @get fn len(self) -> Int                                              // O(1)
    @get fn distinct(self) -> Int                                         // O(1)
    fn pal_len(self, state: Int) -> Int                                   // O(1)
    fn link(self, state: Int) -> Int                                      // O(1)
    fn step(self, state: Int, a: A) -> ?Int                               // O(1)
    fn diff(self, state: Int) -> Int                                      // O(1)
    fn series_link(self, state: Int) -> Int                               // O(1)
    fn occurrences(self) -> List<Int>                                     // O(n)
fn RollingHash.lcs(self, a: Int, b: Int) -> Int                           // O(log n)
fn RollingHash.lcp(self, a: Int, b: Int, of: ?RollingHash = None) -> Int  // O(log n)
struct Hash2D<R, C>
    fn new(code: (R, C) -> Int, bases: ?(Int, Int) = none) -> Hash2D<R, C>  // O(R * C)
    fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal                // O(1)
struct SuffixArray<T>
    fn new<S: Indexed>(s: S) -> SuffixArray<T>                   // O(n log n)
    @get fn len(self) -> Int                                     // O(1)
    fn lcp_between(self, i: Int, j: Int) -> Int                  // O(1)
    fn matches<P: Indexed>(self, pat: P) -> Range<Int>           // O(|pat| log n)
    fn count<P: Indexed>(self, pat: P) -> Int                    // O(|pat| log n)
    fn lcp_range(self, i: Int, len: Int) -> Range<Int>           // O(log n)
    fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int  // O(1)
    @get fn distinct(self) -> Int                                // O(n)
struct SuffixAutomaton<A>
    fn new() -> SuffixAutomaton<A>              // O(|A|)
    fn of<S: Seq>(s: S) -> SuffixAutomaton<A>   // O(n * |A|) mem, O(n)
    fn push(mut self, a: A)                     // O(1) amortized
    @get fn root(self) -> Int                   // O(1)
    @get fn len(self) -> Int                    // O(1)
    fn step(self, state: Int, a: A) -> ?Int     // O(1)
    fn link(self, state: Int) -> ?Int           // O(1)
    fn longest(self, state: Int) -> Int         // O(1)
    fn shortest(self, state: Int) -> Int        // O(1)
    fn first_pos(self, state: Int) -> ?Int      // O(1)
    fn find<S: Seq>(self, word: S) -> ?Int      // O(|word|)
    fn contains<S: Seq>(self, word: S) -> Bool  // O(|word|)
    @get fn distinct(self) -> Int               // O(states)
    fn topo_order(self) -> List<Int>            // O(states)
    fn occurrences(self) -> List<Int>           // O(states)
struct Trie<A>
    fn new() -> Trie<A>                              // O(|A|)
    @get fn root(self) -> Int                        // O(1)
    @get fn len(self) -> Int                         // O(1)
    fn step(self, node: Int, a: A) -> ?Int           // O(1)
    fn find<S: Seq>(self, word: S) -> ?Int           // O(|word|)
    fn add<S: Seq>(mut self, word: S) -> Int         // O(|word|)
    fn remove<S: Seq>(mut self, word: S) -> Bool     // O(|word|)
    fn count<S: Seq>(self, word: S) -> Int           // O(|word|)
    fn count_prefix<S: Seq>(self, prefix: S) -> Int  // O(|prefix|)
    fn ends_at(self, node: Int) -> Int               // O(1)
    fn pass_at(self, node: Int) -> Int               // O(1)
struct AhoCorasick<A>
    fn new<S: Seq>(patterns: List<S>) -> AhoCorasick<A>                  // O(sum |p| * |A|)
    @get fn root(self) -> Int                                            // O(1)
    @get fn len(self) -> Int                                             // O(1)
    fn step(self, state: Int, a: A) -> Int                               // O(1)
    fn link(self, state: Int) -> Int                                     // O(1)
    fn hit(self, state: Int) -> Bool                                     // O(1)
    fn words_at(self, state: Int) -> List<Int>                           // O(output)
    fn matches<S: Seq>(self, text: S) -> yields (at: Int, pattern: Int)  // O(|text| + output)
    fn counts<S: Seq>(self, text: S) -> List<Int>                        // O(|text| + states)
```

Matched C++17 benchmarks (`bash bench/strings/run.sh`, 10/10 structures covered,
**p50 = 0.87x**, **max = 1.11x**):
`z_prefix` 1.11x (26.6 vs 23.9 ms), `rolling_hash` 0.58x (25.0 vs 42.7 ms),
`hash2d` 0.88x (16.5 vs 18.7 ms), `suffix_array` 0.83x (35.7 vs 42.9 ms),
`suffix_automaton` 0.72x (28.1 vs 39.0 ms), `palindromic_tree` 0.96x (19.8 vs 20.7 ms),
`aho_corasick` 0.81x (16.9 vs 20.7 ms), `trie` 1.08x (35.1 vs 32.4 ms),
`manacher_lyndon` 1.11x (16.0 vs 14.4 ms), `lyndon_runs` 0.86x (44.0 vs 51.2 ms).

### 4.14 `std.geometry`: 2D and 3D integer and floating-point geometry

Exact integer points (`Pt`, `Pt3`) and careful floating-point vectors (`Vec`,
`Vec3`), kept as distinct types so exact predicates never silently mix with
approximate arithmetic. Full design, coverage audit, cross-area notes, powers
needed, and dreams audit live in `spec/lib/geometry.md`. Orientation
predicates compute cross and dot signs in `I128`, staying exact for coordinates
up to `1e18`. Plain value types and stateful structures use plain positional
constructors (`Pt(x, y)`, `Circle(c, r)`, `OnlineHull()`); alternate
representations use named factories (`HalfPlane.through`, `Circle.through`,
`Vec.from_angle`).

```cp
struct Pt(x: Int, y: Int)                                                              // Add, Sub, Neg, Mul<Int>, Eq, Ord (x, y), Hash, Read, Show
    @get fn norm2(self) -> Int                                                         // O(1)
    @get fn norm2_128(self) -> I128                                                    // O(1)
    @get fn len(self) -> F64                                                           // O(1)
    @get fn perp(self) -> Pt                                                           // O(1), 90 deg CCW
    @get fn perp_cw(self) -> Pt                                                        // O(1), 90 deg CW
    @get fn angle(self) -> F64                                                         // O(1), atan2(y, x)
    @get fn unit_dir(self) -> Pt                                                       // O(log max(|x|, |y|)), reduced by gcd
    @get fn to_vec(self) -> Vec                                                        // O(1)
fn dot(a: Pt, b: Pt) -> Int                                                            // O(1)
fn cross(a: Pt, b: Pt) -> Int                                                          // O(1)
fn dot128(a: Pt, b: Pt) -> I128                                                        // O(1)
fn cross128(a: Pt, b: Pt) -> I128                                                      // O(1)
fn dist2(a: Pt, b: Pt) -> Int                                                          // O(1)
fn dist2_128(a: Pt, b: Pt) -> I128                                                     // O(1)
fn dist(a: Pt, b: Pt) -> F64                                                           // O(1)
fn turn(a: Pt, b: Pt, c: Pt) -> Int                                                    // O(1), I128: 1 CCW, -1 CW, 0 collinear
fn same_dir(a: Pt, b: Pt) -> Bool                                                      // O(1), I128
fn angle_less(a: Pt, b: Pt) -> Bool                                                    // O(1), I128
struct ByAngle(p: Pt)                                                                  // Ord: CCW in [0, 2*pi) from +x, zero first, I128
fn arg(p: Pt) -> ByAngle                                                               // O(1), float-free exact angular sort key
fn on_segment(p: Pt, a: Pt, b: Pt, strict: Bool = false) -> Bool                       // O(1), I128
fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool             // O(1), I128
fn any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int)  // O(n log n), O(n) memory
fn lower_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt>               // O(n), O(n) memory
fn upper_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt>               // O(n), O(n) memory
fn convex_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt>                     // O(n log n), O(n) memory
fn convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>                  // O(n log n), O(n) memory
fn minkowski_sum(a: List<Pt>, b: List<Pt>) -> List<Pt>                                 // O(|a| + |b|), O(|a| + |b|) memory
fn minkowski_upper(a: List<Pt>, b: List<Pt>) -> List<Pt>                               // O(|a| + |b|), O(|a| + |b|) memory
fn area2(poly: List<Pt>) -> Int                                                        // O(n)
fn area2_128(poly: List<Pt>) -> I128                                                   // O(n)
enum Location = Inside | Boundary | Outside
fn locate(poly: List<Pt>, p: Pt) -> Location                        // O(n), I128
fn locate_convex(hull: List<Pt>, p: Pt) -> Location                 // O(log n), I128
fn diameter2(hull: List<Pt>) -> Int                                 // O(n), I128 rotating calipers
fn min_bounding_box(hull: List<Pt>) -> (area: F64, perimeter: F64)  // O(n), I128 calipers
fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int                     // O(log n), I128
fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int)  // O(log n), I128
fn closest_pair2(pts: List<Pt>) -> Int                              // O(n log n), O(n) memory
fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int)          // O(n log n), O(n) memory

fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool                                            // O(1)
fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int                                                  // O(1)
struct Vec(x: F64, y: F64)                                                                   // Add, Sub, Neg, Mul<F64>, Div<F64>; no Show (precision required)
    @get fn norm2(self) -> F64                                                               // O(1)
    @get fn len(self) -> F64                                                                 // O(1)
    @get fn perp(self) -> Vec                                                                // O(1)
    @get fn angle(self) -> F64                                                               // O(1)
    @get fn unit(self) -> Vec                                                                // O(1)
    fn rot(self, by: Vec) -> Vec                                                             // O(1), rotate by unit direction vector
fn vdot(a: Vec, b: Vec) -> F64                                                               // O(1)
fn vcross(a: Vec, b: Vec) -> F64                                                             // O(1)
fn vdist(a: Vec, b: Vec) -> F64                                                              // O(1)
fn vturn(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> Int                                     // O(1)
fn project(p: Vec, a: Vec, b: Vec) -> Vec                                                    // O(1)
fn reflect(p: Vec, a: Vec, b: Vec) -> Vec                                                    // O(1)
fn line_dist(p: Vec, a: Vec, b: Vec) -> F64                                                  // O(1)
fn seg_dist(p: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> F64                                  // O(1)
fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec                   // O(1)
fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec                             // O(1)
fn circle_line_intersect(center: Vec, r: F64, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>  // O(1)
fn circle_intersect(c1: Vec, r1: F64, c2: Vec, r2: F64, eps: F64 = 1e-9) -> List<Vec>        // O(1)
fn circle_tangents(center: Vec, r: F64, p: Vec, eps: F64 = 1e-9) -> List<Vec>                // O(1)
fn circle_circle_tangents(
    c1: Vec,
    r1: F64,
    c2: Vec,
    r2: F64,
    inner: Bool = false,
    eps: F64 = 1e-9,
) -> List<(Vec, Vec)>                                                           // O(1)
fn circles_union_area(circles: List<(c: Vec, r: F64)>, eps: F64 = 1e-9) -> F64  // O(n^2 log n), O(n) memory
struct HalfPlane(p: Vec, dir: Vec)
fn half_plane_intersect(planes: List<HalfPlane>, eps: F64 = 1e-9) -> List<Vec>          // O(n log n), O(n) memory
fn polygon_area(poly: List<Vec>) -> F64                                                 // O(n)
fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>           // O(n), O(n) memory
fn min_enclosing_circle(pts: List<Vec>, eps: F64 = 1e-9) -> (center: Vec, radius: F64)  // O(n) expected
fn circle_polygon_area(c: Vec, r: F64, poly: List<Vec>, eps: F64 = 1e-9) -> F64         // O(n)

struct Pt3(x: Int, y: Int, z: Int)                          // Add, Sub, Neg, Mul<Int>, Eq, Show
    @get fn norm2(self) -> Int                              // O(1)
    @get fn norm2_128(self) -> I128                         // O(1)
    @get fn len(self) -> F64                                // O(1)
fn dot3(a: Pt3, b: Pt3) -> Int                              // O(1)
fn dot3_128(a: Pt3, b: Pt3) -> I128                         // O(1)
fn cross3(a: Pt3, b: Pt3) -> Pt3                            // O(1)
fn dist2_3(a: Pt3, b: Pt3) -> Int                           // O(1)
fn convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)>  // O(n^2), O(n) memory
```

[new]:

```cp
    // Exact 2D point methods, distance metrics, and arbitrary-base angular order
    // (on Pt):
    /// Dot product of self and o. O(1).
    fn dot(self, o: Pt) -> Int  // O(1)
    /// 2D cross product of self and o. O(1).
    fn cross(self, o: Pt) -> Int  // O(1)
    /// Squared Euclidean distance from self to o. O(1).
    fn dist2(self, o: Pt) -> Int  // O(1)
    /// Euclidean distance from self to o. O(1).
    fn dist(self, o: Pt) -> F64  // O(1)
/// Manhattan (L1) distance |a.x - b.x| + |a.y - b.y|. O(1).
fn manhattan(a: Pt, b: Pt) -> Int  // O(1)
/// Chebyshev (L-infinity) distance max(|a.x - b.x|, |a.y - b.y|). O(1).
fn chebyshev(a: Pt, b: Pt) -> Int  // O(1)
/// Angular sort key in [base, base + 2*pi) CCW from ray base (zero first, I128). O(1).
struct ByAngleFrom(base: Pt, p: Pt)  // Ord, O(1)
/// True iff ray a precedes ray b CCW starting from ray `from` (default +x). O(1), I128.
fn angle_less(a: Pt, b: Pt, from: Pt = Pt(1, 0)) -> Bool  // O(1), I128

// Domain-indexed segment & half-hull twins, segment/ray intersections
/// Domain-indexed Shamos-Hoey sweep; returns a pair of intersecting segment handles. O(n log n), O(n) memory.
fn any_segments_meet_idx<D>(segs: [D](a: Pt, b: Pt), strict: Bool = false) -> ?(D, D)  // O(n log n), O(n) memory
/// Domain-indexed lower hull chain left-to-right on x-sorted pts. O(n), O(n) memory.
fn lower_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>  // O(n), O(n) memory
/// Domain-indexed upper hull chain left-to-right on x-sorted pts. O(n), O(n) memory.
fn upper_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>  // O(n), O(n) memory
/// Intersection of closed segments ab and cd: 0 disjoint, 1 crossing/touching, 2 collinear overlap endpoints. O(1).
fn segment_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> List<Vec>  // O(1)
/// First intersection of ray (o, dir) with closed segment ab, or none. O(1).
fn ray_hits_segment(o: Vec, dir: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> ?Vec  // O(1)

// Online 2D convex hull (construct with OnlineHull())
/// Incremental 2D convex hull of integer points backed by upper and lower ordered chains. O(n) memory.
struct OnlineHull
    /// Empty online convex hull. O(1).
    fn new() -> OnlineHull  // O(1)
    /// Insert point p into the hull; returns whether the hull changed. O(log n) amortized.
    fn add(mut self, p: Pt) -> Bool  // O(log n) amortized
    /// Point location against the current hull. O(log n).
    fn locate(self, p: Pt) -> Location  // O(log n)
    /// Hull vertex maximizing dot(v, dir), or none if empty. O(log n).
    fn extreme(self, dir: Pt) -> ?Pt  // O(log n)
    /// Twice the unsigned area of the current hull (0 if fewer than 3 non-collinear points). O(1).
    @get fn area2(self) -> Int  // O(1)
    /// Number of hull vertices. O(1).
    @get fn len(self) -> Int  // O(1)
    /// True when no points have been added. O(1).
    @get fn is_empty(self) -> Bool  // O(1)
    /// Current hull vertices in CCW order. O(n), O(n) memory.
    fn vertices(self) -> List<Pt>  // O(n), O(n) memory
    /// Remove all points from the hull. O(n).
    fn clear(mut self)  // O(n)

// Minkowski, convex/simple polygon operations, calipers, and proximity
/// Lower convex chain of {p + q} for non-empty left-to-right lower chains a and b. O(|a| + |b|), O(|a| + |b|) memory.
fn minkowski_lower(a: List<Pt>, b: List<Pt>) -> List<Pt>  // O(|a| + |b|), O(|a| + |b|) memory
/// True iff two non-empty CCW convex polygons share at least one point. O(|a| + |b|).
fn convex_polygons_overlap(a: List<Pt>, b: List<Pt>) -> Bool  // O(|a| + |b|)
/// Intersection of two CCW convex polygons (empty if disjoint). O(|a| + |b|), O(|a| + |b|) memory.
fn convex_polygon_intersect(a: List<Vec>, b: List<Vec>, eps: F64 = 1e-9) -> List<Vec>  // O(|a| + |b|), O(|a| + |b|) memory
/// Minimum width (distance between parallel supporting lines) of a CCW convex hull. O(n).
fn hull_width(hull: List<Pt>) -> F64  // O(n)
/// Indices of the two edges of a CCW convex hull crossed by line ab, or none. O(log n).
fn hull_line_intersect(hull: List<Pt>, a: Pt, b: Pt) -> ?(Int, Int)  // O(log n)
/// Farthest pair of distinct handles in pts and their squared distance. O(n log n), O(n) memory.
fn farthest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int)  // O(n log n), O(n) memory
/// O(n) candidate edges guaranteed to contain the Manhattan (L1) MST of pts. O(n log n), O(n) memory.
fn manhattan_mst<D>(pts: [D]Pt) -> List<(u: D, v: D, w: Int)>  // O(n log n), O(n) memory
/// Centroid (center of mass) of a non-degenerate simple polygon. O(n).
fn polygon_centroid(poly: List<Vec>) -> Vec  // O(n)
/// Kernel of a CCW simple polygon (points seeing the entire interior). O(n log n), O(n) memory.
fn polygon_kernel(poly: List<Vec>, eps: F64 = 1e-9) -> List<Vec>  // O(n log n), O(n) memory
/// Triangulate a CCW simple polygon into vertex index triples. O(n log n), O(n) memory.
fn polygon_triangulate(poly: List<Pt>) -> List<(Int, Int, Int)>  // O(n log n), O(n) memory
/// Number of strictly interior lattice points of a simple lattice polygon (Pick's theorem). O(n log max_coord).
fn pick_interior(poly: List<Pt>) -> Int  // O(n log max_coord)
/// Number of boundary lattice points of a simple lattice polygon. O(n log max_coord).
fn pick_boundary(poly: List<Pt>) -> Int  // O(n log max_coord)

// Careful floating-point Vec, Circle, and HalfPlane completions
/// Circle constant pi (canonical home: std.math.PI; re-exported here). O(1).
const PI: F64 = 3.14159265358979323846
    // (on Vec):
    /// Rotated 90 degrees clockwise. O(1).
    @get fn perp_cw(self) -> Vec  // O(1)
    /// Unit vector (cos(rad), sin(rad)). O(1).
    fn from_angle(rad: F64) -> Vec  // O(1)
    /// Dot product of self and o. O(1).
    fn dot(self, o: Vec) -> F64  // O(1)
    /// 2D cross product of self and o. O(1).
    fn cross(self, o: Vec) -> F64  // O(1)
    /// Squared Euclidean distance from self to o. O(1).
    fn dist2(self, o: Vec) -> F64  // O(1)
    /// Euclidean distance from self to o. O(1).
    fn dist(self, o: Vec) -> F64  // O(1)
/// Signed angle from a to b in (-pi, pi]. O(1).
fn angle_between(a: Vec, b: Vec) -> F64  // O(1)
/// 2D circle with center c and non-negative radius r.
struct Circle(c: Vec, r: F64)
    /// Unique circumcircle through a, b, c, or none if collinear. O(1).
    fn through(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle  // O(1)
    /// Area pi * r^2. O(1).
    @get fn area(self) -> F64  // O(1)
    /// Circumference 2 * pi * r. O(1).
    @get fn circumference(self) -> F64  // O(1)
    /// Point location relative to the disk. O(1).
    fn locate(self, p: Vec, eps: F64 = 1e-9) -> Location  // O(1)
/// Inscribed circle of triangle abc, or none if collinear. O(1).
fn incenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle  // O(1)
/// Area of the intersection of two disks (c1, r1) and (c2, r2). O(1).
fn circle_circle_area(c1: Vec, r1: F64, c2: Vec, r2: F64, eps: F64 = 1e-9) -> F64  // O(1)
/// Circle inversion of point p across circle (center, r). O(1).
fn invert(p: Vec, center: Vec, r: F64) -> Vec  // O(1)
    // (on HalfPlane):
    /// Half-plane keeping the closed left side of directed line a -> b. O(1).
    fn through(a: Vec, b: Vec) -> HalfPlane  // O(1)
    /// Half-plane keeping { (x, y) | a*x + b*y + c >= 0 }. O(1).
    fn from_line(a: F64, b: F64, c: F64) -> HalfPlane  // O(1)
    /// Signed orientation value dir.cross(q - p); >= 0 inside. O(1).
    fn side(self, q: Vec) -> F64  // O(1)
/// Half-plane intersection with optional [-bound, bound]^2 clipping box. O(n log n), O(n) memory.
fn half_plane_intersect(planes: List<HalfPlane>, bound: ?F64 = none, eps: F64 = 1e-9) -> List<Vec>  // O(n log n), O(n) memory

// Axis-aligned rectangles, planar subdivisions, and Delaunay / Voronoi
/// Half-open integer rectangle x * y (accepts cut ranges per decision 124).
struct Rect(x: Range<Int>, y: Range<Int>)
    /// Area of the rectangle (0 if empty). O(1).
    @get fn area(self) -> Int  // O(1)
    /// True when either axis range is empty. O(1).
    @get fn is_empty(self) -> Bool  // O(1)
    /// True when p.x in x and p.y in y. O(1).
    fn contains(self, p: Pt) -> Bool  // O(1)
    /// Non-empty intersection of two rectangles, or none. O(1).
    fn intersect(self, o: Rect) -> ?Rect  // O(1)
/// Total area of the union of axis-aligned integer rectangles via sweep line. O(n log n), O(n) memory.
fn rects_union_area(rects: List<Rect>) -> Int  // O(n log n), O(n) memory
/// Faces (CCW interior cycles and CW outer cycle) of a planar straight-line graph. O((n + m) log m), O(n + m) memory.
fn planar_faces<V>(pts: [V]Pt, edges: List<(V, V)>) -> List<List<V>>  // O((n + m) log m), O(n + m) memory
/// Offline sweep-line point location in a planar subdivision; returns face index or none. O((n + m + q) log(n + m)), O(n + m + q) memory.
fn point_location_planar<V>(pts: [V]Pt, edges: List<(V, V)>, queries: List<Pt>) -> List<?Int>  // O((n + m + q) log(n + m)), O(n + m + q) memory
/// Exact Delaunay triangulation of distinct integer points (|coords| <= 1e9) as CCW handle triples. O(n log n), O(n) memory.
fn delaunay<D>(pts: [D]Pt) -> List<(D, D, D)>  // O(n log n), O(n) memory
/// Voronoi cells of pts clipped to [-bound, bound]^2 via the Delaunay dual. O(n log n), O(n) memory.
fn voronoi_cells<D>(pts: [D]Pt, bound: F64) -> [D]List<Vec>  // O(n log n), O(n) memory

    // 3D exact & floating-point geometry
    // (on Pt3):
    /// Explicit conversion to 3D floating-point Vec3. O(1).
    @get fn to_vec3(self) -> Vec3  // O(1)
    /// 3D dot product of self and o. O(1).
    fn dot(self, o: Pt3) -> Int  // O(1)
    /// Exact 128-bit 3D dot product of self and o. O(1).
    fn dot128(self, o: Pt3) -> I128  // O(1)
    /// 3D cross product of self and o. O(1).
    fn cross(self, o: Pt3) -> Pt3  // O(1)
    /// Squared 3D Euclidean distance from self to o. O(1).
    fn dist2(self, o: Pt3) -> Int  // O(1)
    /// 3D Euclidean distance from self to o. O(1).
    fn dist(self, o: Pt3) -> F64  // O(1)
/// Domain-indexed 3D convex hull returning outward-oriented triangular faces of D handles. O(n^2), O(n) memory.
fn convex_hull_3d_idx<D>(pts: [D]Pt3) -> List<(D, D, D)>  // O(n^2), O(n) memory
/// Six times the signed volume of a closed triangulated polyhedron. O(|faces|).
fn polyhedron_volume6(pts: List<Pt3>, faces: List<(Int, Int, Int)>) -> I128  // O(|faces|)
/// 3D floating-point point or vector (Add, Sub, Neg, Mul<F64>, Div<F64>; no Show).
struct Vec3(x: F64, y: F64, z: F64)
    /// Squared Euclidean length. O(1).
    @get fn norm2(self) -> F64  // O(1)
    /// Euclidean length. O(1).
    @get fn len(self) -> F64  // O(1)
    /// Unit vector in the same direction (traps if len == 0). O(1).
    @get fn unit(self) -> Vec3  // O(1)
    /// 3D dot product. O(1).
    fn dot(self, o: Vec3) -> F64  // O(1)
    /// 3D cross product. O(1).
    fn cross(self, o: Vec3) -> Vec3  // O(1)
    /// Squared 3D Euclidean distance. O(1).
    fn dist2(self, o: Vec3) -> F64  // O(1)
    /// 3D Euclidean distance. O(1).
    fn dist(self, o: Vec3) -> F64  // O(1)
    /// Rodrigues rotation around unit axis by rad radians. O(1).
    fn rot(self, axis: Vec3, rad: F64) -> Vec3  // O(1)
/// 3D plane { p | n.dot(p) == d } with unit normal n.
struct Plane3(n: Vec3, d: F64)
    /// Plane through non-collinear points a, b, c, or none. O(1).
    fn through(a: Vec3, b: Vec3, c: Vec3, eps: F64 = 1e-9) -> ?Plane3  // O(1)
    /// Signed distance from p to the plane. O(1).
    fn dist(self, p: Vec3) -> F64  // O(1)
    /// Orthogonal projection of p onto the plane. O(1).
    fn project(self, p: Vec3) -> Vec3  // O(1)
/// Intersection of line ab with plane, or none if parallel. O(1).
fn line_plane_intersect(a: Vec3, b: Vec3, plane: Plane3, eps: F64 = 1e-9) -> ?Vec3  // O(1)
/// Great-circle distance between directions a and b on a sphere of radius r. O(1).
fn sphere_dist(a: Vec3, b: Vec3, r: F64 = 1.0) -> F64  // O(1)
```

Read-only sequence parameters (`List<Pt>`, `List<Vec>`) generalize to
`S: Indexed<Item = Pt>` / `S: Indexed<Item = Vec>` once associated-type
protocols land (`design/stdlib/geometry.md` `## Powers`), so domain arrays
`[D]Pt` and half-open cut-range slices `poly[j..i]` (decision 124) work with
zero copies and no `_range` twins.

Matched C++17 benchmarks (`python3 bench/geometry/run.py --runs 7`, all 16 <= 1.09x):
`convex_hull` (200k pts) 1.06x (35 vs 33 ms), `half_plane_intersect` (100k planes) 0.81x (56 vs 68 ms),
`closest_pair` (100k pts) 1.07x (30 vs 28 ms), `any_segments_meet` (100k segs) 1.06x (129 vs 122 ms),
`convex_hull_3d` (20k pts) 1.09x (28 vs 25 ms), `minkowski_sum` (2×50k ×20) 0.83x (25 vs 30 ms),
`diameter2` (100k hull ×50) 1.02x (94 vs 92 ms), `locate_convex` (50k hull, 200k q) 0.95x (42 vs 45 ms),
`locate` (2k star poly, 20k q) 0.97x (49 vs 50 ms), `polygon_area` (100k poly ×100) 0.98x (21 vs 22 ms),
`circle_intersect` (100k pairs) 0.53x (42 vs 79 ms), `polygon_cut` (4k cuts) 0.94x (4 vs 4 ms),
`min_enclosing_circle` (200k pts) 0.61x (36 vs 59 ms), `min_bounding_box` (100k hull ×50) 1.05x (108 vs 103 ms),
`circle_tangents` (100k pairs) 0.54x (43 vs 79 ms), `circles_union_area` (2k circles) 1.04x (10 vs 10 ms).

### 4.15 `std.grid` and `std.spec`

`std.grid` gives the four and eight directions and in-bounds stepping
over a `(r, c)` domain; `std.spec` the tools for checking observations
on small cases (monotonicity, convexity, Monge, recurrence guessing).

```cp
struct Dir4(k: Int)
    fn up() -> Dir4
    fn right() -> Dir4
    fn down() -> Dir4
    fn left() -> Dir4
    fn all() -> List<Dir4>
    fn from(letters: Str, ch: Char) -> ?Dir4
    @get fn cw(self) -> Dir4
    @get fn ccw(self) -> Dir4
    @get fn back(self) -> Dir4
    @get fn delta(self) -> (Int, Int)
struct Dir8(k: Int)
    fn all() -> List<Dir8>
    @get fn cw(self) -> Dir8
    @get fn ccw(self) -> Dir8
    @get fn back(self) -> Dir8
    @get fn delta(self) -> (Int, Int)
fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C)        // O(1)
fn ray<R, C, D>(self: (R, C), d: D) -> List<(R, C)>    // O(length of the ray)
@get fn adj4<R, C>(self: (R, C)) -> List<(R, C)>       // O(1)
@get fn adj8<R, C>(self: (R, C)) -> List<(R, C)>       // O(1)
@get fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T  // O(|R| * |C|)
fn decrease<T>(xs: List<T>) -> ?Int                    // O(n)
fn increase<T>(xs: List<T>) -> ?Int                    // O(n)
fn convexity_break(xs: List<Int>) -> ?Int              // O(n)
fn concavity_break(xs: List<Int>) -> ?Int
fn unimodality_break(xs: List<Int>) -> ?Int  // O(n)
// O(rows * cols)
fn monge_break(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> ?(Int, Int)
// O(rows * cols)
fn argmins(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> List<Int>
fn dnc_break(cost: (Int, Int) -> Int, n: Int) -> ?(Int, Int)   // O(n^2)
fn knuth_break(opt: (Int, Int) -> Int, n: Int) -> ?(Int, Int)  // O(n^2)
fn differences(xs: List<Int>) -> List<Int>                     // O(n)
fn newton(xs: List<Int>) -> List<Int>                          // O(n^2)
fn poly_degree(xs: List<Int>) -> ?Int                          // O(n^2)
fn poly_at(xs: List<Int>, n: Int) -> Int                       // O(len^2)
fn berlekamp_massey<P>(xs: List<Mod<P>>) -> List<Mod<P>>
// O(k^2 log n)
fn recurrence_at<P>(c: List<Mod<P>>, init: List<Mod<P>>, n: Int) -> Mod<P>
fn guess_at<P>(xs: List<Mod<P>>, n: Int) -> ?Mod<P>
```

### 4.16 `std.dp` [new]: the DP toolkit

The structures and optimizations a DP calls by name; the recurrence stays
in user code, and memoized recursion is `@memo` (core). Each optimization
takes the cost as an operation and states its law, which `std.spec` checks
(`monge_break`, `dnc_break`, `knuth_break`, `convexity_break`). Design,
laws and worked problems: `spec/lib/rationale/dp.md`.

```cp
// sliding-window optimum: std.window.MonoDeque (4.8)
struct Line(k: Int, m: Int)
    fn at(self, x: Int) -> Int  // k * x + m
struct LiChao<L, K>             // convex hull trick, any L.at -> K
    fn new(pick: (K, K) -> K, xs: Range<Int>) -> LiChao<L, K>
    fn add(mut self, l: L, on: ?Range<Int> = none)  // O(log n), O(log^2 n) on a segment
    fn at(self, x: Int) -> ?K                       // O(log n)
    fn line_at(self, x: Int) -> ?L                  // O(log n)
    fn clear(mut self)
fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int)
    -> List<?(val: Int, at: Int)>  // O((rows + cols) log rows) calls
fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int>
// O(n log n) calls; next[i] = min over j < i of prev[j] + cost(j, i)
fn monge_path<K>(n: Int, w: (Int, Int) -> K) -> (val: List<K>, prev: List<Int>)
// O(n) calls (LARSCH); shortest 0 -> i over Monge edges j -> i, j < i
fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>  // O(n^2)
fn aliens<L>(k: Int, lambdas: Range<L>, solve: L -> (val: L, count: Int))
    -> L                                    // O(log) calls of solve
fn subset_sums(xs: List<Int>) -> List<Int>  // O(2^n), sorted
// sums over subsets / supersets: std.ntt.sos(mut f, over: Sub | Sup = Sub, inverse: Bool = false) (4.5)
```

#### `std.slope` [new]: the slope trick

A concave (or convex) function on the integers `lo..=hi`, kept as its value
at `lo` plus its slopes in a lazy treap. Laws and the worked port:
`spec/lib/rationale/dp.md` §6.7.

```cp
struct Concave
fn Concave.point(d: Int) -> Concave        // O(1); 0 at d only
fn Concave.flat(r: Range<Int>) -> Concave  // O(len log len) expected; 0 on r
fn reserve(mut self, n: Int)               // O(n); room for n slopes, no regrowth
fn max_convolve(mut self, g: Concave)      // O(|g| log n) expected; merge slopes
fn add_quadratic(mut self, q: Int -> Int)  // O(1); q quadratic, q'' <= 0
fn at(mut self, c: Int) -> Int             // O(c - lo + log n) expected
fn max(mut self) -> Int                    // O(n)
struct Convex                              // same, with min_convolve, min and q'' >= 0
```

#### DP and search: the complete surface [new]

Everything the area should offer and does not yet, at signature level.
Coverage audit, conventions, cross-area names, the powers these need and
the dreams that use them: `design/stdlib/dp.md` (Coverage, Conventions,
Cross-area, Powers, Dreams). Construction is
one convention across the area: the type called like a function
(`LiChao<Line, Int>(min, 0..n)`, `MonoDeque<Int>(pick: max)`); `T.new` is
the explicit spelling and `T.point`/`T.flat` name the alternative forms. A
binary chooser such as `min` is `pick:`, a key is `by:`, a predicate is
`keep:`, a starting value is `init:`.

```cp
// Search on answers (std.seq)
fn gallop_first<I>(from: I, ok: I -> Bool) -> ?I
// O(log d) calls, d the distance to the answer: the first i >= from
// with ok(i), ok false then true and no upper end known; none past I's end
fn dinkelbach(lo: F64, solve: F64 -> (num: F64, den: F64), eps: F64 = 1e-9) -> F64
// O(log(1/eps)) calls, superlinear in practice: the largest num / den,
// where solve(l) returns the choice maximizing num - l * den, den > 0, lo below
fn bsearch_first<I>(r: Range<I>, ok: I -> Bool, warm: ?I = none) -> ?I
// [new] warm start: O(log d) calls from a guess at distance d

// Optimizations, cut forms (decision 124): ranges of D are the pieces
// between two cuts, results are indexed by the cuts D.bounds
fn knuth<D, K>(cost: Range<D> -> K) -> Knuth<D, K>  // O(n^2) calls, O(n^2) memory
struct Knuth<D, K>
    fn best(self, piece: Range<D>) -> K           // O(1); the optimum on piece
    fn split(self, piece: Range<D>) -> ?D.bounds  // O(1); its best split, none for one position
fn dnc_layer<D, K>(prev: [D.bounds]?K, cost: Range<D> -> K) -> [D.bounds]?K
// O(n log n) calls; next[i] = min over j < i of prev[j] + cost(j..i)
fn monge_path<D, K>(w: Range<D> -> K) -> (val: [D.bounds]K, prev: [D.bounds]?D.bounds)
// O(n) calls (LARSCH); shortest D.bounds.first -> i over Monge pieces
struct MongePath<D, K>                             // the online form
    fn new(w: Range<D> -> K) -> MongePath<D, K>    // O(1); val[first] = K.zero
    fn next(mut self) -> (val: K, prev: D.bounds)  // O(1) amortized calls of w; the next cut's optimum
fn smawk(rows: Int, cols: Int, f: (Int, Int) -> Int) -> List<(val: Int, at: Int)>
// O(rows + cols) calls; leftmost row minima of a totally monotone matrix
fn min_plus_convolve(a: List<Int>, b: List<Int>, convex: Shape = Both) -> List<Int>
fn max_plus_convolve(a: List<Int>, b: List<Int>, concave: Shape = Both) -> List<Int>
// c[k] = best over i + j = k of a[i] + b[j]; Both: O(n + m) by merging
// differences; Right (only b shaped): O((n + m) log n) by monotone minima;
// Neither: O(n m)
enum Shape: Both | Right | Neither

// Convex hull trick, online
struct MonotoneHull  // lines in slope order
    fn new(pick: (Int, Int) -> Int = min) -> MonotoneHull
    fn add(mut self, l: Line)             // O(1) amortized; slopes monotone for pick
    fn at(self, x: Int) -> ?Int           // O(log n)
    fn at_next(mut self, x: Int) -> ?Int  // O(1) amortized; x never decreases between calls
    @get fn len(self) -> Int
    fn clear(mut self)
struct LineContainer  // any order, no coordinate range
    fn new(pick: (Int, Int) -> Int = min) -> LineContainer
    fn add(mut self, l: Line)          // O(log n) amortized
    fn at(self, x: Int) -> ?Int        // O(log n)
    fn line_at(self, x: Int) -> ?Line  // O(log n)
    @get fn len(self) -> Int

// Subsequences and subsets
fn lis<D, T, K>(xs: [D]T, strict: Bool = true, by: T -> K = x => x) -> List<D>
// O(n log n); the positions of one longest increasing subsequence
fn lis_len<D, T, K>(xs: [D]T, strict: Bool = true, by: T -> K = x => x) -> [D]Int
// O(n log n); the longest increasing subsequence ending at each position
fn submasks(m: Int) -> yields Int                  // O(2^popcount(m)); m down to 0
fn supermasks(m: Int, bits: Int) -> yields Int     // O(2^(bits - popcount(m))); m up to 2^bits - 1
fn masks_of_size(bits: Int, k: Int) -> yields Int  // O(C(bits, k)); increasing (Gosper)
fn subset_folds<T, M>(xs: List<T>, op: M) -> List<T>
// O(2^n), O(2^n) memory; out[mask] = fold of xs[i] for i in mask, out[0] = op's identity
fn meet_in_middle<T, K>(xs: List<T>, op: (T, T) -> T, join: (List<T>, List<T>) -> K) -> K
// O(2^(n/2) log) for a sorted join; subset_folds of each half, both sorted, then join

// The slope trick, completed (std.slope): Convex shown, Concave mirrors with max
fn add_left(mut self, a: Int)           // O(log n) expected; adds max(0, a - x)
fn add_right(mut self, a: Int)          // O(log n) expected; adds max(0, x - a)
fn add_abs(mut self, a: Int)            // O(log n) expected; adds |x - a|
fn add_const(mut self, c: Int)          // O(1)
fn add_linear(mut self, k: Int)         // O(1) lazy; adds k * x
fn prefix_min(mut self)                 // O(log n) expected; f(x) = min over y <= x of f(y)
fn suffix_min(mut self)                 // O(log n) expected; f(x) = min over y >= x of f(y)
fn shift(mut self, d: Int)              // O(1) lazy; f(x) = f(x - d)
fn slide_min(mut self, a: Int, b: Int)  // O(log n) expected; f(x) = min over x - b <= y <= x - a of f(y)
fn argmin(mut self) -> Range<Int>       // O(log n) expected; where the minimum is attained
@get fn domain (self) -> Range<Int>     // O(1)

// Law checkers (std.spec), the sampled form of each law above
fn total_monotone_break(f: (Int, Int) -> Int, rows: Int, cols: Int) -> ?(Int, Int)  // O(rows cols)
fn quadrangle_break<D, K>(cost: Range<D> -> K) -> ?(Range<D>, Range<D>)             // O(n^4) on small n

// Search that steers its own generator (decision 126)
fn backtrack<S>(start: S, next: S -> yields S) -> yields Visit<S>
// O(visited); each Visit has .state, .depth and .prune(), which skips its
// subtree; stop with break or return in the for body (decision 126)
struct Visit<S>
    @get fn state(self) -> S
    @get fn depth(self) -> Int
    fn prune(self)
```

[rename] once decision 124 lands: the cut forms above replace the `n: Int`
forms of `knuth`, `dnc_layer` and `monge_path` (no overloading by argument
type, decision 127); `LiChao.new(pick,
xs)` and `MonoDeque.new(by:)` are spelled `LiChao<L, K>(min, xs)` and
`MonoDeque<T>(pick: min)`, `by:` there renamed `pick:` (it chooses, it is
not a key).

### 4.17 `std.derive`: field-wise traits

The `Eq`, `Ord` and `Show` a struct gets field by field, written on
`fields` (decision 28.7), for an `impl` to call:
`impl Ord for Job: fn lt(a, b) = lt_fields(b, a)` orders the other way.

```cp
fn eq_fields<T>(a: T, b: T) -> Bool    // O(fields)
fn lt_fields<T>(a: T, b: T) -> Bool    // O(fields)
fn show_fields<T>(a: T) -> Str         // O(fields)
fn read_fields<T>() -> T               // O(fields): T(read() for _ in fields(T))
trait Hash: fn hash(self) -> U64       // Int, U64, Char, Bool, Str here (decision 28.8)
fn hash_fields<T>(a: T) -> U64         // O(fields)
trait Gen: fn gen(mut r: Rng) -> Self  // Int 0..=9, Bool, Char 'a'..='z' (decision 28.12)
fn gen<T: Gen>(mut r: Rng) -> T        // O(1); T from the type wanted
fn gen_fields<T>(mut r: Rng) -> T      // O(fields)
```

Not yet in `derive.cp`: `Gen`/`gen`/`gen_fields` (`Gen` is a compiler
table name today; `GAPS.md`, Q3). `Hash` is std's own (decision 256).
`impl Hash for P: fn hash(self) = hash_fields(self)` derives a struct's
`Hash` from its fields'; `impl Gen for P: fn gen(mut r: Rng) -> P =
gen_fields(mut r)` derives a stress generator the same way.

## 5. Not here yet

Power-series algebra beyond products, suffix automata, link-cut trees,
persistent structures, wavelet trees, 2-SAT and the rest of `examples/local-lib/` are staged there, not
excluded: they arrive in the order the ports need them, after the [new]
items above. Until then they are the standing test that a user library
gets the stdlib's powers.
