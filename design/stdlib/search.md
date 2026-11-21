# Search on answers (`std.seq`)

Owner: DP and search lane. Surface: `plan/STDLIB.md` 4.2, the search
block. The DP toolkit that calls these searches is `design/stdlib/dp.md`.

The job: "the answer is the first `t` where a monotone check flips" in one
line, over any index space, integer or real, one query or a batch, without
a hand-written midpoint. Sorted lookups (`first_ge` and friends) are the
sequences area's; this file covers searching a predicate on an answer
space.

## 1. What it has to cover

Sources, and what they ask for:

- **Ports.** 28 ports wrote a `while lo < hi` loop (2026-11 count, before
  the rewrites in 6), mostly because the answer space was `0..=2e18` or
  real, or because the predicate needed scratch state. `cf2183h` needs a
  search over `I128` slopes; `cf2245g` over a suffix of a pool; AGC 002 D
  (`design/stdlib/dp.md` 5.3) a batch of searches over one timeline.
- **ecnerwala's library** (`proto/ports/ecnerwala/*_orig.cpp`): binary
  search is always inline (`while (hi - lo > 1)`), with the half-open
  invariant in a comment; ternary search appears as golden section on reals.
- **KACTL** *(judgement, from its table of contents)*: `BinarySearch`
  (integer, on a predicate), `TernarySearch` (integer, unimodal),
  `GoldenSectionSearch` (real). The AtCoder Library has none; its
  `max_right`/`min_left` are segment-tree descents (ranges area).
- **Contest usage** *(judgement)*: binary search on the answer is among
  the most common techniques in Codeforces Div. 1/2 and ICPC sets; parallel
  binary search and ternary search on a convex cost are standard hard-end
  tools.

Patterns, each one call:

| pattern | call |
|---|---|
| least `t` with `ok(t)`, monotone | `bsearch_first(r, ok)` |
| greatest `t` with `ok(t)` | `bsearch_last(r, ok)` |
| real threshold | `bsearch_real(lo, hi, ok)` |
| peak / valley of a unimodal integer `f` | `ternary_max(r, f)` / `ternary_min(r, f)` |
| peak / valley of a unimodal real `f` | `ternary_max_real` / `ternary_min_real` |
| many monotone queries over one timeline | `parallel_bsearch(q, r, reset, apply, ok)` |
| any `t` with `p(t)`, not monotone | `first(r, p)` / `r.first(p)`, O(n) |

## 2. The API

```cp
fn bsearch_first<I>(r: Range<I>, ok: I -> Bool) -> ?I
    // O(log r.len) calls of ok. The first i in r with ok(i); ok is false
    // then true along r. none when ok fails everywhere.
fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I
    // O(log r.len) calls of ok. The last i in r with ok(i); ok is true then
    // false. none when ok fails everywhere.
fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64
    // steps + 1 calls of ok. The least x in [lo, hi] with ok(x), to within
    // (hi - lo) / 2^steps; ok is false then true. none when not ok(hi).
fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I
fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I
    // O(log r.len) calls of f. A position of the maximum (minimum) of f on
    // r; f strictly rises then strictly falls (falls then rises), with a
    // plateau allowed only at the optimum. none for an empty r.
fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
    // steps + 2 calls of f (golden section reuses one probe per round). An
    // x in [lo, hi] within (hi - lo) * 0.618^steps of the optimum of a
    // unimodal f.
fn parallel_bsearch<I>(queries: Int, r: Range<I>, reset: () -> (),
                       apply: I -> (), ok: Int -> Bool) -> List<?I>
    // O((r.len + queries) log r.len) calls of apply and ok, O(queries)
    // memory: ceil(log2(r.len + 1)) rounds of one reset, r.len applies in
    // order and one ok per open query. out[q] is the first t in r such that
    // ok(q) holds once apply ran on every t' <= t; none when it never
    // holds. ok(q) is monotone in the applied prefix.
```

**`bsearch_*` is the O(log n) monotone form of `first`/`last`.** `first(r,
p)` and `r.first(p)` scan and accept any `p`; `bsearch_first(r, ok)` asks
O(log n) questions and needs `ok` monotone. Same answer type, same
argument order; the prefix names the cost and the law. `plan/STDLIB.md`
section 2 says this once for every area.

**Answers are positions, never sentinels.** The integer searches return
`?I`, a position of the searched range's own index space (a `V` for
`V.all`, a cut for `V.bounds`, an `Int` for `lo..=hi`). `bsearch_real`
returns `?F64` because "nothing in `[lo, hi]` works" differs from `hi`.
`steps` is the one knob, so it is a named argument with a default.

**`ternary_*` takes any ordered `K`,** so ties break by a second key
(`f(x) = (cost, -x)`). Integer ternary search is
`bsearch_first(r without its last, i => f(i) >= f(i + 1))` (for a maximum):
the plateau rule is exactly what makes that predicate monotone.

**`parallel_bsearch` takes the timeline as three operations,** not a
structure: the state (a `Dsu`, a `Fenwick`) is whatever the user's `reset`
builds, in the user's file. `reset`/`apply` are named fns that write the
user's variables, since a lambda cannot assign.

## 3. Laws, and how they are checked

| function | law | checker |
|---|---|---|
| `bsearch_first` / `_last` | `ok` monotone along `r` | ends checked in debug |
| `bsearch_real` | `ok` monotone on `[lo, hi]` | none |
| `ternary_*` | strictly unimodal, plateau only at the optimum | `std.spec.unimodality_break` |
| `parallel_bsearch` | each `ok(q)` monotone in the applied prefix | per-query `bsearch_first` as the brute |

A broken law gives some position, never a crash; the checker finds the
smallest witness on small inputs in `cpl test`.

## 4. What stays user code

The predicate, and anything it keeps between probes. A check that rebuilds
a structure per probe (`cf2245g`'s query prefix plus one element) writes a
scratch list and `pop`s it back; the copy is a slice question for the
sequences area, not a search option. Choosing the answer space is user
code too: `0..=2e18` is fine, since the midpoint never overflows.

Not in the design, one dream each: a monotone search over an ordered
`Set`'s elements (`tops.bsearch_first(q => a[q] > s)`, `cf2262b`); the set's
own ordered queries cover it once a second problem needs it. A method form
`r.bsearch_first(ok)` is the free form until decision 53 covers searches.

## 5. Worked problem: AGC 002 D, Stamp Rally

`design/stdlib/dp.md` 5.3 is the full program: one `Dsu` built by
`reset`/`join`, a `reaches(i)` check per query, and
`for t in parallel_bsearch(q, E.all, reset, join, reaches): print(t!)`.
The answer is an `E`, an edge of the timeline, with no `+ 1`.

A one-query example, the classic "least time to make `k` products":

```cp
let (n, k): (1..=200_000, 1..=10**9) = read()
let speed: [0..n]1..=10**9 = read()
print(bsearch_first(0..=10**18, t => sum(t / s for s in speed) >= k)!)
```

## 6. Frictions and the regression port

- **`cf2183h`, I128 slopes** (`research/frictions/cf2183h.md`). The natural
  `bsearch_first(fewest..=I128(0), l => solve(l).groups >= k)!` fails with
  `E0302: expected I128, found I128`: `Range<I128>` does not unify in the
  checker. This is the search's regression port: the rewrite replaces the
  hand loop with `bsearch_first` and the Aliens bookkeeping with
  `std.dp.aliens`. Blocked on the core lane for the range fix (reported
  there); until then `aliens` and the search take `Int`.
- **`cf2245g`, scratch state per probe.** Answered in 4: `bsearch_first`
  over `lo..pool.len` already returns `?Int`.
- **Hand-rolled loops.** The remaining gating ports' `(lo + hi) / 2` sites
  are segment-tree recursions and interactive halving, not searches on a
  predicate; the retired scout ports (`proto/cp.py` interpreter) are not
  rewritten.

## 7. The maximal version, and the next step

Every search names its law and checks it: debug builds sample the
predicate for monotonicity (`bsearch_*`) and unimodality (`ternary_*`)
and report the first violating pair in source terms, so a wrong
observation fails on the sample. Searches go generic over any ordered
integer type (`I128`, `U64`) and over cut spaces, and `parallel_bsearch`
gains an undo-based form (`mark`/`undo_to`) that skips `reset`.

Next step: the `I128` range fix in the core, then the `cf2183h` rewrite
as the gating port for both this file and `std.dp.aliens`.
