# Acceptance exemplars 7–10

These four test the frozen round-6 rules (`EXEMPLARS.md`) on what the first six don't cover: strings,
recursive `@memo`, constant and runtime `Mint`, bitmask DP, floats with `approx`, and inline C++. Each one
is a real accepted submission of yours. The goal is to find rule changes, not to add features: anything new
must come from a library or a general rule. Same caveat as before: none of this compiles yet.

New in this file, all [proposed]:
- **`@memo` table choice is explicit.** When every parameter has a bounded type, the cache is a dense
  array sized by the product of those bounds, and `cp explain` shows its bytes. Otherwise it is a
  compile error naming `@memo(hash)`. A dense table never silently becomes a hash map.
- **Interval arithmetic on range types** (`+ - * / %`) [r7 6]. Floor division means `x % d` has type
  `0..d` for any `x` when `d: 1..`, so indexing a `[0..d]T` with it needs no check. C++ gets a plain `%`
  when the range types show `x ≥ 0`, and a floor-adjusted one otherwise.
- **One `Index` trait** (`count`, `ord`, `from_ord`) [r7 18]. Rule 2's three kinds of index space are
  implementations of it, and so is the stdlib's `Bits<D>` (a set over a small domain, stored in the
  narrowest unsigned integer), so `[Bits<D>]T` is a table over subsets. Only counted domains convert under
  `io(base:)`. User types can implement `Index` too.
- **`@cpp(...)` overrides a function written in the language** [r7 21, 22]. The body in the language is
  the definition, used by the interpreter and tracer and checked against the C++ by stress, like an
  `@alt`. The C++ is emitted as an `inline` function, so each argument is evaluated once.
- **Wrapping arithmetic is explicit** [r7 17]. `U16` and friends trap on overflow in debug, so bit
  tricks use `wrapping_neg`/`wrapping_add` or stdlib helpers such as `w.lowest_bit()`.

---

## 7. AtCoder DP S — Digit Sum

*Count the integers in `1..=K` whose digit sum is divisible by `D`, mod 1e9+7. `K` has up to 10^4
digits, and `D` ≤ 100.*

Your C++ uses a memoized `dp(ind, lim, cmod)` over a `memset(-1)` table, then subtracts the empty number.

### Ideal: recursion with `@memo`, the way the recurrence is stated

```
type Z = Mint(mod: 1_000_000_007)
let k: Str(len: 1..=1e4) = read()                      // bounded, so the memo table has a static size [r11 N27]
let d: 1..=100 = read()
let digits = k.digits()                                 // [k.len]0..=9

@memo
fn count(i: 0..=k.len, r: 0..d, tight: Bool) -> Z:
    if i == k.len: return Z(r == 0)                     // explicit Bool → number [r8 8]
    let top = if tight: digits[i] else: 9
    return (0..=top).map(c => count(i + 1, (r + c) % d, tight and c == top)).sum()

print(count(0, 0, true) - 1)                            // minus the number 0
```

- **Cache:** every parameter is bounded (`0..=k.len`, `0..d`, `Bool`), so the cache is a dense
  `[0..=k.len][0..d][Bool]?Z` array, 8 MB at the maximum: `?Z` is 4 bytes because values ≥ the
  modulus are a free niche [r7 4]. Over the profile's memory, it's a compile-time warning [r7 7]. `cp explain` prints that size at the `@memo`.
- **Lifetime:** `count` is nested in the file body, so the cache is per test. A `@memo` function can't
  outlive the values its bounds mention.
- **Depth:** the recursion is 10^4 deep; the big stack covers it with no annotation (rule 15).
- **`(r + c) % d`** has type `0..d` by interval arithmetic, so the call needs no range check.
- **`.sum()` over `Z`** starts at `Z`'s zero, and the `1`/`0` literals take the type `Z` from the return.

### Variant A: forward DP, no recursion (faster, same length)

```
var free: [0..d]Z = 0                                  // counts of prefixes already below k
var next: [0..d]Z = 0
var tight: 0..d = 0                                    // digit sum of k's prefix, mod d
for x in digits:
    next.fill(0)
    for r in 0..d:
        for c in 0..=9:
            next[(r + c) % d] += free[r]
    for c in 0..x:
        next[(tight + c) % d] += 1
    swap(free, next)
    tight = (tight + x) % d
print(free[0] + Z(tight == 0) - 1)
```
Variant A keeps two rolling `d`-sized arrays instead of the whole table and has no recursion; `@memo`
reads closer to the statement [r7 2, 3].

### Requirement change: count in `[L, R]` instead of `[1, K]`

Make the counter a function of the bound and subtract. That beats decrementing a decimal string:

```
fn upto(k: Str) -> Z:                                   // numbers in 0..=k with digit sum ≡ 0
    let digits = k.digits()
    @memo
    fn count(i: 0..=k.len, r: 0..d, tight: Bool) -> Z: ...     // unchanged
    return count(0, 0, true)

let (l, r): (Str(len: 1..=1e4), Str(len: 1..=1e4)) = read()
print(upto(r) - upto(l) + Z(l.digits().sum() % d == 0))
```
Each `upto` call gets its own cache, because the cache lives in the activation that declares `count`.

### Cold path: modulus read from input

```
let p: Prime = read()                                   // checked prime, or `Int` for a non-prime Mint
type Z = Mint(mod: p)
```
One edit: the declaration changes and nothing else does. `Z` is now scoped to the file body, so it
can't reach a `static`, and `count`'s cache can't outlive it. `cp explain` shows Barrett reduction at the
declaration, about 1.3x a constant-modulus multiply.

---

## 8. CCC 2003 S4 — Substrings

*For each of `t` strings (length ≤ 5000), count its distinct substrings, including the empty one.*

Your C++ writes a templated polynomial `Hash<mod>` with `bpow`, `inv` and inverse powers. It runs
double hashing at two moduli per substring length, with a hand-rolled open-addressing table.

### Ideal: the suffix array is the library's job

```
--- with multitest() ---
let s: Str = read()
let sa = SuffixArray(s)                                 // SA-IS, O(n)
print(1 + zip(sa.order, sa.lcp).map((p, h) => s.len - p - h).sum())
```

- **`Str` is indexed by the integer range `0..s.len`** [r7 12], so positions are plain integers and
  `s.len - p - h` is ordinary arithmetic. **`sa.order`** is `[Rank]0..s.len` and **`sa.lcp`** is
  `[Rank]0..=s.len`, where `lcp[r]` is the overlap with rank `r - 1`. `Rank` is a fresh counted domain,
  so a rank can't index the string.
- **`1 +`** is the empty substring. It stays in your code because it's the problem's convention.
- **Cost:** about 2 KB of emitted SA+LCP, well under the 48 KB warning [r7 14].

### Variant A: hashing, when the problem needs it (comparing two strings, rolling updates)

```
--- with multitest() ---
type H = Hash2                                          // stdlib: (Mint(mod: P1), Mint(mod: P2)), random base
let s: Str = read()
let h = s.prefix_hashes<H>()
var total = 1
for len in 1..=s.len:
    total += HashSet<H>(s.windows(len).map(w => h[w])).len     // windows: slices of one length [r8 6]
print(total)
```

- **`h[i .. j]`** is the substring's hash. The prefix differences and inverse powers live in the
  library, and so does the `bpow`/`inv` your C++ wrote by hand.
- **Random base:** `Hash2` picks the base on the judge and prints the seed to stderr. Local runs use a
  fixed seed, and stress treats the seed as a generator input that the shrinker can minimize [r7 13].
- **`HashSet<H>`** replaces the hand-rolled 8192-slot table. Ordered `Set` stays the default (locked),
  and hashing is opt-in by name [r7 11].

### Requirement change: also print the most frequent substring of length `m`

```
let m: 1..=s.len = read()
let counts = HashCounter(s.windows(m).map(w => h[w]))
let best = s.windows(m).max_by(w => counts.of(h[w]))  // total: m ≤ s.len; absent keys count 0
print(best)                                             // a slice prints as text [r11 N33]
```
`s[i .. j]` is a slice of bytes and prints as text. Only the new lines change.

---

## 9. AtCoder DP U — Grouping

*Split `N` ≤ 16 rabbits into groups to maximize the sum of `a[i][j]` over pairs in the same group.*

Your C++ computes `pcomp[mask]` in O(2^n n^2), then enumerates submasks with `j = (j - 1) & i`.

### Ideal

```
domain R = read(1..=16)
let a: [R][R]Int = read()

var score: [Bits<R>]Int = 0
for s in Bits<R>.nonempty:
    let (x, rest) = s.pop_min()                         // total on non-empty sets [r8 3, 13]
    score[s] = score[rest] + rest.map(y => a[x][y]).sum()

var best: [Bits<R>]Int = 0
for s in Bits<R>.nonempty:
    let (x, rest) = s.pop_min()
    best[s] = rest.subsets().map(t => score[t + x] + best[rest - t]).max()   // subsets include ∅
print(best[Bits<R>.full])
```

- **`for s in Bits<R>.nonempty`** iterates in numeric order, so subsets always come before supersets. The DP relies on
  that, and the docs promise it.
- **`s.subsets()`** is submask enumeration. It compiles to `t = (t - 1) & s` with no allocation.
- **Fixing the group of the minimum** removes double counting. Enumerating `rest.subsets()` really
  halves the work, where filtering `s.subsets()` would not [r7 16]. It's an algorithm choice, so it stays
  in your code.
- **`score`** is O(2^n n) instead of O(2^n n^2): `s - x` drops one element.
- **Representation:** `Bits<R>` is a `U16` because `n ≤ 16` fixes the width; `[Bits<R>]Int` is 2^16
  entries. `cp explain` shows both.
- **No `!`:** `Bits<R>.nonempty` yields sets whose type is non-empty, so `pop_min` is total, and
  `subsets()` always includes `∅`, so its `max` is total. Both are stdlib types, not prover facts.

### Cold path: raw bit tricks

```
let w = s.raw                                           // U16
let low = w & w.wrapping_neg()                          // or w.lowest_bit()
let s2 = Bits<R>.from_raw(w ^ low)                      // checked: no bits ≥ n
```
`raw` and `from_raw` are the escape hatch. Everything else in `Bits` is built on them, so a user library
can build its own set type with the same cost.

### Cold path: an intrinsic the stdlib doesn't wrap

```
@cpp("_pdep_u64(x, mask)", includes: "<immintrin.h>", target: "bmi2")
fn pdep(x: U64, mask: U64) -> U64:
    var out: U64 = 0, bit: U64 = 1, m = mask
    while m != 0:
        if x & bit != 0: out |= m.lowest_bit()
        m &= m - 1
        bit <<= 1
    return out
```

- The body is the meaning; the interpreter, tracer and debug builds run it. Stress compares the two.
- The C++ is emitted as an `inline` function, so arguments are evaluated once. `includes` and `target`
  are emitted once each. The profile says whether the judge supports `bmi2`; if not, the bundler fails
  rather than risk SIGILL, and `#pragma GCC target` scope is shown by `cp explain` [r7 22].
- The no-UB guarantee covers everything except `@cpp` bodies, and `cp explain` lists each one [r7 23].

---

## 10. CEOI 2017 — Sure Bet

*There are `n` outcomes of type A with odds `a_i` and `n` of type B with odds `b_i`. Each bet costs 1.
Choose bets to maximize `min(ΣA chosen, ΣB chosen) − (number chosen)`. Print with 4 decimals.*

Your C++ binary-searches the answer 200 times over `double` with a greedy check.

### Ideal: a greedy with two pointers, no binary search

```
with io(precision: 4)
let n: 1..=1e5 = read()
let bets: [n](F64, F64) = read()

fn solve() -> F64:
    let a = bets.map(b => b.0).sorted(order: desc)    // [n]F64: anonymous lengths index by Int [r8 2]
    let b = bets.map(b => b.1).sorted(order: desc)
    var (sa, sb, i, j) = (0.0, 0.0, 0, 0)
    var best = 0.0
    while i < n or j < n:
        if j == n or (i < n and sa < sb):
            sa += a[i]
            i += 1
        else:
            sb += b[j]
            j += 1
        best max= min(sa, sb) - F64(i + j)
    return best

print(solve())
```

- **`sa < sb`** is an ordered comparison, which is allowed. Only `==`/`!=` on floats are rejected by
  default.
- **`F64(i + j)`** is an explicit conversion. There's no implicit int-to-float.
- **`with io(precision: 4)`** sets fixed four-decimal output for this file. `print(x, precision: 6)`
  overrides it at one call.
- **`i` and `j`** are plain `Int`, and `a`/`b` are `[n]F64` indexed by `0..n`, so each access has one
  bounds check that `cp explain` lists. No prover changes a type [r7 26].
- **Default precision** without `with io(precision:)` is the profile's (6 for most judges) [r7 31].

### Stress: the brute and a float checker

```
@alt(solve, n: ..=8, check: (input, ours, theirs) => approx(ours, theirs, abs: 1e-4))
fn brute() -> F64:
    type S = Bits<0..bets.len>                       // Bits takes any Index, ranges included [r9 13]
    let value = (x: S, y: S) =>
        min(x.map(i => bets[i].0).sum(), y.map(i => bets[i].1).sum()) - F64(x.len + y.len)
    return S.map(x => S.map(y => value(x, y)).max()).max()
```
`approx(a, b, abs:, rel:)` is an ordinary function and the only float equality. Its tolerance has no
built-in default; `with approx(abs: 1e-6)` sets an explicit per-file one [r7 28, 29]. `with lint(float_eq: allow)`
re-enables `==` for the rare exact case, such as values known to be integral.

### Requirement change: print which bets to take

```
var taken: List<(Side, Int)> = []
...
        sa += a[i]
        taken.push((Side.A, i))
        i += 1
...
```
Recording the best prefix needs `best_k = i + j` and a replay. Because the greedy is a single pass, the
replay is the same loop stopped at `best_k`. This is the case for writing the loop as a function
`fn greedy(stop: ?Int)`, the same way chunk functions are used in Split.

---

## What these four found

| Gap | Rule change |
|---|---|
| `@memo` silently picking hash vs dense | Dense only for bounded parameters; otherwise `@memo(hash)` must be written |
| `%` results needing a check to index | Interval arithmetic on range types |
| Subset DP needing raw ints everywhere | `Bits<D>` implements a general `Index` trait; `raw`/`from_raw` escape |
| Inline C++ with no trace story | `@cpp(...)` overrides an in-language body, checked by stress |
| Bit tricks trapping in debug | Explicit wrapping ops (`wrapping_neg`) and helpers (`lowest_bit`) |
| Float tolerances varying per site | `approx(a, b, abs:/rel:)` with no built-in default |
| Hash constants and bases | `Hash2` in the stdlib, random base, `with hash(seed:)` for repeatability |

Comprehensions stay rejected: every fold here is `.map(...).sum()` or `.max()`, and none read worse for it.

Round 7's review (`wf/grill-r7.md`, 31 items) found no algorithm bugs. Its fixes are applied above.
Round 8 settled it: an anonymous `[n]T` is indexed by the integer range `0..n`; only named domains are
nominal [r8 2].

## Questions

1. **`@memo` dense-by-default** for bounded parameters (an ordinary `[bounds]?T`), `@memo(hash)` explicit. OK?
2. **`@cpp(...)` as an override of an in-language body**, never a bodiless extern. OK?
