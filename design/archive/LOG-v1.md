# CP Language Design Log (v1)

This cumulative log records every design-level change made in `SPEC.md` v1
relative to the skeleton in `01-decisions.md` (and resolves all four open
questions raised in `00-grounding.md`).

Total design-level changes this round: **12**.

---

## Entry 1: Unify bounded range syntax to `..<` and `..=` (drop bare `a..b`)

- **Skeleton (`01-decisions.md` #4, #6):** Used `0..<n` in domain declarations
  (#4) while specifying `a..b` and `a..=b` for ranges (#6).
- **Change in `SPEC.md` (Sections 5, 7):** Standardize exclusively on `a ..< b`
  (half-open) and `a ..= b` (closed), plus lower-bounded `a..` for range types
  and open slices. Bare bounded `a..b` is disallowed.
- **Rationale:** Matches every grounding solution in `00-grounding.md`, removes
  an internal syntax contradiction, and enforces P0 by making endpoint
  inclusivity visually unmistakable at every call site.

```cp
domain Year = 0..<n
let lo = w.first ..< m
let hi = m ..= w.last
```

---

## Entry 2: Ban raw `x + k` / `x - k` on linear domains and resolve cut arithmetic (Open Question 1)

- **Skeleton (`01-decisions.md` #4):** Specified checked `x + k` (trapping out
  of range) on domains and left cut arithmetic open.
- **Change in `SPEC.md` (Sections 5, 17):** Make raw `x + k` / `x - k` a
  compile error on linear domains and `I.bounds` cuts. Provide three explicit
  boundary policies uniformly across positions and cuts: fallible stepping
  (`.next`, `.prev`, `.offset(k) -> ?D`), explicitly checked stepping
  (`.offset(k)! -> D`), and total clamped stepping (`.fwd(k)` / `.back(k) -> D`).
  Add `I.between(c1, c2) -> Range<I>` to construct an element range from two
  cuts without type-punning `Range<I>` and `Range<I.bounds>`. Keep wrapping
  `+` / `-` exclusively on cyclic `ring(n)` domains where arithmetic is total.
- **Rationale:** Trapping `x + k` failed our guiding test ("could this line
  compile and trap off the end?") and created the `Tourism` paradox where
  `Cut.clip(b - k ..< b)` would trap on `b - k` before `clip` ran. Clamped
  `b.back(k) ..< b` stays inside `Cut` and cannot underflow.

```cp
// Before (00-grounding.md Tourism friction)
dp[b] = best.max(day_cuts(day(b) - 1) & Cut.clip(b - k ..< b))

// After (SPEC.md v1)
dp[b] = best.max(day_cuts(day(b) - 1) & (b.back(k) ..< b))
```

---

## Entry 3: Resolve `@grader` init-sized state via `@grader(init: fn ...)` file-body unification + `extern fn` (Open Question 2)

- **Skeleton (`01-decisions.md` #10):** Left `@grader` state sized in `init`
  as an open question and did not specify grader callback imports.
- **Change in `SPEC.md` (Sections 11, 15, 17):** For stateful multi-call
  graders, `@grader(init: fn name(params) -> R)` treats the top-level file
  statements as the body of `init`, establishing domains and `let`/`var` state
  in strict top-to-bottom order; subsequent `export fn` / `export mut fn`
  callbacks operate on that initialized state (lowering to a single
  `std::optional<GraderState>` constructed via `emplace`). Add `extern fn` for
  grader-provided callbacks (IOI `press`, `use_machine`) and explicit
  `@export(name: ...)` / `@grader(naming: camel)` ABI mapping.
- **Rationale:** Reuses the "file is a function body" core model without adding
  path-dependent types, uninitialized globals, late-bound domains, or a
  bespoke `state:` block construct.

```cp
@grader(init: fn initialize(n: 1..=1500))
domain V = 0..<n
var cnt: [V]Int = 0

export mut fn has_edge(u: V, v: V) -> Int:
    let hi = max(u, v)
    cnt[hi] += 1
    if cnt[hi] == hi.ord: 1 else: 0
```

---

## Entry 4: Define `fn` vs `mut fn` capture rules for local/file helpers and ban `static var`

- **Skeleton (`01-decisions.md` #2, #8, #10):** Specified `let`/`var` and
  `mut` parameters but did not define whether file-level or nested helpers
  could read or mutate enclosing `var`s (as used in `Drazil Likes Heap` and
  `Horses`), nor whether `static var` existed.
- **Change in `SPEC.md` (Sections 3, 9, 11):** Plain local/file `fn` may read
  enclosing `let` and `var` bindings (non-escaping; forbidden under `@memo`),
  while mutating enclosing `var`s requires `mut fn` (also non-escaping, with
  standard borrow exclusivity). `static var` is explicitly disallowed (only
  immutable `static let` exists).
- **Rationale:** Keeps short CP helpers (`larger_child`, `delete`, `answer`)
  ergonomic without parameter plumbing, makes mutation visible in the helper
  header (`mut fn`), prevents memoization bugs on mutable state, and guarantees
  zero mutable leaks across `@multitest` cases.

---

## Entry 5: Unify inline conditional to `if c: a else: b` only

- **Skeleton (`01-decisions.md` #7):** Listed `if c: a else: b`, while
  `00-grounding.md` mixed both `if c: a else: b` and Python-style postfix
  `a if c else b`.
- **Change in `SPEC.md` (Section 8):** Standardize exclusively on prefix
  `if c: a else: b` and disallow postfix `a if c else b`.
- **Rationale:** One unambiguous form that never collides with loop filters
  (`for x in xs if cond:`) or pattern guards (`while let top = s.last() if ...:`).

---

## Entry 6: Add `Range1<D>` non-empty range subtype and non-empty reduction typing

- **Skeleton (`01-decisions.md` #6):** Defined only `Range` with `.first`,
  `.last`, `.mid` without separating empty from non-empty ranges.
- **Change in `SPEC.md` (Sections 7, 16.1):** Introduce `Range1<D>` (subtype of
  `Range<D>`, guaranteed `len >= 1`) whose `.first`, `.last`, and `.mid` return
  `D` directly rather than `?D`. Views guaranteed non-empty (`runs`, `windows`,
  `chunks`, `longest_window`) yield `Range1<D>`. Reductions (`min`, `max`,
  `argmin`, `argmax`) return `T`/`D` on provably non-empty domains and
  `Range1<D>`, and `?T`/`?D` on possibly empty ranges or filtered streams.
- **Rationale:** Eliminates spurious `!` unwraps on guaranteed non-empty blocks
  (like `runs` and `longest_window`) while making empty-range errors impossible
  to ignore on arbitrary `Range<D>`.

---

## Entry 7: Generalize `sliding` to linear and cyclic domains (Open Question 4) and add explicit `OrderSet<T>` / `Mask<D>`

- **Skeleton (`01-decisions.md` Stdlib):** Listed `longest_window` only for
  two pointers, and `SortedSet` / `BitSet` without explicit order-statistic or
  single-word mask types.
- **Change in `SPEC.md` (Sections 5, 16.1, 16.2, 17):** Add general incremental
  `sliding` supporting both linear domains and `ring(n)` (resolving Open
  Question 4 so O(n) two pointers and O(n log n) `bisect` are equally pleasant
  and explicitly named). Add explicit `OrderSet<T>` (order-statistic tree for
  `kth`/`count_lt`) and `Mask<D>` (single-word 8-byte bitmask for `D.len <= 64`)
  alongside `SortedSet` and `BitSet`.
- **Rationale:** Directly covers corpus patterns (order-statistic trees appear
  in 38 corpus problems; single-word bitmasks appear in bitmask DP) without
  ever switching container representations secretly under the hood.

---

## Entry 8: Add segment-length-aware `Action<T, F>`, `SegTree` walks, and `Rollback` / `undo` blocks

- **Skeleton (`01-decisions.md` #9, Stdlib):** Sketched `SegTree<V>(max,
  update: add)` without specifying length-dependent lazy actions, binary walks
  on segment trees, or scoped rollback for DSU/Mo's/offline connectivity.
- **Change in `SPEC.md` (Sections 8, 9, 10, 16.3):** Define `Action<T, F>` with
  `apply: (F, T, Int) -> T` (receiving segment length so `add`/`assign`/`affine`
  on `sum` and `add` on `max` share one uniform type). Add `SegTree.max_right` /
  `min_left` walks. Add explicit `Rollback` trait, `RollbackDsu<D>`,
  `Journaled<[D]T>`, `defer`, and scoped `undo x, y:` blocks.
- **Rationale:** Covers the full range of lazy segment tree and rollback
  contest problems with zero hidden journaling and zero runtime dispatch cost.

---

## Entry 9: Extend `@multitest` with `case: tc` / `until:` and specify bounded compile-time `const` evaluation

- **Skeleton (`01-decisions.md` #10):** Covered basic `@multitest` + `static
  let`, leaving Code Jam/Hacker Cup case numbering, sentinel-terminated inputs,
  and compile-time table generation unspecified.
- **Change in `SPEC.md` (Sections 3, 11):** Add `@multitest(case: tc)` (binding
  1-based `tc`), `@multitest(until_eof)`, `@multitest(until: cond)`, and
  `@multitest(sum: ...)` budget metadata. Define `const` as compile-time
  evaluated (deterministic, no I/O) with a compiler warning above 16 KB to
  protect the 64 KB submission cap.
- **Rationale:** Handles all multitest styles in the 500-problem corpus (133
  multitest problems, including 16 Code Jam / Hacker Cup tasks) without adding
  any contest-specific core syntax.

---

## Entry 10: Specify 1-byte `Bit` layout, floor `/` and Euclidean `%`, and `Float` safety rules

- **Skeleton (`01-decisions.md` #3):** Listed `Bit` and `Float` without fixing
  `Bit` storage vs `BitSet`, integer division/modulo sign rules, or float
  equality/printing safety.
- **Change in `SPEC.md` (Section 4):** Fix `Bit` as an explicit 1-byte `0`/`1`
  type (never silently bit-packed — use `BitSet<D>` for packed words). Fix `/`
  as mathematical floor division and `%` as non-negative Euclidean remainder
  (zero extra instructions when operands are proven non-negative). Make `Float`
  64-bit IEEE-754 with an `==` lint and required explicit print precision
  (`digits:` or `{x:.N}`).
- **Rationale:** Removes C++'s negative-remainder bug, accidental float exact
  equality / 6-digit default truncation bugs, and any ambiguity over whether
  `[Cell]Bit` secretly packs bits.

---

## Entry 11: Add explicit `ModRing` / `DynMod` for runtime moduli and formalize the Stdlib Inclusion Criterion (Open Question 3)

- **Skeleton (`01-decisions.md` #3):** Included only compile-time `Mod<P>`, and
  left Open Question 3 (where to draw the line on specialized helpers like
  `dist_sum`) unresolved.
- **Change in `SPEC.md` (Sections 1, 4, 16.7, 17):** Keep `Mod<P>` strictly for
  compile-time moduli (4 bytes, compile-time primality check for `/`) and add
  explicit `ModRing` / `DynMod` (Barrett reduction, no hidden global modulus)
  for runtime moduli read from input. Formalize the three-part Standard Library
  Inclusion Criterion, rejecting narrow single-problem helpers like `dist_sum`
  in favor of composable `Range1` subranges and `.len`.
- **Rationale:** Supports runtime-modulus problems without penalizing static
  `Mod<P>` or introducing hidden global mutable state, and keeps the stdlib
  lean and memorable.

---

## Entry 12: Formalize generator push/pull lowering (`Iter<T>`) and explicit escape hatches (`a.raw[i]`, `unchecked:`)

- **Skeleton (`01-decisions.md` #5, #8):** Mentioned views and slices without
  specifying how lazy view chains (`filter`, `map`, `zip`, `runs`, `windows`)
  compile with zero allocations or how hot kernels drop remaining dynamic
  checks.
- **Change in `SPEC.md` (Sections 5, 6, 14, 15):** Unify views under `Iter<T>`
  generators (`yield` / `yield from`) with guaranteed push lowering into
  consuming loops, stack state-machine pull lowering for non-recursive `zip`
  (banning recursive generators in pull positions so there are no hidden heap
  stacks), and explicit `Iterator<T>` structs for stored iterators. Specify
  greppable escape hatches (`a.raw[i]`, `D.at(i)`, `unchecked:`) and
  `cp build --checks`.
- **Rationale:** Completes the zero-cost C++17 lowering guarantee across all
  views and hot DP/matrix loops while keeping every unsafe or unchecked site
  explicit and auditable.
