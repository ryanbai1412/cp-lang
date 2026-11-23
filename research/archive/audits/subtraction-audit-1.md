# Stage 3: Subtraction & Concept-Budget Audit #1

*Adversarial simplification audit of `research/archive/architecture/language-architecture-synthesis.md`, `decisions.md`, `plan/19-index-and-lends.md`, `plan/11-20`, `crates/cpl-syntax/src/token.rs`, `stdlib/*.cp`, and all 299 `.cp` ports in `proto/ports/`.*

The test for every feature in CP Lang is blunt: can a strong competitive programmer memorize the entire language from a single cheat sheet in 15 minutes at 2 AM and write clean, fast, zero-UB solutions on the first compile without fighting the compiler?

The good news is that CP Lang's core architecture is already unusually coherent. Sealed `Index`/`Seq` domains, `[I]T` arrays, `?T` optionals, lexical `lends`, unified `Mod<P>`, algebraic `Index` derivation, and flat `@memo` form a tight, orthogonal foundation. Almost all of the cognitive weight in the current spec comes from three kinds of accretion that crept in across 16 scout batches and 20 design docs:

1. **Zombie keywords and duplicate spellings.** The lexer (`crates/cpl-syntax/src/token.rs`) and `plan/` docs reserve keywords that appeared **zero times** across 299 ported `.cp` solutions (`with`, `by`, `mut fn`, `static let`, `<=>`, `;`, `requires`/`ensures`/`invariant`, `dyn`, `pool`), while several core constructs still carry two or three incompatible spellings across docs (`if c then a else b` vs `if c: a else: b` vs `a if c else b`, `v.before` vs `v.l`, `v.offset(k)` vs `v + k`, `ModR` vs `Mod<M>`, `BitSet` vs `Bitset`, `first_where` vs `bsearch_first`, `pairs(V)` meaning both all unordered pairs and adjacent pairs).
2. **Speculative abstraction layers built for 1–2% problems.** Proposals like `[D]E: Index` exponential array indexing, `Pool<Node<_>>` recursive type handles, `Window<I>`/`BagWindow<I>` trait hierarchies, six extra compiler representation traits, twenty algebraic wrapper types in `stdlib/algebra.cp`, and a five-construct `protocol` DSL for interactive problems add heavy conceptual machinery for problems that occur in 0.5–3% of contests.
3. **Standard-library sprawl.** Accepting every helper requested by any single scout batch balloons `std` past 45 data structures and 19 search names. Goal G8 exists specifically so user local libraries (`examples/local-lib/`) have the exact same zero-cost power as `std` — keeping `std` small and memorizable while specialized structures live in personal libraries.

Applying the 18 concrete cuts below reduces the language's concept budget dramatically without giving up CP Lang's safety, speed, or expressiveness:

- **Keywords:** from 55 across `token.rs` (36) plus proposed/contextual keywords (19) down to **40 total** — **32 contestant keywords** (including the 3 literals `true`, `false`, `none`; single-file contest solutions use ~27, fewer than Python's 35) plus **8 library-author keywords** (`trait`, `impl`, `where`, `lends`, `lend`, `law`, `inline`, `yield`). Fifteen keywords are deleted (`by`, `with`, `undo`, `requires`, `ensures`, `invariant`, `stress`, `protocol`, `limit`, `dyn`, `do`, `pool`, `decorator`, `static`, `yields`).
- **Numeric types:** from 8 (`Int`, `I32`, `I64`, `U8`, `U32`, `U64`, `I128`, `F64`) down to **4** (`Int` with range types for compact storage, `U64`, `I128`, `F64`).
- **User-visible traits:** from ~26 proposed down to **8** core value/index traits (`Index`, `Seq`, `Copy`, `Eq`, `Ord`, `Hash`, `Show`, `Read`) plus **3** library-author traits (`Indexed`/`IndexedMut`, `Iterator`, `Gen`).
- **Predicate and contract constructs:** from 6 (`assert`, `assume`, `where` refinements, `requires`, `ensures`, `invariant`, plus `stress` read clauses) down to **2** (`assert` and `assume`).
- **Contest `@` decorators:** from 18 plus `protocol` down to **7** (`@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`/`@interactor`, `@grader`), plus 2 library-only (`@get`, `@test`).
- **Algebraic wrappers:** from 20 names in `stdlib/algebra.cp` down to bare operators (`+`, `min`, `max`, `^`, `gcd`), `(id, op)` pairs, `desc(x)`, and `Action` for lazy trees.
- **Search API:** from 19 names down to **7** (`bsearch_first`, `bsearch_last`, `first_ge`, `last_le`, `first`, `find`, `min_by`/`max_by`).

Every surviving construct is captured in the companion canonical reference, `docs/one-page-cheat-sheet.md`.

---

## Part I: Kernel and Syntax Deletions and Merges

### Cut 1: Delete the `with` keyword (struct update AND scope guards) and `undo` blocks — keep only `defer`

Three separate mechanisms currently compete for scoped cleanup and mutation across `decisions.md`, `plan/14`, and `research/archive/architecture/language-architecture-synthesis.md`: `defer stmt` (5.7), `undo dsu, best: ...` (5.8), and `with placed(r, c, d): ...` built on a special `lends ()` projection (synthesis §2.3D, `L14`). Worse, `with` is *already* overloaded three ways: struct functional update (`e with w: 7` in `plan/17`), the proposed scope-guard statement, and the method name `.with(x)` on `Subset`.

A scan of all 299 `.cp` files in `proto/ports/` and 19 files in `stdlib/` settles this cleanly:

- Struct functional update `e with w: 7` appears **0 times** in 299 ports. All 12 occurrences of `with` in `proto/ports/` are `.with(x)` method calls on `Subset` (`fear.with(...)`, `s.with(i)`), which reserving `with` as a hard keyword in `token.rs` actively conflicts with.
- `undo` as a block keyword appears **0 times** in 299 ports — its only 2 occurrences (`sdoi17r1p1.cp`) are a contestant naming a local variable `let undo = ...`.
- The proposed `with placed(...):` scope guard requires weakening `plan/19 §3.4`'s clean rule that every path through a `lends` body reaches *exactly one* `lend`, solely so `cco02p3`-style interactive wall checks can skip lending when a move fails.

Deleting `with` (both forms) and `undo` leaves **one** scope-cleanup primitive — `defer` — which adds zero nesting depth, needs no helper function, never bends `lends`, and covers backtracking, DSU rollback, and interactive moves uniformly.

Before:
```cp
// Three different mechanisms + a `lends ()` rule exception
fn placed(r: Line, c: Line, d: Digit) lends ():
    mark(r, c, d, true)
    lend ()
    mark(r, c, d, false)

with placed(r, c, d):
    fill(score + d.label * worth(r, c))

undo dsu:
    dsu.unite(a, b)
    solve(child)

let e2 = e with w: 7
```

After:
```cp
// One mechanism (`defer`), zero extra indentation, `Rollback` stays a 2-method struct API
mark(r, c, d, true)
defer mark(r, c, d, false)
fill(score + d.label * worth(r, c))

let snap = dsu.mark()
defer dsu.undo_to(snap)
dsu.unite(a, b)
solve(child)

var e2 = e
e2.w = 7
```

For `cco02p3`'s conditional step, `if not step(d): continue` followed by `defer step(d.back)` handles early exit naturally with zero special `lends ()` rule.

**Cost:** One extra line when cloning a struct with a tweaked field (used 0 times in 299 ports) and two lines (`let snap = dsu.mark(); defer dsu.undo_to(snap)`) instead of `undo dsu:` for rollback DSU (used in < 1% of problems). **Gain:** Frees two keywords (`with`, `undo`), removes a parser clash with `s.with(x)` and `let undo`, and keeps `lends` obeying one unbroken law.

---

### Cut 2: Collapse the 6-way predicate and contract zoo into `assert` and `assume`

Across `plan/18`, `decisions.md` (7.1, 7.9, 7.19), and `synthesis §4`, six overlapping predicate/constraint constructs have accumulated: `assert cond`, `assume cond`, `read() where cond`, `requires cond`, `ensures cond`, `invariant cond`, plus a new `stress x in 0..=3 by 0.25` grammar clause (`Q6`). In 299 `.cp` ports and 19 `stdlib/*.cp` files, `requires`, `ensures`, and `invariant` appear **0 times**.

Look at what `requires`, `ensures`, `invariant`, and `read() where cond` actually do: they check a condition in local debug/stress runs, drop the check in judge builds, and feed range/flow facts to the checker. That is *word for word* the semantics of `assume` from `synthesis §4 (Q1)`! Meanwhile, type-level `where` refinements overload the generic trait-bound keyword `where`, still cannot express cross-variable relations across two separate `read()` calls, and fail whenever the condition depends on an earlier array.

Collapse all six into **two** constructs with unmistakable runtime semantics:

- `assert cond` — checked in **both** local and judge builds; feeds proven flow facts to the range checker. (Resolves the contradiction between `decisions 7.1` and `plan/17` line 98.)
- `assume cond` — local-only promise: checked in debug/stress runs, stripped in judge builds, filters derived stress inputs, validates custom `gen:` output, and feeds local warnings (never used to unsafely strip a judge bounds check). Put `assume` at the top of a function instead of `requires`, inside a loop instead of `invariant`, before returning instead of `ensures`, and right after `read()` instead of `where`.

For `stress x in 0..=3 by 0.25` (`Q6`), delete the `stress` syntax clause entirely: `read(gen: ...)` already handles custom distributions without adding a keyword and step grammar to `read()`.

Before:
```cp
let qs: List<(l: V, r: V) where l <= r> = read()
let x: F64 = read() stress x in 0..=3 by 0.25

fn query(st: SegTree<V, Int>, l: V, r: V) -> Int
    requires l <= r
    ensures result >= 0:
    var i = l.to_idx
    while i <= r.to_idx:
        invariant i >= l.to_idx
        ...
```

After:
```cp
let qs: List<(l: V, r: l..=V.last)> = read()      // dependent bound when simple
// or for cross-variable constraints:
let (a, b): (Int, Int) = read()
assume a <= b and a + b <= 1_000_000

let x: F64 = read(gen: () => rand(0, 12) * 0.25)

fn query(st: SegTree<V, Int>, l: V, r: V) -> Int:
    assume l <= r
    ...
```

**Cost:** Contract preconditions/postconditions sit inside the function body rather than in the header signature, so doc generators do not display them separately (for contest code, nobody runs a doc generator; for stdlib, doc comments state preconditions and `assume` enforces them in tests). **Gain:** Deletes four keywords/clauses (`requires`, `ensures`, `invariant`, `stress`), stops overloading `where` in types, preserves 100% of Ryan's approved contracts and stress filtering, and leaves a 2-word rule: `assert` always runs; `assume` is local-only.

---

### Cut 3: Delete `mut fn` — infer captured writes on ordinary nested `fn`

`decisions 5.10` and `plan/14` require writing `mut fn dfs(v: V):` whenever a nested function mutates an outer `var`. Yet `plan/19 §11` ("Captured variables and calls") *already* walks the call graph of every nested `fn` to compute its exact read and write sets over outer `var`s so the borrow checker can forbid calling it while any captured `var` is lent. Meanwhile, `mut fn` appears **0 times** in 299 ports and 19 stdlib files, and Blind Critic #1 (Finding 5.6) hit an immediate compile error trying to write `mut fn push(...)` from the docs.

Adding a fifth meaning to `mut` (mutable param, mutable call argument, `mut self`, `lends mut`, and `mut fn`) creates pure contest friction: adding a single `cnt += 1` inside a local helper forces editing its signature to add `mut`.

Delete `mut fn`. The surviving rule is one sentence: any `fn` declared after a `let` can read it; any `fn` declared after a `var` can read and write it, cannot escape that scope, and its captured borrows are tracked automatically by the compiler. `@memo` already checks purity independently.

Before:
```cp
var vis: [V]Bool = false
mut fn dfs(u: V):
    vis[u] = true
    for v in g[u]:
        if not vis[v]: dfs(v)
```

After:
```cp
var vis: [V]Bool = false
fn dfs(u: V):
    vis[u] = true
    for v in g[u]:
        if not vis[v]: dfs(v)
```

**Cost:** A reader scanning a local helper's signature does not see a `mut` tag signaling outer-`var` writes (the body is usually 5–15 lines right below it). **Gain:** Fixes Blind Critic Finding 5.6, removes a redundant declaration form, and eliminates a frequent mid-contest compiler fight.

---

### Cut 4: Delete the `by` keyword (`0..n by 2`) — keep only `.step(k)`

`crates/cpl-syntax/src/token.rs` reserves `by` as a hard keyword for stepped ranges (`0..n by 2`), while `synthesis §2.3B (N8)` also proposes `.step(k)` as a method on `Range`. Across 299 `.cp` ports and 19 `stdlib` files:

- Range-step `.. by` appears **0 times** (two regex hits were `by:` arguments after a range in a comprehension).
- `by:` as a named argument (`sorted(xs, by: ...)`, `argmax(..., by: ...)`, `runs(a, by: ...)`) appears **47 times across 36 files**.

Reserving `by` as a keyword forces special parser exceptions so `by:` works as an argument label, and gives two spellings for the exact same operation. Delete the `by` keyword and keep `.step(k)`.

Before:
```cp
for j in (2 * p)..=n by p:
    composite[j] = true
```

After:
```cp
for j in (2 * p..=n).step(p):
    composite[j] = true
```

**Cost:** Three extra characters in the ~2% of problems that run a sieve or harmonic step loop. **Gain:** Deletes a hard keyword, eliminates the `by` keyword / `by:` parameter collision, and chains cleanly with `.rev` (`(0..n).step(2).rev`).

---

### Cut 5: Delete `[D]E: Index` exponential array indexing from Kernel Extension A

`synthesis §2.1A` proposes deriving `Index` automatically for five compound shapes: `Bool` (`K = 2`), fieldless/payload `enum` (sum `K`), `?T` (`K + 1`), bounded integer ranges `lo..=hi`, tuples `(A, B)` (product `K`), and arrays `[D]E` (exponential `K = E.K ^ D.K`).

The first five are clean, O(1), stack-allocated `Copy` values. The sixth — `[D]E: Index` — hides three severe traps for just **2 out of 710** corpus problems (`noi15p3` base-3 prime mask, `cco19p1` 4×4 board state):

1. **Hidden O(`D.K`) indexing cost.** Every `dp[assign]` lookup has to loop over all `D.K` entries doing radix multiplication/division, and `for assign in Assign:` materializes an array on every state — violating G1 ("no hidden costs").
2. **Heap-vs-stack schizophrenia.** Ordinary `[D]E` arrays are heap-backed, non-`Copy` containers (`Array<D, E>`). Making `[D]E` implement `Index` forces a special compiler rule for compile-time-small arrays and breaks the mental model that every `Index` is a tiny, O(1), `Copy` value.
3. **Silent exponential blowup.** Writing `[V]Bool` where `V = 1..=100` has `K = 2^100`, which explodes `Int` even though `[V]Bool` is a tiny 100-byte array.

Delete `[D]E: Index`. Keep `Bool`, unit and payload `enum` (computed *on demand* when actually used as an `Index`, so enums with large `V` fields never eagerly overflow), `?T`, bounded ranges `lo..=hi`, and tuples `(A, B)`. For bitmasks, use `Subset<D>` (O(1) bit ops, `Copy`). For the 0.3% base-3 or small-board DP, `@memo(hash)` works out of the box or a 4-line `encode` function packs the state explicitly with visible cost.

Before:
```cp
enum Side = Neither | A | B
type Assign = [Small]Side        // hidden O(D.K) radix encode on every dp[assign]!
var ways: [Assign]Z = 0
```

After:
```cp
// Bitmasks use Subset<D> (O(1) bit operations); rare base-3 DP encodes explicitly or uses @memo(hash)
@memo(hash)
fn wins(board: Int, turn: Bool) -> F64:
    ...
```

**Cost:** The 2/710 corpus problems using base-3 or small-grid state encodings either rely on `@memo(hash)` or write a 4-line base-`b` encoder. **Gain:** Every `Index` in the language stays strictly O(1), `Copy`, and overflow-safe.

---

### Cut 6: Delete `Pool<T>`, `Pool<Node<_>>`, `PoolMap`, `Pool.reusable`, and `Handle` (Kernel Extension C)

`synthesis §2.1C` introduces five new concepts for dynamic node arenas (tries, treaps, persistent segment trees): `Pool<T>`, a one-off recursive type placeholder `Pool<Node<_>>()` to tie the self-handle knot, a separate `Handle` trait ("not an `Index`"), a parallel `PoolMap<nodes.Id, V>` container, and `Pool.reusable()` generational debug tracking.

Look closely at what CP Lang *already* has without any of those five concepts:

- `List<Node>` already grows dynamically and checks bounds.
- Bounded range types like `type Id = 0..=2_000_000` already store in **4 bytes** and niche-pack `?Id` in **4 bytes** — exact parity with `Pool`'s 4-byte handle!
- `xs.freeze() -> (domain D, [D]T)` (`plan/19 §4.4`) already turns a finished `List<Node>` into a zero-check static domain `Node` for Aho-Corasick failure links or persistent DAG DP.

Delete `Pool<T>`, `Pool<Node<_>>`, `PoolMap`, `Pool.reusable`, and `Handle`. Add one tiny method on `List<T>` if desired (`xs.push_id(x) -> Int`, returning the index pushed), and keep `xs.freeze()`.

Before:
```cp
struct Node<Id>:
    kid: [Bool]?Id = none

var nodes = Pool<Node<_>>()      // bespoke recursive placeholder syntax
fn reach(root: nodes.Id, x: Int) -> nodes.Id:
    var (o, rest) = (root, x)
    while rest > 1:
        o = nodes[o].kid[rest % 2 == 1] ??= nodes.push(Node())
        rest /= 2
    o
```

After:
```cp
type Id = 0..=2_000_000          // 4-byte handle, 4-byte niche-packed ?Id
struct Node:
    kid: [Bool]?Id = none

var nodes: List<Node> = [Node()]
fn reach(root: Id, x: Int) -> Id:
    var (o, rest) = (root, x)
    while rest > 1:
        o = nodes[o].kid[rest % 2 == 1] ??= nodes.push_id(Node())
        rest /= 2
    o
// After construction, if you want zero-check side arrays:
let (N, frozen) = nodes.freeze()
```

**Cost:** Two simultaneous dynamic arenas in the same function share `Int`/`Id` until `freeze()`, and indexing `nodes[o]` retains a bounds check outside `unchecked:`. **Gain:** Deletes five concepts, including the only type-level knot-tying syntax (`Node<_>`) in the entire language.

---

### Cut 7: Delete `Range1<I>` — keep one `Range<I>` type everywhere

Having both `Range<I>` (possibly empty, `.first -> ?I`) and `Range1<I>` (non-empty, `.first -> I`) doubles the range mental model and causes type mismatches whenever a non-empty view (`runs`, `windows`) is passed to a helper expecting `Range<I>`. Across 299 `.cp` ports, `Range1` appeared in **1 file** (`ioi11p3.cp`), and even `synthesis §2.2` line 307 instinctively wrote `w.first!` on a supposed `Range1`!

Delete `Range1<I>`. Every range (`l..=r`, `l..r`, `..i`, `i..`, `runs(a)`, `D.windows(k)`, `tree.sub(v)`) has one type: `Range<I>`, with `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len -> Int`, `.is_empty -> Bool`. Use `r.first!` when you know the range is non-empty, and use dependent bounds `(l: V, r: l..=V.last)` when reading ordered pairs.

**Cost:** One `!` character (`r.first!`) on ranges known to be non-empty. **Gain:** Eliminates a duplicate range type, its conversions, and a whole class of `Range1<I>` vs `Range<I>` signature mismatches.

---

### Cut 8: Delete `I32`, `I64`, `U8`, and `U32` — keep 4 numeric types

`plan/11` defines eight numeric types (`Int`, `I32`, `I64`, `U8`, `U32`, `U64`, `I128`, `F64`) plus range types (`0..=1e9`). Range types already pack `0..=1e9` into 4 bytes and `0..256` into 1 byte automatically without wrapping bugs, `Char` is already 1 ASCII byte, and `Int` already covers `-(2^63 - 1)..=(2^63 - 1)` with an 8-byte `?Int` niche. Nobody in a contest needs signed `-2^63` specifically (`I64`) or manual `U8`/`I32`/`U32` width juggling.

Keep exactly **four** numeric types: `Int` (plus range types when you want tight memory layout), `U64` (wrapping 64-bit unsigned for bitmasks, hashes, XOR basis), `I128` (128-bit signed for `a * b % m` and big cross products), and `F64` (64-bit IEEE float with quiet-NaN niche).

**Cost:** None in pure CP Lang; low-level C++ FFI bindings (if ever needed) can keep fixed-width aliases in an internal FFI module. **Gain:** Cuts the numeric type table in half and eliminates "should I use `I32` or `0..=1e9`?" hesitation.

---

### Cut 9: Delete dead syntax — implicit single-letter generics, `do:`, `{ ... ; ... }` blocks, `;`, `<=>`, `.field` shorthand, `dyn`, `decorator`, `static let`

Corpus evidence across 299 `.cp` ports and 19 `stdlib/*.cp` files shows a cluster of features with 0–2 uses that add parser complexity or subtle traps:

- **Implicit single-letter generics** (`plan/17` line 61: omitting `<D>` on `fn peel(s: [D]Bool)`): active footgun! A typo like `fn dfs(u: W)` when your domain is `V` silently defines a generic function over `<W>` instead of reporting "unknown type `W`". Always write `<D>` explicitly.
- **`;` and inline `{ let k = ...; ... }` brace blocks** (`plan/17` lines 14–17, 45): **0 uses** in 299 ports + 19 stdlib files. Deleting `{ ... }` removes confusion with map literals (`[k: v]`) and string interpolation (`"{x}"`); multi-step lambdas simply use a named 2-line helper `fn` or indented block.
- **`<=>` spaceship operator** (`token.rs` line 83): **0 uses** anywhere. `Ord` only needs `<`.
- **`do:` blocks** (`decisions 4.5`): **2 uses** in 299 ports (both inside `[D] x => do:`). Allowing an indented block directly after `=>` (or writing a 3-line `for` loop) eliminates `do:` cleanly.
- **`.field` lambda shorthand** (`by: .w`): **1 use** in 299 ports. Fails as soon as you want descending (`-.w`? `by: e => -e.w`) or a pair key (`e => (e.w, e.u)`). Keep ONE lambda form: `e => e.w`.
- **`dyn` function types** (`decisions 3.3`), **user `decorator` macros** (`decisions 3.11`), and **`yields` return keyword** (`plan/12`, redundant when `yield` inside the body marks a generator returning `Iterator<T>`): **0 uses** anywhere; unnecessary compiler surface for zero contest utility.
- **`static let`** (`decisions 6.3`): **0 uses** in 299 ports + 19 stdlib files. `const` already handles compile-time tables, and when a multitest problem needs a runtime sieve computed once before `t` test cases, writing an explicit `for _ in 0..read():` loop makes the lifecycle obvious with zero magic.

Before:
```cp
fn peel(s: [W]Bool): ...            // silent generic instead of typo error on `V`!
sort(mut es, by: .w)                // breaks the moment you need descending or tie-break
let at: [C]Pt = [C] c => do:
    let p = locate(c)
    p
```

After:
```cp
fn peel<D>(s: [D]Bool): ...         // explicit generic; typos are instant errors
sort(mut es, by: e => e.w)          // extends seamlessly to `e => (-e.w, e.u)`
var at: [C]Pt = Pt(0, 0)
for c in C: at[c] = locate(c)
```

**Cost:** Explicit `<D>` adds 3 characters on generic helpers (rare in contest code); multi-test runtime precomputation uses an explicit `for _ in 0..read():` loop instead of `@multitest` + `static let`. **Gain:** Deletes 7 syntax rules/keywords and eliminates silent typo-to-generic bugs.

---

### Cut 10: Kill every duplicate spelling and syntax contradiction (one canonical spelling each)

Blind Critic #1 failed first-compile on 8 out of 8 unseen problems largely because `plan/` docs, `decisions.md`, `stdlib/`, and `proto/cpc` drifted into multiple spellings for the same construct. Lock **one** spelling and delete every alias:

- **Boundaries:** keep `v.l`, `v.r`, `v.l(k)`, `v.r(k)` — **delete** `v.before` and `v.after` (2 hits in 1 port).
- **Index stepping:** keep `v + k -> ?V`, `v - k -> ?V`, and `v.cyc(k) -> V` — **delete** `v.offset(k)` (4 hits in 1 port) and **delete** `v.next`/`v.prev` (27 hits in 15 ports, all trivially `v + 1`/`v - 1`). This cuts three directional vocabularies (`l/r`, `left/right`, `next/prev`) down to two: `+ k`/`- k` for stepping positions, `.l`/`.r` for position → boundary, and `.left`/`.right` for boundary → position.
- **Modular arithmetic:** confirm `synthesis §2.1E` — keep **one** `Mod<P>` for both `const` and runtime-frozen `let` moduli; **delete** `ModR` and `ModInt`.
- **Bitsets:** keep `Bitset<D>` — **delete** `BitSet<D>` and `Bits<D>`.
- **Inline conditional:** keep `if cond: a else: b` — **delete** `if cond then a else b` (`plan/11`) and `a if cond else b` (`plan/16`).
- **Branching:** keep `elif` — never `else if`.
- **`match` arms:** keep bare `Pattern => expr` (or indented block after `=>`), destructuring enum payloads positionally (`Add(l, r) => ...`) — **delete** `case` and `:` arm syntax.
- **XOR vs power:** keep `^` for XOR and `**` for integer/float power everywhere — **delete** `2^20` notation (`plan/11` line 104) and `xor` keyword.
- **Indexed iteration:** keep `.items` on arrays, slices, lists, strings, and maps — **delete** `enumerate`.
- **Adjacent pairs:** keep `xs.adjacent` (consecutive pairs, O(n); `xs.cyc.adjacent` wraps) — **delete** `pairs(V)`, which `plan/19` defined as O(n²) all pairs `u < v` while `plan/20` defined it as O(n) adjacent pairs! Writing `for i in V: for j in i.r..:` for `u < v` is one line and impossible to misread at 2 AM.

**Cost:** Updating 15 prototype ports that wrote `.next`/`.prev` and aligning `plan/` docs. **Gain:** Zero "wait, is it `then` or `:`? `BitSet` or `Bitset`? `ModR` or `Mod`?" stalls under contest pressure.

---

### Cut 11: Trim 18 `@` decorators and the interactive `protocol` DSL down to 7 contest decorators

`plan/15`, `plan/16`, `plan/17`, and `synthesis` accumulate 18 `@` decorators plus four interactive-only keywords (`protocol`, `limit`, `@wire`, `@final`). Cut the redundant ones ruthlessly:

- **Delete `protocol`, `limit`, `@wire`, `@final`** (`plan/16`): interactive problems are rare and short. Replace the whole 5-keyword protocol DSL with plain I/O (`ask(...)` to print + flush + read response, or `print` + `read`) and a single `@interactor fn judge(...)` block that responds and enforces query limits with ordinary `assert`.
- **Delete `@derive(...)`** (`plan/15`): `Eq`, `Ord`, `Hash`, `Show`, `Read`, `Gen`, `Copy`, and `Index` derive automatically whenever a struct/enum's fields support them. Never force a contestant to write `@derive(Read, Gen)`.
- **Delete `@solution(f, when: ...)`** (`plan/15`): subtask branching is literally `if n <= 5000: solve_small() else: solve_fast()`.
- **Delete `@local`** (`plan/15`): `debug(...)` is already stripped on the judge, and unused helpers are tree-shaken.
- **Delete `@sample(...)`** (`plan/15`): `cpl new` already puts sample files beside the `.cp` file; inline escaped sample strings bloat solutions.
- **Delete `@invariant(...)`** (`plan/15 §4.3`, symmetry property test, not loop invariant): write a 3-line `@test` or check inside `@brute`.
- **Delete `@soa`** (`plan/11` open): use parallel arrays `[V]A`, `[V]B` when you want SoA layout.
- **Unify `@graded` / `@grader`** to ONE spelling: `@grader`.

That leaves **7 contestant decorators**: `@memo` (with optional `@memo(hash)`), `@multitest`, `@brute`, `@tags`, `@gen`, `@checker` / `@interactor`, and `@grader` — plus 2 library-author decorators (`@get`, `@test`).

Before:
```cp
protocol Guess:
    limit 20
    @wire("?") fn ask(x: Int) -> Char
    @final @wire("!") fn answer(x: Int)

@interactor(Guess, adaptive)
fn judge(n: Int): ...
```

After:
```cp
// Solution uses ordinary flush-and-read `ask` + `print`
let resp: Char = ask("? {mid}")
print("! {ans}")

// Local judge is a plain @interactor function with ordinary `assert`
@interactor
fn judge(secret: 1..=n):
    var q = 0
    ...
    assert q <= 20
```

**Cost:** Query limits in interactive problems are checked by `assert` inside `@interactor` rather than declared in a separate `protocol` block. **Gain:** Deletes four keywords/decorators and cuts total `@` surface by more than half.

---

### Cut 12: Confirm all 18 rejections in `research/archive/architecture/language-architecture-synthesis.md §5`

Audit verdict on the 18 one-off features already rejected by `synthesis §5` (reactive segment trees, triangular-index inversion, `ModR`, `pool` keyword, `domain Node = A(X) | B(Y)`, `read(increasing)` / `Symmetric` / `Span` / fixed-first read flags, `first_where` / `last_where`, auto-vivifying `m[k].push(x)`, implicit `?T` ordering, `a[i: v]` colon updates, `kth_shortest(sources_at:)`, `Left`/`Right` merge iterator, bare `for i, x in xs`, `next_layer(fill, body)`, debug shadow-bit uninit arrays, hardcoded `Order` enum, 14 prelude micro-helpers, built-in rectangle halving): **all 18 rejections stand.** Do not revive any of them.

---

## Part II: Trait and Standard-Library Simplifications

### Cut 13: Slash 26 proposed traits down to 8 contestant traits + 3 library-author traits

`plan/11`, `plan/13`, `decisions 3.5`, and `synthesis` list roughly 26 traits. Audit every one of them:

- **Delete 6 of the 9 representation traits (`decisions 3.5`):**
  - `Small` — redundant with `Copy`; never used in any bound.
  - `Relocate` — CP Lang has no stored borrows, no self-referential structs, and no user destructors, so *every* type is trivially relocatable by construction. A universal invariant is not a trait.
  - `Zero` — pure backend optimization (`calloc`/`memset` when filling with `0`/`false`); never needed in user signatures.
  - `Bits` — `Bitset<D>` and `Subset<D>` take `D: Index`, not `Bits`; `stdlib/algebra.cp` only used `Bits` for `Xor<T>`, which Cut 15 deletes.
  - `OrdKey` — speculative radix-sort trait; 0 uses in 299 ports or `stdlib/`.
  - `Bounded` — only existed for `Max<T>`/`Min<T>` sentinels, which `?T` + `min`/`max` skipping `none` replaced.
- **Internalize `Niche`:** keep automatic niche packing for `?Int`, `?F64`, `?V`, `?Enum`, `?Range`, `?Bool`, `?Char` in the compiler layout engine, but drop `Niche` as a user-visible trait.
- **Delete `Shrink` trait:** `synthesis §4 Tier 3` introduces choice-stream shrinking, which shrinks *any* `Gen` automatically by shrinking its `rand(...)` choices! Once choice-stream shrinking exists, no type needs a separate `Shrink` trait.
- **Delete `Modulus`, `Handle`, `FenwickIndex`, `Window`, and `BagWindow` traits:**
  - `Mod<P>` accepts any `const` or frozen `let` `Int` directly (no `Modulus` trait).
  - `Handle` dies with `Pool` (Cut 6).
  - `FenwickIndex` dies in favor of two plain concrete structs `Fenwick<I, T>` and `Fenwick2D<R, C, T>`.
  - `Window` and `BagWindow` die in Cut 14.

**Surviving traits:** 8 core traits (`Index`, `Seq`, `Copy`, `Eq`, `Ord`, `Hash`, `Show`, `Read` — all auto-derived when fields support them) plus 3 library-author traits (`Indexed`/`IndexedMut`, `Iterator`, `Gen`).

**Cost:** Custom non-standard niche layouts or custom radix-sort keys cannot be written in userland without a compiler intrinsic (needed in 0 of 710 corpus problems). **Gain:** Cuts the trait surface by nearly 60%.

---

### Cut 14: Delete the `Window<I>` / `BagWindow<I>` trait hierarchy (`synthesis §2.2`), `TwoStackFold`, `GroupWindow`, and `slide_until`

`synthesis §2.2` proposes two traits (`Window<I>`, `BagWindow<I>`), three wrapper structs (`MonoDeque`, `TwoStackFold`, `GroupWindow`), four driver methods (`slide`, `slide_while`, `slide_until`, `g.sliding`, plus `mo`), and two calling conventions (struct object vs inline callbacks). Even inside `synthesis §2.2` itself, the kwarg drifted from `ok:` on line 273 to `until:` on line 307 — a textbook 2 AM recall trap.

Look at what the corpus actually needs:

- Fixed-width sliding windows: `D.windows(k)` already exists; add `sliding_min(a, k)` / `sliding_max(a, k)` (plus `MonoDeque<T>` for monotone-queue DP).
- Variable-width two pointers: **one** callback helper `sliding(D, add:, drop:, ok:)` (already in `plan/10-language-tour.md`!) yielding the longest valid window ending at each `r` — deleting `slide_until` and `slide`.
- Mo's algorithm (1/151 scout ports): a plain function `mo(qs, add:, drop:)` in `std.algo` (or local lib) taking two inline callbacks — zero `BagWindow` trait needed!

Keep the tiny `5.4` borrow-checker refinement (two sequential `inline` callbacks in one helper do not alias each other), because it makes `sliding` and `mo` work directly on plain local `var`s with zero wrapper structs.

Before:
```cp
// Must choose between Window vs BagWindow, slide vs slide_while vs slide_until, struct vs callbacks
for w in Field.slide_until(until: w => cost(w) <= budget):
    best max= w.len
```

After:
```cp
// One variable-width helper `sliding` (plus `D.windows(k)` and `sliding_min`/`sliding_max`)
var total = 0
for w in sliding(V, add: i => total += a[i], drop: i => total -= a[i], ok: () => total <= k):
    best max= w.len
```

**Cost:** Problems needing two-stack non-invertible window folds (`TwoStackFold`, < 0.5% of contests) live in a user local library instead of core `std`, and "shortest window >= K" uses a 4-line `while` loop (or negating `ok`). **Gain:** Deletes 2 traits, 2 wrapper structs, and 3 overlapping window driver names.

---

### Cut 15: Collapse 20 algebraic wrappers in `stdlib/algebra.cp` into bare operators, `(id, op)`, `desc(x)`, and `Action`

`stdlib/algebra.cp` currently exports 20 names: `Semigroup`, `Monoid`, `Group`, `Action`, `Order`, `Semiring`, `Sum`, `Prod`, `Max`, `Min`, `Xor`, `MaxOpt`, `MinOpt`, `MaxPlus`, `MinPlus`, `Less`, `Greater`, `flip`, `both`, `opt`. Look at line 96 of `stdlib/algebra.cp`:

```cp
sort(mut xs, by: Order<(Int, Int)>.by(p => p.1))
```

And look at Blind Critic #1 (Finding 8): the critic had to hand-write a 6-line `opt_max` combiner for `SegTree<Pos, ?Int>` even though `decisions §13` *already* specifies that `min`/`max` skip `none` (`min(none, x) == x`) and bare operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) work directly as arguments!

Collapse all 20 names to four simple rules:

1. **Bare operators work directly** in `SegTree`, `Fenwick`, `SparseTable`, and `scan`: `SegTree<V, Int>(+, a)`, `SegTree<V, ?Int>(max, a)`, `SparseTable<V, Int>(gcd, a)`, `scan(a, +)`.
2. **Custom monoids pass `(id, op)` directly**: `SegTree<V, Node>(Node.id, combine, a)` (or `Monoid(id, op)` for library authors who want `laws_hold`). Delete `Semigroup`, `Group`, `Semiring`, `Sum`, `Prod`, `Max`, `Min`, `Xor`, `MaxOpt`, `MinOpt`, `MaxPlus`, `MinPlus`, `flip`, `both`, `opt` from the contestant surface. `Fenwick<V, T>` defaults to `+`/`-` (optional `(id, add, sub)` for XOR), `SparseTable` takes any `(T, T) -> T`, and `Matrix.mul_with(other, add:, mul:, zero:)` handles tropical `(max, +)` without a `Semiring` type.
3. **Sorting and heaps use `by:` and `desc(x)`** — delete `Order<T>`, `Less`, and `Greater`. Merge `Heap<T>(Order)` and `Heap.by(...)` into just **`MinHeap<T>()`** and **`MaxHeap<T>()`** (using `desc(x)` inside tuples for mixed tie-breaks like `(dist, desc(id))`).
4. **Keep `Action`** ONLY for `LazySegTree`, with six stock tags (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) plus custom `Action(id, compose, apply)`.

Before:
```cp
sort(mut xs, by: Order<(Int, Int)>.by(p => p.1))
var pq = Heap<(Int, V)>(by: Less<(Int, V)>)
var st = SegTree<V, ?Int>.from(opt(Semigroup<Int>((a, b) => max(a, b))), init)
```

After:
```cp
sort(mut xs, by: p => p.1)
var pq = MinHeap<(Int, V)>()
var st = SegTree<V, ?Int>(max, init)
```

**Cost:** `laws_hold` property testing of user-defined `Group`/`Semiring` structs moves out of the contestant API (library authors still have `Monoid` and `Action` in `std.algebra`). **Gain:** Deletes 15–18 wrapper names from contest memory and gives one idiomatic call style across sorting, heaps, prefix scans, and range trees.

---

### Cut 16: Slash 19 search and scan names down to 7

`synthesis §2.3A` and `stdlib/seq.cp` propose 19 names (`r.mid`, `partition_point`, `bsearch_first`, `bsearch_last`, `bsearch_first_some`, `bsearch_last_some`, `bisect`, `first_ge`, `last_le`, `argmin_convex`, `local_max`, `local_min`, `first`, `a.find`, `a.where`, `min_by`, `max_by`, `argmin`, `argmax`, plus `running_sum`).

Cut to **7 canonical names** (plus `r.mid`):

- **`bsearch_first(r, ok) -> ?I`** and **`bsearch_last(r, ok) -> ?I`**: monotone binary search (checked monotone in debug). Works on `Range<I>`, domain `D`, integer ranges `0..=hi`, AND float ranges `(lo..=hi).bsearch_first(ok, steps: 80)` — deleting `bisect`, `partition_point`, `bsearch_first_some`, and `bsearch_last_some` (if you want the witness from the predicate, call `check(best)` once after O(log n) iterations instead of memorizing two 18-character method variants!).
- **`xs.first_ge(x) -> ?I`** and **`xs.last_le(x) -> ?I`**: sorted array/list lookup — identical names to ordered `Set`/`Map`!
- **`first(it) -> ?T`**: first item of any generator (`first(v for v in V if ok(v))`) — deletes `a.where(pred)`.
- **`a.find(x) -> ?I`**: first position equal to `x`.
- **`min_by(it, key)` / `max_by(it, key)`**: covers `argmin`/`argmax` via `min_by(a.items, p => p.1)` (keep `argmin`/`argmax` out of the core cheat sheet; move `argmin_convex`/`local_min`/`local_max` to `std.search` or local lib).
- Delete **`running_sum(xs)`** from `stdlib/seq.cp` — it is literally a 1-line alias for `scan(xs, +)`, and `prefix(a)` already provides O(1) range sums.

**Cost:** One extra `check(best)` call when binary searching for a witness object (1 extra call out of ~60 probes). **Gain:** Cuts the search vocabulary by more than 60% and eliminates every near-synonym (`bisect` vs `bsearch_first`, `where` vs `first`, `running_sum` vs `scan`).

---

### Cut 17: Unify collection verbs across every container

Blind Critic #1 wasted compiles guessing `.erase(...)`, `.len()`, and `.first()`, while `stdlib/README.md` mixes `.insert`/`.has`/`.union` (`Bitset`), `.add`/`in`/`|` (`Subset`), `.count()` (`Bitset`), `.len()` (`Heap`), and `.len` (`List`). Lock **one** verb table across every container:

- **Size & emptiness (properties, never parens):** `.len`, `.is_empty`, `.clear()` on every container, string, range, slice, heap, set, map, and bitset.
- **Sequential ends (`List`, `Deque`, `MinHeap`, `MaxHeap`):** `.push(x)`, `.pop() -> ?T` (plus `Deque`: `.push_front(x)`, `.pop_front() -> ?T`; heaps: `.peek -> ?T`).
- **Ordered ends (`List`, `Deque`, `Range`, `Set`, `Map`, `SortedBag`):** `.first -> ?T`, `.last -> ?T` (and `.pop_first()`, `.pop_last()` on ordered sets/maps). Delete `Set.min()` / `Set.max()`.
- **Ordered predecessor/successor (`Set`, `Map`, `SortedBag`, sorted `List`/`Array`):** `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`, plus `.rank(k)` / `.nth(i)` on ordered sets/bags.
- **Sets (`Set`, `HashSet`, `SortedBag`, `Subset`, `Bitset`):** `.add(x)`, `.remove(x)`, `x in s`, `|`, `&`, `-`, `^` (delete `.insert`, `.erase`, `.has`, `.union`, `.count()`).
- **Maps (`Map`, `HashMap`):** `m[k]` (or `m[k] ??= init`), `m.get(k) -> ?V`, `m.remove(k)`, `k in m`, `for (k, v) in m.items`.

**Cost:** Small method renames in `stdlib/bitset.cp` and `stdlib/heap.cp`. **Gain:** Guessing a method name at 2 AM succeeds on the first try across every collection type.

---

### Cut 18: Split core `std` from `examples/local-lib/` — move ~20 niche structures out of `std`

`synthesis §2.3E` and Phase 4 catalog over 45 stdlib structures. Four of them can be deleted outright because existing abstractions already cover them:

- **Delete `RangeSet<K>`:** merge into `Steps<K, Bool>` (already in `stdlib/steps.cp`).
- **Delete `RangeApply<D, F>`:** `LazySegTree` (or a `Fenwick` difference array for groups) already handles range-update + point-query.
- **Delete `LazyHld` (and make `Hld` optional):** `Rooted<V>` already provides `tree.sub(v)` (one `Range<Pre>`) and `tree.chains(a, b)` (O(log n) `Range<Pre>`s). Composing `tree.chains(a, b)` with *any* `SegTree`, `LazySegTree`, `Fenwick`, `Steps`, or `Bitset` takes two lines and eliminates separate `Hld` / `LazyHld` wrapper hierarchies!
- **Delete `Distinct<[I]T>` and `Sorted<[I]T>` input wrappers (`synthesis §4 Tier 1`):** reading into a wrapper forces unwrapping on every call to functions expecting `[I]T`. Keep `Perm<V>` and `Tree<V>` (which have real domain methods like `.inv()`, `.cycles()`, `.root(...)`), and express distinct/sorted input preconditions with `assume a.distinct` / `assume a.is_sorted` (plus `gen:` when needed).

Move the remaining specialized, 1–2-problem structures out of the core `std` mental surface into `examples/local-lib/` (where they compile at identical zero-cost speed): `Rope`, `RangeKth`, `LiChao`/`LiChaoArena`, `PotentialDsu`, `Functional`, `Congruence`, `Fixed`, `Scanner`, `Tri`, `Powers`, `dnc_opt`, `knuth`, `Hash2D`, `Trie`, `fold_balanced`, `fold_others`, `overlaps`, `borders`, `Counter` (just `HashMap<T, Int>` + `(cnt[x] ??= 0) += 1`!), and `Frac`.

Before:
```cp
var hld = LazyHld<V, Int>(tree, AddSum)
hld.apply_path(a, b, delta)
```

After:
```cp
var st = LazySegTree<tree.Pre, Int>(AddSum, init_in_pre_order)
for seg in tree.chains(a, b): st.apply(seg, delta)
let total = sum(st.fold(seg) for seg in tree.chains(a, b))
```

**Cost:** Advanced ICPC/IOI topics (implicit treaps, wavelet trees, Li Chao trees, Knuth/D&C DP, potential DSU) are imported from a personal `local-lib` rather than `std`. **Gain:** Core `std` fits cleanly in one mental page while `examples/local-lib/` demonstrates G8 in action.

---

## Part III: Concepts That Earned Their Keep

Adversarial scrutiny also revealed which abstractions genuinely pay for their conceptual weight in contest bugs prevented, lines saved, and zero-overhead speed. **Do not delete or weaken these:**

1. **Nominal `domain V = 1..=n` + `Index`/`Seq` + `[V]T` + `[Row, Col]T` + ASCII-only `Str` (`[domain]Char`).** One general indexing mechanism eliminates vertex/edge swaps, row/col swaps, 0-vs-1 indexing bugs, and bounds checks in hot loops, while keeping strings flat O(1) byte arrays with zero UTF-8 tax.
2. **Boundary indices `.l`/`.r` (`.l(k)`/`.r(k)`), open ranges `..i`/`i..`, and `prefix(a)`.** Praised directly by Blind Critic #1; turns the single most error-prone CP pattern (off-by-one prefix sums and half-open intervals) into type-checked code.
3. **`?T` with automatic niche packing, `??`, `??=`, `?`, `if let`, `min=`/`max=`, and `min`/`max` skipping `none`.** Eliminates `INF`/`-1` sentinel overflows across DP, BFS, Dijkstra, and segment trees (`SegTree<V, ?Int>(max, a)`), while `??=` solves both map-bucket initialization and trie allocation cleanly without auto-vivification.
4. **Lexical `lends` / `lends mut`, `law disjoint`, explicit `.copy()`, and last-use moves (with value-copying destructuring).** Invisible to contestants writing normal solutions, prevents the exact UAF bug Blind Critic #1 uncovered (`let (r, c) = q[head]` copies values), eliminates lifetime annotations, and unlocks safe `swap(mut g.row(r1), mut g.row(r2))` and write-back `a[l..=r].sort()`.
5. **Unified `Mod<P>` for compile-time `const` AND runtime-frozen `let` moduli (`synthesis §2.1E`).** One type for both `Mod<1_000_000_007>` and `Mod<m>`, with Barrett/Montgomery reduction under the hood and zero extra concepts.
6. **Algebraic `Index` for `Bool`, unit/payload `enum` (on demand), `?T`, bounded ranges `lo..=hi`, and tuples `(A, B)`, paired with flat/salted-hash `@memo`.** Unlocks typed flow/2-SAT graphs (`FlowGraph<Node>`), multi-dimensional DP (`[V, Bool]Int`, `[0..=w]Int`), and eliminates the 31.86× hash-memo regression found in Stage-6 max-N benchmarking.
7. **Exhaustive `match` on payload `enum`s.** Praised by Blind Critic #1; catches missing transition/query cases at compile time.
8. **Euclidean `/` and `%`.** `a % b` is always in `0..|b|`, eliminating negative-modulo bugs while lowering directly to native C++ `/` and `%` whenever the range checker proves non-negativity.
9. **Typed `read()` + `assume` + `gen:` + choice-stream shrinking + `@brute`.** Makes differential stress-testing a 5-line habit instead of a 50-line harness, with zero manual shrinkers.
10. **Explicit escape hatches `a.raw[i]` and `unchecked:`.** Guarantees zero compiler fights when a contestant has a subtle non-local invariant or needs raw C++ inner-loop speed (G14, G17).
11. **`Rooted<V>` with `tree.sub(v)` and `tree.chains(a, b)`.** A tiny, composable tree view that collapses Euler tour, subtree queries, LCA, and HLD into ordinary `Range` loops over any range data structure.
12. **Salted hashing & compile-time complexity warnings.** Prevents anti-hash hacks on Codeforces (Stage-6 finding) and catches accidental quadratic/exponential mistakes before submission (praised by Blind Critic #1).

---

## Part IV: Spec Conflicts Resolved for `docs/one-page-cheat-sheet.md`

To make `docs/one-page-cheat-sheet.md` a single unambiguous reference for future Blind Critics, these eight contradictions across `plan/`, `decisions.md`, `stdlib/`, and `proto/cpc` are locked to one rule:

1. **Inline `if`:** `if cond: a else: b` only (not `if c then a else b`, not `a if c else b`).
2. **`match` arms:** `Pattern => expr` with bare variant names (not `case`, not `:`).
3. **Bitwise XOR vs power:** `^` is XOR, `**` is power (never `^` for power, never `xor`).
4. **Destructuring:** `let (a, b) = xs[i]` copies (or moves on last use) so mutating `xs` afterward is always memory-safe (fixes Blind Critic #1 UAF).
5. **Auto-derived `Ord` on `struct`:** synthesized only when every field implements `Ord`, never unconditionally on structs containing `List`/`Array` (fixes Blind Critic #1 invalid C++ codegen).
6. **`assert` vs `assume`:** `assert` stays checked in judge builds; `assume` is local/debug/stress only and stripped in judge builds.
7. **Nested `fn` mutation:** plain `fn` mutates outer `var`s automatically (no `mut fn`).
8. **Modular arithmetic:** `Mod<P>` handles both `const` and runtime `let` moduli (no `ModR`).
