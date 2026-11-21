# Spec v1 skeleton: decisions (to be expanded into SPEC.md)

Principle: the obvious way to write it is the correct and fast way. Every line asks "could this compile and
be wrong?" Small general core; contest shapes live in the stdlib; nothing is chosen secretly (representation
follows the declared type; algorithms are named).

## Core language
1. Lexical: indentation blocks after `:`, spaces only; `//` comments; snake_case values/fns, PascalCase
   types/domains (parser uses this for `Name<`); `1e9`, `2e5`, `1_000` are Int when exact; every string
   literal interpolates `{x}` with printf-style specs `{x:.6}`.
2. Bindings: `let` immutable, `var` mutable; `fn f(a: T) -> R:` block (last expr is value) or `= expr`;
   lambdas `x => e`; `do:` block expression; `return` only for early exit.
3. Types: Int (i64, Int.MIN = -(2^63-1), spare value is ?Int's niche); range types `lo..=hi` are refinements
   that pick storage and prove overflow (dependent on earlier values allowed: `1..=h - 1`); Bit, Bool, Char,
   Str (ASCII bytes, O(1) index), Float; I32/U64/I128 for bit work; `?T` with `!`, `??`, `?`, `if let`,
   `while let`, `match ... some(x)/none`; tuples and named tuples `(lo: Int, hi: V)`; struct (fields+methods,
   generated ctors, `with` update); enum (`A | B`, payloads); `Mod<P>`.
4. Domains: nominal typed index spaces. Forms: `domain V = 1..=n`; `0..<n`; derived `I.bounds` (cuts between
   elements; `x.before`, `x.after`, `b.count`); products `(Row, Col)`; sums `A + B`; stdlib shapes
   `grid(r, c)` (.adj4/.adj8), `ring(n)` (wrapping +, .next/.prev, `V.arc(from:, until:/through:)`),
   `heap_tree(levels:)` (.children/.parent/.depth); fresh domains `xs.indices`, `compress(xs)`.
   Domain arithmetic `x + k` is checked (traps out of range); `x.offset(k) -> ?X`; `X.clip(range)` clips
   an Int range into the domain. (Open: sharper cut arithmetic.)
5. Arrays `[D]T`, `[A, B]T` indexed `a[i, j]`; tables `[D] i => e`; List<T>; slices `a[r]` are views.
6. Ranges `a..b`, `a..=b`, `by k`, `.rev`, `.first/.last/.mid/.len`, `&` intersection, `clipped(r)`.
7. Control: inline `if c: a else: b`; match with guards and patterns; `for x in xs if cond:`; `while`,
   `loop`, `while let p = e if guard:`; `repeat n:`; compound `min=` / `max=`.
8. Ownership: variables own; params borrow; mutation is `mut` at declaration and call site; big values
   copy only via `.copy()`; implicit move at last use; no stored references; identity via Pool handles.
9. Abstraction: traits for one canonical instance (Ord, Add, Show, Read); operation values for many
   (`SegTree<V>(max, update: add)`, `Heap(by: min)`); generics monomorphized; `@get` getters.
10. Program structure: the file is a function body. File decorators change how it is driven, never what it
    means: `@multitest` (whole file per test; `static let` runs once), `@interactive` (`ask(line, as: T)`
    prints+flushes+reads; `answer(line)`; reading -1 exits), `@grader` (`export fn` matched to grader names;
    state sized in init: open). Library files have no top-level statements.
11. I/O: `read()` typed by expected type (records, arrays of domains, grids `read(digits)`), validates
    ranges; `print` any value (lists space-separated), `print(x, layout: lines)`.
12. Testing hooks: `@brute(f)`; generators derived from input types; tooling spec separate.

## Stdlib shapes (named algorithms)
Sequences: prefix_sums(..).sum(range), bisect(range, pred) / bisect_index, longest_window(seq, ok:),
windows(k), runs(a, by:), sorted/sort(by:), compress, argmax/max_by, sum/count/any/all, enumerate, zip.
Collections: List, Deque, SortedSet/SortedMap (ordered; `.set(x, flag)`, first_ge/last_le), HashMap (explicit),
Heap(by:), BitSet, Pool. Algebra: Mod<P> (`falling`, `fact`, `binom` tables), op values sum/min/max/xor/mul,
SegTree(op, update:), Fenwick, SparseTable, Dsu. Graphs/trees: Graph<V>.undirected/directed, Tree<V>(edges)
.rooted(at:) (parent, depth, preorder, down_edges, subtree_sizes), bfs(from:, next:) -> [D]?Int, dijkstra,
toposort, scc, flow, matching. Strings: z, kmp, suffix_array, hashing. Math: gcd, primes, ntt/convolve, Poly,
Matrix. Geometry: Point<T>, cross, hull.
