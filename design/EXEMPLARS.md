# Taste exemplars, round 6 (frozen after the round-6 fixes)

Six problems, written the way the ideal language would want them, ignoring what the current spec allows.
Each round is grilled by a hostile reviewer: `wf/grill-r1.md` (74 items), `wf/grill-r2.md` (50),
`wf/grill-r3.md` (24), `wf/grill-r4.md` (29), `wf/grill-r5.md` (25), `wf/grill-r6.md` (17, verdict: converged). Earlier
rounds are kept as `wf/EXEMPLARS-r2.md` to `wf/EXEMPLARS-r6.md`. Round 4 gave `@` one meaning after Ryan pointed out it was doing three jobs. Round 5
fixes the round-4 report, chiefly by making `with` lexical with an explicit hand-off (`uses`), instead of
half lexical and half dynamic. Round 6 deletes runtime (`dyn`) contexts: a runtime modulus is a
block-scoped type, like a domain, so it can't escape or go stale.

Items marked **[proposed]** are my calls; **[you]** are Ryan's. Checked by script: the `Cover`/`Tick` laws
(`wf/law3.py`, 100k cases), the problem-4 sweep against brute force (`wf/sweep.py`), and `Tick` on a real
lazy segment tree with non-power-of-two sizes and arbitrary range splits (`wf/split.py`, 3,000 random
runs). The version without range restriction fails (`wf/split_naive.py`). Skipping the fold for uniform
tags gives the same answers (`wf/split_uniform.py`).

---

## The rules

Round-3 item numbers are in brackets.

1. **Partiality.** Brackets (`a[i]`) and `x!` trap. Accessors that can come up empty (`.first`, `.last`,
   `.max()`, `j.left`) return `?T`, unless the length is **syntactically** at least 1: an array over `[k]` with
   `k: 1..=n`, or a domain `V = n` with `n: 1..` (21.6) [2impl Q46]. No prover changes a type. `.get(i)` is the fallible index.
   Stdlib constructors can return statically non-empty types: `c.window(k, incl: true)`,
   `Bits<D>.nonempty`, `s.subsets()`, `s.windows(m)` with `m: 1..=s.len`, and length-prefixed reads
   `[len: 1..=n]T`. Folds over them are total [r8 3]. `map` and `zip` keep non-emptiness; `where` and
   `filter` drop it [r9 12].

2. **Three kinds of index space**, each with its own spelling [21].

   | kind | spelled | identity | `base` converts | compares with `Int` | arithmetic |
   |---|---|---|---|---|---|
   | counted | `domain V = n` | nominal, fresh | yes | no (`v.ord`) | none |
   | labeled | `domain G = labels(1..=3)` or `labels("a", "b")` | nominal, fresh | no; keeps `.label` | no (`g.label`) | none |
   | integer range | a type like `1..=n` | structural (`Int`) | no | yes | ordinary `Int` |

   **Reading a size declares the domain** [r8 1]: `domain V = read(1..=1e5)`, or inside a tuple
   `let (domain A, k): (1..=1e6, 1..=A.count) = read()`, the same binding form as `domain Piece` below.
   `V.count` is the size. Sizes never convert under `io(base:)`; only positions do [r9 16]. When the size
   is used often, `let n = V.count` is the idiom [r9 19]. **An anonymous length `[n]T` is indexed by the integer range `0..n`** [r8 2];
   only named domains are nominal.

   `domain X = …` always makes a fresh type; `type` makes an alias (`type Cut = A.Cut`). Cuts are counts of
   elements before them, so they never convert [22]. A function returning a fresh index space is bound
   with the keyword: `let (domain Piece, pieces) = t.components(without: c)`. Domains are block-scoped and
   declared before use **[you]**. A domain is a type whose size is a runtime value: a generic parameter
   `<Piece>` receives that size as a hidden argument from the value that introduced it (`pieces`), so
   `for x in Piece` and `[Piece]Bool` work inside generic code. The first parameter mentioning the domain
   supplies the size; others are checked equal in debug; a domain no parameter mentions is passed explicitly,
   `f<Col, Row>(r)` [p1 S3]. `cp explain` shows the hidden argument [r4 18, r5 12].
   - **Runtime-parameterized types are scoped the same way, with the same lowering.** `type Z = Mint(mod:
     p)` with `p` read from input makes a fresh, block-scoped type, like `domain V = n`. Code that takes a
     `Z` receives `p` as a hidden argument per activation, exactly as a domain's size travels, so recursion
     and reentry are safe; the compiler may use a static instead when no reentry is possible, and
     `cp explain` says which [r6 6, 7]. Values of `Z` can't leave the block or cross loop iterations. A
     generic with statics or `@memo` can't be instantiated with a scoped type [r6 8, 9]. With a constant
     (`type Z = Mint(mod: 998244353)`) it is specialized; a runtime modulus uses Barrett reduction, and
     `cp explain` shows it at the declaration [r6 10]. Constant to `p` is one local edit.

3. **`with` sets context; `@` says what a declaration is. [proposed, answering Ryan]** [r4 11, 12, 21–27]
   - **`with lib(arg: v)`** sets a context value. A library declares the context and its defaults:
     `context io: base: Int = 0, answers: (Str, Str) = profile.answers`. A bare `with` line covers the rest
     of the enclosing block; `with …:` takes a block. The innermost `with` wins, and an argument passed at
     a call (`read(base: 0)`) beats every context.
   - **Contexts are lexical.** A function body, a lambda and a tool root (an `@interactor`, `@gen`,
     `@check` or `@alt` run by a tool) all see the context where they are written, plus their own `with`s.
     Nothing follows the call stack implicitly, so no function changes behaviour because of its caller.
   - **`uses` is the one hand-off.** A function that should follow its caller receives the caller's
     context as an implicit argument and passes it on. Public library API declares it: on a function
     (`fn read<T>() -> T uses io`), a trait method (binding every impl, so a user `impl Read` can't
     forget it), or a module (`module seg uses checks`). Only nested functions and lambdas infer it (one
     fixpoint per recursive group, never across a module boundary); top-level functions declare it or stay
     lexical. The editor shows both [r5 14, 15, 18; r6 11–13]. A function value or lambda binds the context where it
     is written or referenced [r5 17]. That is how
     `with io(base: 1)` reaches the parsing inside `read<[k]Edge<V>>` and how `with checks(bounds: off)`
     reaches the checks inside `SegTree.apply`. The editor marks calls that receive a context.
   - **Contexts are compile-time constants.** They specialize only `uses` functions, one copy per distinct
     value times each generic instantiation; `cp explain` lists every copy with its bytes, and the 48 KB
     warning counts them [r5 16]. Copies are cloned functions with mangled names, not C++ templates,
     because C++17 template parameters can't hold floats or strings [r6 14]. Runtime values are never contexts: they are parameters or scoped types
     (rule 2).
   - **Tool roots live at program level.** `@interactor`, `@gen` and `@check` may capture only contexts,
     bindings above the wrap line and statically sized types, never per-test state or a runtime-sized domain [r6 4]; `@alt` captures, because it is substituted at its
     call site [r5 9].
   - **`@marker`** attaches a library-declared role to the next declaration: `@alt`, `@check`, `@gen`,
     `@grader`, `@interactor`, `@memo`. `--- with multitest() ---` makes the rest of the file
     the per-test body (16.3). The declaring
     library defines what the role does: a tool consumes it (stress reads `@alt`) or it wraps the
     declaration (`@memo` adds a cache that lives as long as the block declaring the function, so per test
     for a top-level function). A marker never affects code it isn't attached to.
   - Checks and lints are contexts like any other: `with checks(bounds: off)`, `with lint(float_eq: allow)`.
     Checks turned off this way stay on in debug, so only release specializes them [r4 27]. `checks` has
     separate `bounds` and `ranges` fields. User libraries declare their own: `with mylib(eps: 1e-9)`. Keywords can't be variable names (lint) [r4 2].

4. **What `base` converts.** `read`, `print` and interpolation of counted positions and ranges of them.
   Nothing else. A `Range` reads and prints as `first last`, inclusive, and is stored half-open. Grader
   arguments never convert. An interactive error reply is compared on the raw token, before conversion.

5. **Bools print through `yes(b)` [you].** Bare `print(flag)` is a compile error naming the fix. `yes(b)`
   uses the `io` context's `answers`, which defaults to the judge profile's spelling.

6. **Functions and lambdas.** `params => e` is the only lambda; its parameters are patterns, so `(u, v) => e`
   takes two arguments or one pair, whichever is expected. Arrays and bound methods are functions.
   Functions hoist within their block; every capture must be initialized before the first call (checked).

7. **Expressions.** One conditional expression, `if c: x else: y`. `T` promotes to `?T` implicitly. Arithmetic and
   bitwise operators lift over `?T` (`none + 5` is `none`); ordering doesn't (`?T` isn't `Ord`); `==` is
   structural [you, p3]. **Postfix `x?`** covers what operators don't reach: `a[x?]`, `f(x?)`, `t?.size`,
   `(dp[j]? + w, j)`. If `x` is `none`, the nearest enclosing parenthesized group, call argument or statement
   right-hand side is `none`. `max=`/`min=` treat `none` as the identity on either side (rule 14), so
   `dp[b] max= dp[j] + w` skips unreachable `j`. A bare variant name in a named argument resolves against that parameter's enum
   (`order: desc`). A `let` may shadow an earlier binding in the same block.

8. **Definite initialization, no `++`.** `var a: [V]Int = 0` fills. Bindings above a
   `--- with multitest() ---` line are the only state shared across tests (16.3).

9. **Numbers and storage.** [10, 11, 16]
   - `Int` is i64 with a niche. Fixed widths (`I32`, `U64`) mean wrapping storage: they wrap in release and
     trap in debug **[you]**. Range types (`0..n`) are **contracts**: assigning an unproven value traps in
     both builds, like an index, and `with checks(ranges: off)` removes it. Pick by meaning: a fixed width
     for hot counters whose bound you can't state cheaply, a range for values with a real bound.
   - Storage for a range type is the narrowest integer that holds it. Widening is implicit; narrowing is a
     checked conversion (`I32(x)`), everywhere, including returns.
   - Parameter types may use runtime bounds (`d: 0..day(Cut.last)`), checked at the call and listed by
     `cp explain`. **Struct fields may only use static bounds**, so a struct's layout never depends on input.
   - `[D]T` applies to the whole following type, and range types are atomic. In a destructured tuple type,
     later components may name earlier ones. Reversed ranges are empty; exact exponent literals are `Int`.

10. **Three kinds of sequence.** [14] **Streams** are one-shot (`read`, stateful generators like
    `successors`). **Views** are lazy and can be iterated again (`g.edges`, `x.filter(…)` and `x.map(…)` on
    a view). **Arrays and slices** are stored and indexable (`[D]T`, `a[l .. r]`, `g.adj(v)` over the
    CSR). A domain counts as the array of its positions, so `V.map(f)` builds a `[V]T` [r4 5, 15]. `map`
    and `filter` on an array build an array, reusing a temporary's buffer; on a view they build a view.
    `[D]T` is (pointer, length) and `List<T>` adds capacity; `.list()` on a temporary moves [r13 P5]. Accessors return copies of values
    that fit in a register. `cp explain` lists every allocation.
    - **No invalidation.** Only length-changing operations on a `List` (`push`, `pop`, `resize`, `clear`)
      invalidate, and only views and slices of that same `List`. Element writes never do. It is a compile
      error when the access paths are syntactically the same; otherwise debug checks a per-`List`
      generation count (`adj[w].push` while iterating `adj[u]` traps only if `w == u`). Views and slices
      can't be stored in struct fields or statics [r5 24, 25]; stdlib types like `Graph` and `Buckets`
      store offsets and build slices on access, which user libraries can do too [r6 17]. A slice is
      (pointer, length) into an element buffer, never a borrowed `List` header [r6 15]. Views read live
      elements; a lint flags writing to a view's base while iterating it [r6 16].
    - **Common folds, not hand loops [r8, proposed].** `prefix_sums`, `prefix_max`, `suffix_max` (arrays
      over a slice's elements, or its cuts with `at: .cuts`; `Range<D>.map` keeps `D` as the index), `windows(m)`, `seen.mark(v)` (set and
      return whether it was new), `D.mask(xs)`, `s.pop_min()`, and explicit `Int(b)`/`Z(b)` for indicators.
      `map(f).sum()` stays the one spelling of a mapped fold, and rolling buffers stay `fill` + `swap`.
    - **Neighbours, not loop state [you, proposed].** `xs.pairs()` yields adjacent `(prev, x)` (n - 1 of them);
      `xs.with_prev()` yields `(?T, T)`, with `none` first, for `let p = prev?.after ?? start`. Both are
      views. Loops have no syntax for the previous iteration.

11. **One guard keyword.** `where` filters in `for` and `while let`. Keywords may be argument labels
    (`first(where:)`). Options are named arguments, never bare flags.

12. **Stress testing.** [3, 17, 18, 19]
    - `@alt(f)` marks another implementation of `f`: a brute, or a second fast algorithm. `cp stress` runs
      each against `f`. Size caps name a program-level read or domain (`@alt(solve, A: ..=8)`) or, qualified, a `gen`
      binding (`@alt(find_split, input.n: ..=10)`) [r4 16]. An uncapped alt uses the `stress` context's
      default `max_size` (`with stress(max_size: 1000)`) [r10 22].
    - In whole-program stress an inner function's alt is substituted at its real call sites and compared
      there, so its preconditions hold. Stressing it alone needs its own `@gen` [r4 19, 26].
    - **One checker shape**, a relation: `check: (input, ours, theirs) => Bool`. Equality is the default. A
      program-level `@check fn ok(input, output) -> Bool` is the unary case and takes its types from the
      program's input and output (for a grader: the argument tuple and the return type). The unary check
      runs on every implementation's output; the relation runs between `f` and each alt [r4 17].
    - A unary checker can't catch a wrong "impossible", so problems with an impossible answer also get a
      brute `@alt` with a relation comparing feasibility.
    - Inputs come from one generator system, `gen<T>(…)`; constraints no type states go in `@gen fn
      input()`. The shrinker records `gen` choices. A function that mutates captured state can't have an
      `@alt`.

13. **Interaction.** [13] An `@interactor` is an ordinary program run by `cp stress` on the other end of the
    pipes. The stdlib `Wire` owns error replies, query limits (declared on the message type) and flushing [p3]. **Flush rule:** a program whose code mentions
    `Wire`, and every `@interactor`, flushes pending output before every read and at exit, including
    after `fail`. This is decided at compile time, on the program after alts and tool roots are stripped,
    and local runs use the same mode [r4 13, r5 10]. Other programs never
    flush early.

14. **Segment trees.** [9] One form, `SegTree<D, M, A>(init:)`, queried with `t.fold(range)`. Tags are
    values (`Add(x)`). `Max<T>` and `Min<T>` are monoids on `?T` with identity `none`. An action may define
    `restrict(tag, child, scope)`. **Meaning:** `apply(R, t)` sets each element `e` in `R` to
    `apply(restrict(t, e, S), e)`, where `S` is `fold(R)` before the tag; the tree must agree with that
    whatever nodes `R` splits into, so it folds `R` first and restricts every canonical node against `S`.
    An action may also define `uniform(t) -> Bool`, with the law that a uniform tag ignores `restrict`;
    uniform tags skip the extra fold, which is free when the tag is a literal [r4 8, 9]. `cp explain` shows
    which `apply` calls pay the fold; a caller holding the fold already passes it, `apply(R, t, scope: s)`,
    checked in debug [r5 8]. `cp test` checks the laws on random values and `apply` on random ranges
    against the meaning. Segment tree beats wait for the ports.

15. **Recursion.** The program runs on a big stack (the judge profile's size); nothing infers, annotates or
    counts recursion depth (18.3). Checks turned
    off by `with checks(…)` stay on in debug, so a program that passes debug has no UB in release on that
    input.

**Rejected so far:** `print(Bool)` magic, `@yes`, `@base`, `@unchecked`, `@io(…)`/`@checks(…)` as
attributes (now `with`), `@distinct`, `++`, the monotone `where` subrange, `protocol:` blocks, named `step`
blocks, `CoverSweep`, `Inf<Int>`, `read` mode flags, `_` placeholders, comprehensions, uninitialized `var`,
capitalized fresh-domain bindings, bare flags, labeled domains spelled like counted ones, range-typed struct
fields over runtime values, dynamically scoped contexts, runtime (`dyn`) contexts, length as a generic argument.

---

## 1. Jagged Swaps (CF 1896A): the trivial case

**Problem.** t tests. A permutation of length n (3 to 10). You may swap a[i] and a[i+1] whenever a[i] is
greater than both neighbours. Can you sort it? (Yes exactly when a[1] = 1.)

```
--- with multitest() ---

let n: 3..=10 = read()
let a: Perm<1..=n> = read()
print(yes(a.first == 1))
```

- `Perm<S>` is a bijection from an index onto `S`; `Perm<I, S>` names the index domain, which defaults to
  `[n]` [2]. Here `S = 1..=n` is an **integer range**, so the values are `Int`s and `a.first == 1` compares
  directly (rule 2) [1].

- `a.first` is total: the length type `3..=10` is syntactically at least 1.

**Cold path.** "Possible"/"Impossible": `with io(answers: ("Possible", "Impossible"))` at the top, or
`yes(b, answers: …)` at this one call. Same name either way.

---

## 2. Tourism (CCC 2019 S4, your AC): boundary-heavy DP

**Problem.** n attractions in order (n ≤ 10⁶). Visit them in order, at most k per day, in the minimum
⌈n/k⌉ days. A day scores its best attraction. Maximize the total.

**The fact the solution rests on.** The day ending at cut b is day ⌈b/k⌉. So the cuts that can end day
d − 1 before cut b are the window `b − k ..= (d − 1)·k` (clamped at 0). It's never empty, because
b ≤ d·k. The DP is over cuts.

```
let (domain A, k): (1..=1e6, 1..=A.count) = read()   // attractions; the size is read into the domain [r8 1]
let a: [A]1..=1e9 = read()
type Cut = A.Cut                                      // 0..=n, the gaps between attractions

fn day(b: Cut) -> Int = ceil_div(b.ord, k)
fn day_end(d: 0..day(Cut.last)) -> Cut = Cut.at(d * k)  // last cut of a non-final day; checked per call

print(solve())                                        // functions hoist (rule 6)
```

### The brute, written first

```
@alt(solve, A: ..=8)
fn slow() -> Int:
    var dp: [Cut]Int = 0                              // every cut is reachable; scores are positive
    for b in Cut.rest:
        for j in b.window(k) where day(j) == day(b) - 1:   // the ≤ k cuts before b [r8 6]
            dp[b] max= dp[j] + a[j .. b].max()!
    return dp[Cut.last]
```

- `dp` isn't `?Int`: every cut has a predecessor, the cut `(day(b) - 1)·k`, inside `b.window(k)`, and
  scores are positive, so 0 is a correct start. A defensive `?Int` would add an `if let` and a `!` [Ryan].

- `a[j .. b]` is the attractions between two cuts; it can't be off by one. The `!` stays (rule 1).

### Fast A: monotone stack plus lazy segment tree, O(n log n)

```
fn solve() -> Int:
    var dp: [Cut]Int = 0
    var best = SegTree<Cut, Max<Int>, Add<Int>>(init: none)   // best[j] = dp[j] + max of a over (j, now]
    var stack: List<(top: Int, cuts: NonEmpty<Range<Cut>>)> = []
    for x in A:
        best[x.before] = dp[x.before] + a[x]
        var cuts = Range(x.before)
        while let s = stack.last where s.top <= a[x]:
            best.apply(s.cuts, Add(a[x] - s.top))
            cuts = s.cuts.hull(cuts)                // smallest range covering both [r11 N2, r12 R8]
            stack.pop()
        stack.push((a[x], cuts))
        let b = x.after
        dp[b] = best.fold(b.window(k) & (..=day_end(day(b) - 1)))!
    return dp[Cut.last]
```

- `Max<Int>` folds over `?Int`, so an unset cut is `none`, not a sentinel [5]. The `!` states "this window
  has a set cut", which holds because every cut before `b` is set by the time `b` is reached.

- `day_end`'s parameter type `0..day(Cut.last)` makes the round-2 doc bug impossible: calling it for the
  final day traps at runtime. The check is one compare per call, and `cp explain` lists it.

- `SegTree` has 2·2^⌈log₂(n+1)⌉ nodes: about 17 MB of `?Int` values plus 8 MB of `Add` tags on internal
  nodes for 10⁶ cuts. `cp explain` prints values and tags separately.

### Fast B: the block insight, O(n), no segment tree

```
@alt(solve)
fn solve_linear() -> Int:
    var dp: [Cut]Int = 0
    for today in A.chunks(k):
        let prev = today.before.window(k, incl: true)       // cuts that can end yesterday; clamps on day 1
        let tail = a[prev].suffix_max(at: .cuts)  // [prev]?Int: best after cut j [r11 N35]
        let incl = prev.map(j => dp[j] + tail[j]?).suffix_max()   // ?Int: none = no tail [r11 N36]
        let excl = prev.map(j => dp[j]).suffix_max()
        let run = a[today].prefix_max()                     // [today]Int
        for x in today:
            let j = x.after.back(k)
            dp[x.after] = max(incl[j], run[x] + excl[j])
    return dp[Cut.last]
```

- `@alt(solve)` means `cp stress` checks all three implementations against each other [4].

- `prev` is computed from `today`, not carried across iterations, so day 1 needs no seed [you]. `r.before`
  mirrors `r.after`.

- **Scans replace the hand loop [r8 4, 12].** A scan over a slice is indexed by its elements and is total
  (`run: [today]Int`); `at: .cuts` indexes it by cuts instead, where the empty suffix is `none`. A scan
  over a `Range<D>` view is indexed by `D` (`incl: [prev]Int`) [r9 8, 9]. That deletes the `0` seeds, the reversed loop and the
  day-1 guard `if let x = j.left`. Checked against the brute in `wf/fastb_check.py`.

- **Memory:** the per-day arrays are over `prev` (k + 1 cuts), not all of `Cut`. A loop-local array
  whose size has a loop-invariant bound reuses one buffer across iterations, which only removes
  allocations, so it isn't a representation choice (rule 10). The round-8 version, with `[Cut]` arrays, was
  measured at 1.6–1.7x your memory (`lower/tourism/RESULTS.md`).

- Moving between elements and cuts takes these names: `b.window(k)` (the ≤ k cuts before `b`; empty at
  `Cut.first`), `c.window(k, incl: true)`, `x.before`, `x.after` (element to cut), `j.left`,
  `j.right` (cut to `?element`), `b.back(k)` (clamped), and `r.after`, `Cut.rest` over ranges [9].

### Requirement change

Any number of days, each costing c. Fast A changes two lines: the window becomes `b.window(k)` and the
DP subtracts c. The brute changes two lines the same way: its `where day(…)` filter goes and it subtracts
c. Fast B is dead and gets deleted; its `@alt(solve)` would otherwise fail stress, which is the right
alarm. So A is the main exemplar and B the performance exemplar.

### Cold path

The hot loop needs its remaining checks gone: `with checks(bounds: off)` on the line above `for x in A:`.
It applies to the rest of `solve`, and reaches the checks inside `SegTree.apply` and `fold` because their
module is declared `uses checks`. The top-level helpers `day` and `day_end` stay lexical and checked;
their one check per `b` isn't the hot part [r6 2]. It applies only in release (rules 3 and 15). `cp explain` shows the extra unchecked
copy of each and its bytes.

---

## 3. Tree Queries (CF 1328E): trees and recursion

**Problem.** A tree on n vertices rooted at 1. Each query gives k vertices: is there a root path that every
given vertex is on or adjacent to? (Replace each vertex by its parent; all must then be ancestors of the
deepest.)

```
with io(base: 1)
let (domain V, m): (2..=2e5, 1..=2e5) = read()
let edges: [V.count - 1]Edge<V> = read()             // lengths only ever in [k]
let t = Tree(edges).rooted(at: V.first)               // parent, depth, preorder: one pass, no recursion
let tour = t.euler()                                  // entry/exit times, built because you asked

repeat m:
    let ps = read<[len: 1..=V.count]V>().map(v => t.parent[v] ?? v)  // length-prefixed [r8 5]
    let deepest = ps.max_by(v => t.depth[v])          // total: the length type is at least 1
    print(yes(ps.all(u => tour.in_subtree(deepest, of: u))))
```

- `rooted` builds three arrays of 4 bytes per vertex (`?V`, a narrow depth and the order). Entry and exit
  times cost two more and are built only by `t.euler()`.

- `in_subtree(v, of: u)` is reflexive by its name.

**Variant B: the DFS by hand.**

```
let t = Tree(edges).rooted(at: V.first)
var tin: [V]0..V.count = 0, tout: [V]0..=V.count = 0
var clock: 0..=V.count = 0
fn dfs(v: V):
    tin[v] = clock                                    // checked: clock < n isn't proven (explain lists it)
    clock += 1
    for u in t.children(v): dfs(u)
    tout[v] = clock
dfs(t.root)
```

- Recursion n deep needs nothing: the program runs on a big stack (rule 15).

- A nested recursive function that captures lowers to a context struct with reference members, never
  `std::function`.

**Requirement change.** "On the path or within distance d of it":

```
let up = t.lifting()                                  // n log n, built explicitly; explain shows the bytes
let ps = read<[len: 1..=V.count]V>().map(v => up.ancestor(v, d) ?? t.root)
```

**Cold path.** The input is a parent array with the root's parent written as 0, a sentinel among 1-based
labels. Read plain numbers on that one line:

```
let raw: [V]Int = read()                              // integers never convert
let parent = V.map(v => if raw[v] == 0: none else: V.at(raw[v] - 1))
```

---

## 4. Beautiful Subsegments (CF 1677E): a performance-tight sweep

**Problem.** A permutation p of n (2·10⁵) and q queries (10⁶). Query [l, r]: count subsegments inside it that
contain two positions x < y with p[x]·p[y] equal to the subsegment's maximum.

**The idea.** For each pair of values a < b with a·b ≤ n, the good subsegments form a rectangle in
(start, end) space. Sweep the end r; for each start, count the rectangles covering it, and accumulate how
often each start was uncovered. A query's answer is all pairs minus the uncovered history.

```
with io(base: 1)
let (domain I, domain Q): (1..=2e5, 1..=1e6) = read()
let n = I.count
let p: Perm<I, 1..=n> = read()                        // positions onto the integers 1..=n
let pos = p.inverse()                                 // [1..=n]I, indexed by Int
let span = p.max_spans()                              // [I]Range<I>: where p[z] is the maximum
let qs: [Q]NonEmpty<Range<I>> = read()                // "l r": inclusive on read, half-open inside

var opens: [I]List<Range<I>> = [], closes: [I]List<Range<I>> = []
for a in 1..=n:
    for b in a + 1 ..= n / a:                         // empty once a + 1 > n / a
        let (x, y, z) = (pos[a], pos[b], pos[a * b])
        let starts = span[z].first ..= min(x, y, z)
        let ends = max(x, y, z) ..= span[z].last
        if let s = starts.nonempty(), e = ends.nonempty():   // several bindings, left to right [r11 N1]
            opens[e.first].push(s)
            closes[e.last].push(s)
let by_end = Q.group(by: k => qs[k].last)             // Buckets<I, Q>

var cover = SegTree<I, Cover, Tick>(init: Cover(low: 1, at_low: 1, hist: 0))  // 1 = not a start yet
var ans: [Q]Int = 0
for r in I:
    for s in opens[r]: cover.apply(s, Tick(add: 1))
    cover.apply(r, Tick(add: -1))                     // r becomes a valid start
    let all = cover.fold(I)
    if all.low == 0: cover.apply(I, Tick(ticks: 1), scope: all)
    for k in by_end[r]:
        let len = qs[k].len                           // last == r, so this is r - l + 1
        ans[k] = len * (len + 1) / 2 - cover.fold(qs[k]).hist
    for s in closes[r]: cover.apply(s, Tick(add: -1))

print(ans, sep: "\n")
```

And the algebra, which the contestant writes:

```
struct Cover: low: ?Int, at_low: I32, hist: Int       // min cover, starts at the min, uncovered moments
struct Tick: add: Int = 0, ticks: Int = 0             // ticks: +1 history per element at the range minimum

impl Monoid for Cover:
    id = Cover(low: none, at_low: 0, hist: 0)
    fn op(l, r):
        let lo = min(l.low, r.low)                    // min over ?Int ignores none
        Cover(low: lo,
              at_low: (if l.low == lo: l.at_low else: 0) + (if r.low == lo: r.at_low else: 0),
              hist: l.hist + r.hist)

impl Action<Cover> for Tick:
    id = Tick()
    fn then(t, u) = Tick(add: t.add + u.add, ticks: t.ticks + u.ticks)
    fn apply(t, x) = Cover(low: x.low.map(v => v + t.add), at_low: x.at_low, hist: x.hist + t.ticks * x.at_low)
    fn restrict(t, child, scope) = Tick(add: t.add, ticks: if child.low == scope.low: t.ticks else: 0)
    fn uniform(t) = t.ticks == 0                      // the ±1 adds skip the extra fold
```

- **The law and the code now agree [18].** `scope` is the range's fold before the tag, and the condition is
  `child.low == scope.low`, which doesn't depend on the tag. Checked: identity, composition and
  `apply(t, op(l, r)) == op(apply(restrict(t, l, P), l), apply(restrict(t, r, P), r))` with `P = op(l, r)`
  hold on 100k random cases.

- **The guard is part of the meaning, not a hidden invariant.** A tick adds history to every element equal
  to the minimum *of the range it's applied to*. Round 3 claimed that held on any range, but a bottom-up
  tree splits a range into several nodes and ticked each node's own minimum. Rule 14 now restricts against
  the whole range's fold, which `wf/split.py` checks on non-power-of-two trees. "Uncovered" means minimum
  0, which is what the `if` states.

- **`cover.fold(I)`**, not `.total` [20]. **`qs[k].len`**, not position arithmetic [21].

- **Ties [23].** `max_spans` on a general array gives each maximum the span up to a strictly larger element
  on the left and a larger-or-equal one on the right, so equal maxima don't double-count. A `Perm` has no
  ties.

- **Event lists are plain `[I]List` [r8 11].** The cold path is one edit per list to `Buckets<I, Range<I>>`
  (CSR from one counting sort), which a lint suggests when memory is tight. The CSR lowering measured
  0.73–1.06x a `vector<vector<pair>>` baseline (`lower/beautiful/RESULTS.md`).

- `Cover.low` is `?Int` only for the identity. `at_low: I32` is 4 bytes and is a hot counter whose bound
  isn't cheap to prove, so it's a fixed width, not a range contract that would check every `op` (rule 9).

**Cold path.** Queries 0-based while the permutation is 1-based: `let qs: [Q]NonEmpty<Range<I>> = read(base: 0)`.

---

## 5. Connectivity Addicts (CF 1738F): interactive

**Problem.** t tests. A hidden graph on n vertices; you know the degrees. Query "? u" returns u's next
neighbour. Using at most n queries, colour the vertices so each colour class is connected and the sum of
degrees in a class is at most its size squared.

```
with io(base: 1)
--- with multitest() ---

domain V = read(1..=1000)
let deg: [V]0..V.count = read()
var judge = Wire<Query<V>>(error_reply: -1, limit: [asks: V.count])   // exits on a raw -1; flushes

fn neighbour(v: V) -> V = judge.query(.ask(v))

var groups = Dsu<V>()
var seen: [V]Bool = false
for v in V.sorted(by: deg, order: desc):
    if not seen.mark(v): continue                     // mark: set, and say whether it was new [r8 7]
    repeat deg[v]:
        let u = neighbour(v)
        groups.union(u, v)
        if not seen.mark(u): break

judge.say(.answer(V.map(v => groups.root(v))))
```

- `repeat deg[v]` is safe for isolated vertices; a negative `repeat` count traps.

- `-1` is compared on the raw token before `base` turns it into a `V`, so it never becomes vertex 0 [32].

In the same file, never submitted:

```
enum Query<V>: ask(V), answer([V]V)                   // generic, so it lives at program level [p3]
impl Read + Print for Query<V>:
    fn print(q) = match q:
        ask(v) => "? {v}"
        answer(c) => "! {c}"
    fn read() = match read<Byte>():
        '?' => ask(read())
        '!' => answer(read())
        other => fail("bad token {other}")
impl Limited for Query<V>:
    type Kind = labels(asks)
    fn kind(q) -> ?Kind = match q:
        ask(_) => asks
        answer(_) => none

struct Case: n: 1..=8, edges: List<(Int, Int)>
@gen fn hidden() -> List<Case>(len: 1..=20) = (1..=gen(1..=20)).map(_ =>
    domain V = gen(1..=8)
    let g = gen<Graph<V>>(simple: true)
    Case(n: V.count, edges: g.edges.map((u, v) => (u.ord, v.ord)).list()))   // replayable and shrinkable as one value [p3]

@interactor
fn local_judge(tests: List<Case>, wire: Wire):
    print(tests.len)
    for case in tests:
        domain V = case.n
        let g = Graph<V>.undirected(edges: case.edges.map((a, b) => (V.at(a), V.at(b))))
        print(V.count)
        print(V.map(g.deg))
        var w = wire.typed<Query<V>>(limit: [asks: V.count])   // replies -1 past the limit
        var next: [V]0..=V.count = 0
        loop:
            match w.read():              // exhaustive over Query
                ask(v) =>
                    if next[v] == g.deg(v):
                        print(-1)
                        fail("no neighbour left")
                    print(g.adj(v)[next[v]])
                    next[v] += 1
                answer(colour) =>
                    check(V.group(by: colour).all(c => g.connected(c) and sum(c.map(g.deg)) <= c.len ** 2))
                    break
```

- `local_judge` is written below `with io(base: 1)`, so it reads and prints 1-based vertices even though
  `cp stress` calls it from outside: tool roots take the context where they are declared [r4 12]. It
  lives at program level, so it can't see the solution's `n`, `deg` or `V`; it declares its own [r5 9].

- Deadlocks are fixed by the flush rule, not by a call the author has to remember. Because the program
  constructs a `Wire`, the next test's plain `read()` after `judge.say` flushes too.

- `g.adj(v)` is a slice of the CSR (rule 10), so `g.adj(v)[next[v]]` is O(1) indexing.

- `group` yields only non-empty buckets [30]. The interactor's `check` is the checker, so many valid
  colourings don't cause false failures [31].

- `hidden()` is the test, so a failing case is saved and replayed as one value. `gen` is the same generator stress uses for `read` types, so interactor and input generation shrink the
  same way [35].

**Cold path.** A reply that doesn't fit a typed message (say, a free-form line): use an untyped
`Wire(error_reply: -1)` and `judge.ask<Str>("? {v}")` at that one call. A query limit shared across tests
is one edit: a `var raw = Wire(…)` above the wrap line, with `raw.typed<Query<V>>()` per test.

---

## 6. Split the Attractions (IOI 2019 P2, your AC): a long grader problem

**Problem.** A connected graph (n ≤ 10⁵, m ≤ 2·10⁵) and sizes a, b, c summing to n. Split the vertices into
sets of those sizes with at least two of them connected. Return the labels 1/2/3, or all zeros. You
implement `find_split`.

**The idea.** Call the smallest size s and the middle m. Take a spanning tree and its centroid; removing it
leaves pieces smaller than n/2. One piece with at least s vertices is the small side. Otherwise join pieces
through non-tree edges until the total reaches s (it stays under 2s, so the rest keeps at least m), or report
impossible. Take connected sets of exactly s and m.

### Variant B (the default): steps as code that reads what came before

```
domain Group = labels(1..=3)                          // a, b, c in the grader's order

@grader
fn find_split(n: I32, a: I32, b: I32, c: I32, p: List<I32>, q: List<I32>) -> List<I32>:
    domain V = n
    let g = Graph<V>.undirected(from: p, to: q)       // checked conversion of the grader's lists [r8 14]
    let want: [Group]I32 = [a, b, c]
    let [small, mid, big] = Group.sorted(by: want)
    let (s, m): (Int, Int) = (want[small], want[mid]) // I32 widens implicitly

    let t = bfs(from: [V.first], next: g.adj).tree()
    let centre = t.centroid()
    let (domain Piece, pieces) = t.components(without: centre)

    fn join() -> ?List<Piece>:                        // reads g, pieces, centre, s
        let pg = Graph<Piece>.undirected(g.edges      // g.edges is lazy: no intermediate list
            .filter((u, v) => u != centre and v != centre and pieces.of[u] != pieces.of[v])
            .map((u, v) => (pieces.of[u]!, pieces.of[v]!)))
        let forest = bfs(from: Piece, next: pg.adj)    // one tree per component, each contiguous in order
        var got: List<Piece> = [], total = 0
        for x in forest.order:
            if forest.parent[x] == none:
                (got, total) = ([], 0)
            got.push(x)
            total += pieces.size[x]
            if total >= s: return got
        return none

    let chosen = Piece.first(where: x => pieces.size[x] >= s).map(x => [x])
        ?? join()
        ?? return List.filled(n, 0)
    let picked = Piece.mask(chosen)                   // [Piece]Bool [r8 15]
    let region = V.map(v => pieces.of[v].map(x => picked[x]) ?? false)

    var label: [V]Group = big
    for v in g.connected_take(s, inside: region)!: label[v] = small
    for v in g.connected_take(m, inside: w => not region[w])!: label[v] = mid
    return label.map(x => I32(x.label)).list()        // map allocates once; .list() on a temporary moves
```

In the same file, for stress:

```
@gen
fn input() -> (I32, I32, I32, I32, List<I32>, List<I32>):
    let n = gen<I32>(3..=12)
    domain V = n
    let g = gen<Graph<V>>(connected: true)
    let a = gen<I32>(1..=n - 2), b = gen<I32>(1..=n - 1 - a)
    return (n, a, b, n - a - b, g.edges.map((u, v) => I32(u.ord)).list(), g.edges.map((u, v) => I32(v.ord)).list())

@check
fn valid(input, output) -> Bool = ...                 // types from find_split; sizes match, two sets connected

@alt(find_split, input.n: ..=10, check: (input, ours, theirs) => ours.all(x => x == 0) == theirs.all(x => x == 0))
fn find_split_slow(n: I32, a: I32, b: I32, c: I32, p: List<I32>, q: List<I32>) -> List<I32> = ...  // all labelings
```

- `valid` alone accepts all zeros, so a wrong "impossible" would pass. The brute alt compares only
  feasibility, and `valid` checks the labeling itself.

- `gen<I32>` and the explicit `I32(u.ord)` keep `@gen` free of implicit narrowing. `g.edges` is a view, so
  iterating it twice is fine (rule 10).

- One generic `bfs(from:, next:)` serves the tree, the piece forest and any grid [r16 R11]. A forest from
  every piece needs no `seen` array: a new root starts a new component.

- `(u, v) =>` takes the pair `g.edges` yields (rule 6). The filter also drops edges inside one piece, which
  would otherwise be self-loops [41].

- `I32` widens to `Int` implicitly; `I32(x.label)` narrows explicitly; `[V]T` to `List<T>` is an explicit
  `.list()` [43]. The lambda uses `w`, not the loop's `v` [44].

- `Piece.first` is the first position; `Piece.first(where:)` is search. The label tells them apart, as in
  Swift [45].

### Variant A (the cold path): a step worth testing alone

When `join` deserves its own alternative, it moves to the top level with explicit inputs. The body is
unchanged because the generic parameter keeps the name `Piece` and `s` keeps its type [37]. `Piece`'s size
arrives with `pieces` (rule 2):

```
fn join<V, Piece>(g: Graph<V>, pieces: Partition<V, Piece>, centre: V, s: Int) -> ?List<Piece>:
    ...same body...

@alt(join, check: (input, ours, theirs) => ours.is_some() == theirs.is_some())
fn join_slow<V, Piece>(g: Graph<V>, pieces: Partition<V, Piece>, centre: V, s: Int) -> ?List<Piece>:
    ...try every subset of pieces...
```

The two return different valid lists, so the relation compares feasibility; a separate `@check` on `join`
would validate the list itself. Whole-program stress substitutes `join_slow` inside `find_split`, where a
real partition exists; stressing `join` alone would need a `@gen` that builds one (rule 12).

**Correction to round 2 [40].** Round 1's `join` allocated a `Set<Piece>` per start, not a `[Piece]Bool`,
and it wasn't quadratic. Round 3 keeps the single `seen` array.

---

## Concept ledger

| concept | why existing concepts don't cover it |
|---|---|
| `with` context | scoped settings (base, checks, answers, lints); replaces every one-off attribute |
| `uses` | the hand-off that lets a context reach library internals; declared on public API, inferred privately |
| `@` marker | tools need to know what a declaration is (alt, check, gen, grader, interactor, memo) |
| labeled domain | indices whose written labels are meaningful output (`Group`) |
| `type` alias | naming an associated domain without making it fresh |
| scoped type | `type Z = Mint(mod: p)`: a runtime parameter held by a block-scoped type, the same rule as domains |
| stream / view / array | one-shot input, lazy re-iterable sequences and stored data have different costs |
| `Buckets` | grouped lists in one allocation; `[D]List<T>` has per-key headers and slack |
| `Wire` | error replies, typed messages and flushing shared by every interactive problem |
| `gen`, `check`, `fail` | generators and checkers for stress; one system for every use |
| `restrict`, `uniform` | range-relative lazy tags as a lawful, randomly checkable action, free when uniform |
| `static var` | the one place mutable state crosses tests |
| `Partition<V, P>` | the result of `components`: a fresh domain plus `of` and `size` |

Deleted in rounds 4–6: attribute-style configuration (`@io`, `@checks`), range-typed hot fields, dynamic
context scope, `dyn` contexts, `read(len:)`, `Edges<V, k>`. Still not measured: emitted bytes per stdlib shape and per context
specialization against the 64 KB limit, and the zero-cost claims (the `?Int` fold, the restricted
`apply`, the debug-only generation counts, implicit context arguments). These need the first hand-lowered ports.

---

## Questions for you

1. **Runtime modulus as a scoped type, not a context.** `type Z = Mint(mod: p)` makes a fresh block-scoped
   type, like `domain V = n`, which rules out stale tables and memo caches. Good replacement for `dyn`?
2. **`uses` declared on public API, trait methods and modules; inferred only for nested functions and
   lambdas.** Top-level helpers stay lexical. Is a visible `uses` on your library's public API acceptable?
3. **Tool roots can't capture per-test state.** The interactor declares its own `n` and `V`. OK?
