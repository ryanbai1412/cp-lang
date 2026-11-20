# Friction backlog, batch 11: number theory and linear algebra

A Stage-1 scout pass over ten submissions: three matrix powers (two
modular, one max-plus), two determinants, one generalized CRT, two sieves,
one Mobius sum, one modular counting DP and one polynomial problem. The
ports are in
[`proto/ports/scout_batch_11/`](../proto/ports/scout_batch_11/):
canada21p5, noi20p1, noi21p2, noi07p5, noi18p4, noip21p1, sdoi17r1p3,
sdoi17r1p1, noip21p2 and shoi16p3. Nothing in `crates/` or `proto/cpc/`
was changed. Items are numbered `N1`... so they do not collide with `F`
(batch 1), `G` (batch 3), `B` (batches 2 and 4), `K` (batch 5), `S`
(batch 6), `D` (batch 7), `H` (batch 8) or `T` (batch 10).

What was checked: all ten ports agree with their originals on random
inputs (`proto/ports/scout_batch_11/check.py`, 20 to 200 inputs each) on
top of commit `fe7ca6d`. Three build with `proto/cpc`; seven run on
`proto/cpi` with small inputs only, and none of those seven was run at
full size. Every "today" snippet is a line a port contains. Every quoted
message was produced by the prototype named with it. The "wanted" snippets
do not build. The first count is exact: the ports of this batch that have
the line. A corpus count is a regular-expression search over the 710
problems with an accepted C++ submission, so it estimates how often the
pattern is written and is not a list of problems that were read.

The request also named XOR bases and NTT. No accepted submission outside
the existing ports uses either, so this batch has no evidence for them
beyond N4 (row reduction) and N10 (convolution).

## Suggested order for a Stage-2 sprint

1. N1, N2, N3: one `Matrix` over a `Semiring`, with `pow`. 4 of 10
   between them, and the three longest hand-written helpers of the batch.
2. N5, N6, N7: the two prototypes disagree on `Mod` with `Int`, on `sum`
   over `Mod`, and on bounded values. 6 of 10 hit at least one.
3. N4: row reduction over a field, 2 of 10.
4. N8 and N9: stepping through multiples and the sieve's missing tables,
   3 of 10.
5. N10: interpolation and convolution over `Mod`, 2 of 10.
6. N11 and N12: congruences and quotient blocks, one port each.
7. N13 to N16: smaller items.

## N1. No matrix: the product is written out in every port (3 of 10: canada21p5, noi20p1, noi21p2)

Today, in canada21p5, with `noi21p2` and `noi20p1` each carrying their
own copy:

```cp
type Mat = [Off, Off]Z

fn mul(a: Mat, b: Mat) -> Mat:
    var c: Mat = [Off, Off] (i, j) => Z(0)
    for i in Off:
        for j in Off:
            if a[i, j] != Z(0):
                for l in Off: c[i, l] += a[i, j] * b[j, l]
    c
```

plus an identity (`[Off, Off] (i, j) => Z(if i == j: 1 else: 0)`) and a
row-times-matrix `apply` in noi20p1. That is 8 to 20 lines per port, and
the originals carry the same code as `operator*` templates. The inner line
`c[i][k] += a[i][j] * b[j][k]` appears in 10 of the 710 corpus problems.

Wanted:

```cp
use std.matrix.{Matrix}

var step = Matrix<Off, Off, Z>.zero()
...
print((step ** k)[Off(0), Off(d)] / comb.choose(n, d))
```

Proposal: `std.matrix` with `Matrix<R, C, T>(ring: Semiring<T>)` stored as
one flat `[R, C]T` table. `a * b` needs `a`'s columns and `b`'s rows to be
the same domain and gives a `Matrix<R, D, T>`; `Matrix.identity()`,
`m ** e` for square ones (N3), and `row * m` for a `[R]T`. The ring
defaults to `+` and `*` when `T` has them, so `Matrix<V, V, Mod<P>>` needs
no argument. `examples/local-lib/math/matrix.cp` is this for `Mod` only,
indexed by `Int`, stored as a list of lists; the general one replaces it.

noi21p2 shows what the index types must allow. Its layers have sizes read
at run time and there are k of them, so there is no fixed set of named
domains, and the port falls back to `List<List<Z>>` with `u - 1`:

```cp
var step = [[Z(0) for _ in 0..size[Layer(g.to_idx + 1)]] for _ in 0..size[Layer(g.to_idx)]]
...
    step[u - 1][v - 1] = Z(1)
paths = mul(paths, step)
```

So `Matrix<0..r, 0..c, T>` over anonymous count domains has to work too,
with the shape checked when the product runs.

## N2. Max-plus needs a hand-made "no walk" on every line (noi20p1)

Today:

```cp
const NONE = -4e18                           // "no walk": the zero of max-plus
...
            if a[i, j] > NONE:
                for l in State:
                    if b[j, l] > NONE: c[i, l] = max(c[i, l], a[i, j] + b[j, l])
...
    if best[at(f.city, 0)] > NONE: best[at(f.city, 0)] += f.bonus
```

Ten comparisons with `NONE` in 42 lines. Leaving one out is not caught
where it happens: `NONE + NONE` still fits in an `Int`, so the bad value
travels until a later sum overflows or a wrong maximum is printed.

Wanted:

```cp
var day = Matrix<State, State, ?Int>.zero(MaxPlus<Int>)
for r in roads: day[at(r.u, 0), at(r.v, r.w - 1)] = joy[r.v]
```

Proposal: `Semiring<T>(zero, one, add, mul)` in `std.algebra` (it is in
`plan/13-abstraction.md` and not in `stdlib/algebra.cp`), with `MaxPlus<T>`
and `MinPlus<T>` over `?T` where `none` is the zero: `add` is `max` that
ignores `none`, `mul` is `+` that keeps it. The same two values serve
Floyd-Warshall and min-plus DP steps. This is a library value, not a new
kind of number. 1 of 10 here.

## N3. A power of something that is not a number (3 of 10: canada21p5, sdoi17r1p3, noi20p1; batch 3 G13)

G13 asked for `power(x, k, op, one)`. This batch adds two cases it does
not cover.

No identity. sdoi17r1p3 raises a vector of residue counts to the n-th
power under cyclic convolution. The identity exists but is one more thing
to build, so the port starts from `x` and runs `e - 1` steps:

```cp
fn power(x: Vec, e: Int) -> Vec:
    var base = x
    var acc = x
    var t = e - 1
    while t > 0:
        if t % 2 == 1: acc = conv(acc, base)
        base = conv(base, base)
        t /= 2
    acc
```

Many exponents, one base. noi20p1 jumps k + 1 different distances with the
same matrix, and applies each power to a row (O(s^2)) rather than
multiplying matrices (O(s^3)):

```cp
var pows = [day]
while 2 ** pows.len <= t: pows.push(mul(pows[pows.len - 1], pows[pows.len - 1]))
...
    for b in 0..pows.len:
        if (f.day - now) / 2 ** b % 2 == 1: best = apply(best, pows[b])
```

Proposal: G13's `power` takes a `Semigroup` and `e >= 1`, or a `Monoid`
and `e >= 0`; and `Powers(x, op, up_to: t)` holds the squarings, with
`p.fold(e)` for the power and `p.apply(e, v, act)` to push a value through
them. The same table is binary lifting on a functional graph. A binary
power loop is written in 44 of the 710 corpus problems.

## N4. Row reduction is written twice (2 of 10: noi21p2, noi07p5)

Today, identical in both ports:

```cp
fn det(m: Mat) -> Z:
    var (a, d) = (m, Z(1))
    let n = a.len
    for c in 0..n:
        var pivot = c
        while pivot < n and a[pivot][c] == Z(0): pivot += 1
        if pivot == n: return Z(0)
        if pivot != c:
            swap(a[pivot], a[c])
            d = -d
        d *= a[c][c]
        let inv = a[c][c].inv
        for r in c + 1..n:
            let f = a[r][c] * inv
            for j in c..n: a[r][j] -= f * a[c][j]
    d
```

The original of noi21p2 spends 62 lines and three `goto`s on it. With `det`
in the library, noi07p5 would be about 15 lines: the Laplacian as a
comprehension and a loop around `guess_at`.

Proposal: `std.linalg` over any `T` with `+ - * /` (so `Mod<P>` for prime
`P`, and `Float`): `row_reduce(mut a) -> (rank: Int, det: T)` as the one
routine, and `det`, `rank`, `solve(a, b) -> ?[C]T` and `inverse` on top of
it. Over GF(2) the same module holds `XorBasis` with `insert`, `reduce`
and `max`; this batch has no problem that needs it, so that half is
untested here.

## N5. `Mod` and `Int` mix in one prototype and not in the other (5 of 10)

*Checked.* `proto/cpc` builds sdoi17r1p3's `all[R(x % k)] += 1` and
noip21p2's `var weight: Z = 1`. `proto/cpi` stops on the same shape:

```
`+` needs two Mod<1000000007> values, got 5
```

so five of the interpreter ports wrap their constants, 34 times in total:

```cp
step[u - 1][v - 1] = Z(1)
det([[Z(if i == j: min(k, i) + min(k, size - 1 - i) else: if abs(i - j) <= k: -1 else: 0) ...
before[i + 1] = before[i] * Z(x - i)
```

Proposal: one rule in both prototypes and in `decisions.md`: an `Int`
next to a `Mod<P>` in `+ - * /` and in assignment is reduced into it, and
a `Mod<P>` never turns back into an `Int` without `.v`. 5 of 10
(canada21p5, noi21p2, noi07p5, sdoi17r1p1, shoi16p3).

## N6. `sum` over `Mod` values does not run (3 of 10: canada21p5, noi21p2, shoi16p3)

*Checked with `proto/cpi`:*

```cp
fn mul(a: Mat, b: Mat) -> Mat = [Off, Off] (i, k) => sum(a[i, j] * b[j, k] for j in Off)
```

```
no `+` for 0 and 4
```

`sum` starts from the `Int` 0. So the matrix product is three nested loops
(N1), and shoi16p3's transition, which is one sum over `j`, is:

```cp
var next = List.filled(m, Z(0))
for keep in 0..=below:
    for j in keep..m: next[keep] += still[j] * comb.choose(j, keep) * comb.choose(m - 1 - j, below - keep) * ways
still = next
```

Proposal: `sum` and `product` start from `T.zero` and `T.one` of the
element type (`Mod` already declares both), in both prototypes. With N5
this is the single change that lets a counting recurrence be written as
its formula.

Also checked: `count(...)` is `unknown name count` in `proto/cpi` and
works in `proto/cpc`; canada21p5 writes `sum(1 for i in 0..n if s[i] != t[i])`.

## N7. A bounded value is not a number in the interpreter, and its type is misread by the compiler (bug; 2 of 10: noi18p4, noip21p2)

*Checked with `proto/cpi`:* with `let life: [Dragon](1..=1e12) = read()`
and `let reward: [Dragon](1..=1e6) = read()`,

```cp
at_least = max(at_least, ceil_div(life[d], atk))    // stdlib/math.cp:27: cannot negate (29)
at_least = max(at_least, (life[d] + atk - 1) / atk) // no `+` for (29) and (12)
```

The port reads plain `[Dragon]Int` and the bounds moved into comments.

*Checked with `proto/cpc`:*

```cp
let v: [0..=m](1..=P - 1) = read()
```

```
expected Mod<998244353>, got (f0: Int[1..=998244352])
```

The parenthesised range is taken as a record with one field. Without the
parentheses, `[0..=m]1..=P - 1`, it builds. `1..<P` is `expected an
expression (at '<')`.

Proposal: a value read with a range type is an `Int` everywhere it is
used, in both prototypes; `(lo..=hi)` as an element type is the range, not
a record; and the docs show one spelling for "an array of bounded
numbers".

## N8. Every multiple of i (3 of 10: noip21p1, sdoi17r1p3, sdoi17r1p1)

Today:

```cp
var j = i
while j <= LIMIT:
    skipped[j] = true
    j += i
```

*Checked:* `for j in (2 * i..=m).step(i)` is ``Range<Int> has no method
`step` `` in `proto/cpc` and ``3..11 has no method `step` `` in
`proto/cpi`. Wanted:

```cp
for j in (i..=LIMIT).step(i): skipped[j] = true
```

Proposal: `step(k)` on every range, for `Int` and for domains. A `for`
loop of the form `for (j = ...; j <= n; j += i)` is in 16 of the 710
corpus problems; every sieve, harmonic sum and divisor table has one.

## N9. The sieve knows smallest prime factors and nothing built from them (2 of 10: sdoi17r1p1, sdoi17r1p3)

Today, in sdoi17r1p1:

```cp
let sieve = Sieve(top)
var mu = List.filled(top + 1, 1)
for x in 2..=top:
    let p = sieve.factor(x)[0].0
    mu[x] = if x / p % p == 0: 0 else: -mu[x / p]
```

`spf` is private, so the smallest prime factor costs a whole
factorization. Wanted: `let mu = sieve.mobius()`.

Proposal, all in `std.math`: `sieve.spf(x)` public;
`sieve.multiplicative(at: (p, e) => ...)` gives the table of any
multiplicative function from its value at prime powers, and `mobius()`,
`phi()` and `divisor_count()` are three one-line uses of it. sdoi17r1p3
could not use `Sieve` at all because it builds with `proto/cpc` (batch 8
H3) and wrote its own six lines.

noip21p1 is a different sieve with the same loop: mark the multiples of
every number with a digit 7. N8 covers it; no `Sieve` method is needed.

## N10. Interpolation and convolution over `Mod` (2 of 10: shoi16p3, sdoi17r1p3)

shoi16p3 needs "the value at u of the degree-m polynomial through these
m + 2 points". `std.spec.poly_at` does this for `Int` only (it must be
exact, so it cannot be used at u = 1e9), so the port has its own:

```cp
fn lagrange_at(ys: List<Z>, x: Int) -> Z:
    let d = ys.len - 1
    if x <= d: return ys[x]
    var before = List.filled(d + 2, Z(1))    // (x - 0) * ... * (x - (i - 1))
    for i in 0..=d: before[i + 1] = before[i] * Z(x - i)
    var (after, total) = (Z(1), Z(0))        // (x - (i + 1)) * ... * (x - d)
    for i in (0..=d).rev:
        let term = ys[i] * before[i] * after * comb.inv_fact(i) * comb.inv_fact(d - i)
        total += if (d - i) % 2 == 0: term else: -term
        after *= Z(x - i)
    total
```

Wanted: `poly_at(ys, u)` with `ys: List<Mod<P>>`.

sdoi17r1p3 multiplies two vectors with indices added modulo k:

```cp
for i in R:
    for j in R: c[R((i.idx + j.idx) % k)] += a[i] * b[j]
```

Proposal: `std.poly` with `poly_at` for `List<Mod<P>>` in O(n) for points
0, 1, 2, ..., `convolve(a, b)` and `convolve(a, b, wrap: k)` (quadratic for
short inputs, NTT when `P` allows), and a `Poly<T>` value with `+ - *`
and evaluation built on them. `std.spec.berlekamp_massey` already has a
private "product reduced by a recurrence" that is the third copy of this
loop. There is no NTT problem in this batch, so the fast path is untested
here.

## N11. A congruence is two loose numbers (noi18p4)

Today. A linear congruence is solved by a local function:

```cp
fn solve_linear(a: Int, b: Int, m: Int) -> ?(Int, Int):
    let (a, b) = (a % m, b % m)
    let g = gcd(a, m)
    if b % g != 0: return none
    let step = m / g
    (mul_mod(b / g, ext_gcd(a / g, step).x % step, step), step)
```

and folding many of them through `crt`, which takes four numbers and
returns an optional pair, is:

```cp
let one = solve_linear(atk, life[d], regrow[d])
all = if all == none or one == none: none else: crt(all!.0, all!.1, one!.0, one!.1)
...
print(r + md * max(0, ceil_div(at_least - r, md)))
```

Wanted:

```cp
var all = Congruence.any
for d in Dragon:
    ...
    all = all.and(Congruence.solve(atk, life[d], regrow[d]))?
print(all.first(at_least: at_least))
```

Proposal: `struct Congruence(r: Int, m: Int)` in `std.math` with
`Congruence.solve(a, b, m) -> ?Congruence`, `c.and(d) -> ?Congruence`
(today's `crt`), `c.first(at_least: x)`, and `Congruence.any` for
"x = 0 mod 1". The original has about 60 lines for the same four operations.

The fold also shows a general gap. *Checked with `proto/cpi`:*
`all = if let (r, md) = all: ... else: none` is `expected an expression
(at 'let')`: `if let` is a statement only, so "combine two optionals"
needs `== none` tests and four `!`. Proposal: `if let` as an expression,
or `a.and_then(x => ...)` on `?T`.

## N12. Quotient blocks (sdoi17r1p1)

Today:

```cp
var (answer, l) = (Z(1), 1)
while l <= min(a.n, a.m):
    let r = min(a.n / (a.n / l), a.m / (a.m / l))     // last T with the same two quotients
    answer *= (upto[r] * undo[l - 1]).pow((a.n / l) * (a.m / l))
    l = r + 1
```

The original has about 30 lines (`solve1` and `part`) for this loop. The formula
for `r` is the line everyone gets wrong once. Wanted:

```cp
for b in quotient_blocks(a.n, a.m):
    answer *= (upto[b.last] * undo[b.first - 1]).pow(b.q.0 * b.q.1)
```

Proposal: `quotient_blocks(n...) yields (first, last, q)` in `std.math`:
the maximal ranges of `T` on which every `n / T` is constant, for one or
more numbers. 1 of 10.

The same port keeps `upto` (prefix products) and `undo` (their inverses)
to divide two prefixes. `Fenwick` takes a `Group`; a plain prefix table
that takes one, `Prefix(g, xs).fold(l..=r)`, would hold both lists. `Prod`
is a `Monoid` today, so it needs a `ProdGroup<T>` for fields.

## N13. `var (a, b, c) = ...` gives names that cannot be assigned (bug; sdoi17r1p3)

*Checked with `proto/cpc`:*

```cp
var (base, acc, t) = (x, x, e - 1)
while t > 0:
    if t % 2 == 1: acc = conv(acc, base)
```

```
line 18: `acc` is not a var
```

`proto/cpi` accepts it, and `stdlib/mod.cp` is written this way. The port
uses three `var` lines. Proposal: `var` with a tuple pattern makes every
name a `var` in `proto/cpc`.

## N14. Tables with two indices are spelled three ways (3 of 10)

*Checked with `proto/cpc`:* `var binom: [0..=n][0..=n]Z = 0` is
`expected [range]Mod<998244353>, got Int[0..=0]`, and `[0..=n, 0..=n]Z = 0`
with `binom[a, b]` builds. *Checked with `proto/cpi`:* `[0..m]Int` is
`` `Int` is not an index type ``, so every count needs a named `domain`
(noi18p4 declares `domain Sword = 0..m` to read one line), and a filled
table is `[Off, Off] (i, j) => Z(0)` because `= 0` does not fill there.
`let (n, m, t, k): (Int, Int, Int, Int) = read()` is `unknown name read`
in `proto/cpc`.

Proposal: `[A, B]T` is the one spelling; `= x` fills it in both
prototypes; anonymous count domains work in the interpreter; a tuple
`read()` works in the compiler.

## N15. A state that is a pair (noi20p1; batch 7 "a pair of domains")

Today:

```cp
domain State = 0..5 * n                      // (city, days still to wait)
fn at(c: City, wait: Int) -> State = State(c.to_idx * 5 + wait)
```

The first version had `(c.to_idx - 1) * 5`, because `City` is `1..=n` and
`to_idx` is the position. It failed at run time with `-5 is not in domain
State (0..=14)`. Wanted: `domain State = (City, 0..5)` and
`State(c, wait)`, with the matrix of N1 indexed by it. This is the product
domain batch 7 asked for under D2, seen here as a matrix index.

## N16. Reading a list that will change (noi18p4)

```cp
domain Sword = 0..m
let start: [Sword]Int = read()
var swords = sorted([s for s in start])
```

Wanted: `var swords: List<Int> = read(m)`. A fixed array is the wrong
type for input that is then edited. 1 of 10.

## Seen again from earlier lists

- **Batch 7 D1** (no ordered bag): noi18p4 needs "the largest sword not
  above x, else the smallest", remove it, insert another. The port
  rebuilds a sorted list per dragon, which is O(n^2); the original uses a
  `multiset`.
- **Batch 8 H3** (the standard library in `proto/cpc`): *checked on
  `fe7ca6d`:* `use std.mod.{Mod, Comb}` and `use std.math.{Sieve}` are
  both "library code the prototype compiler has no stand-in for yet (it
  runs on proto/cpi)". So noip21p2 builds Pascal's triangle by hand
  instead of `Comb`, sdoi17r1p3 its own sieve, and 7 of 10 were not built
  by `proto/cpc` at all.
- **Batch 8 H1** (a bound that names an earlier input): noip21p2 reads
  `let k: 1..=30 = read() where k <= n`, and 72 of 200 generated cases
  were thrown away.
- **Batch 8 H12** (`popcount` missing in `proto/cpc`): noip21p2 writes
  `sum(1 for b in 0..6 if (x >> b) % 2 == 1)`.
- **Batch 3 G13** (`power`): N3.
- **Batch 8 H7**: `swap(a[pivot], a[c])` on two rows of a list of lists
  works in `proto/cpi` (N4).

## Already good

- `std.spec.guess_at` turned noi07p5 from 182 lines into 37. The port
  states Kirchhoff's theorem for small n and asks for term n - k; the
  original's bitmask DP, Berlekamp-Massey and matrix power are not needed.
- `Mod<P>` with `P = 65521` or `20170408` (not prime) is the same type as
  the usual two, and `.inv` works for both since it uses the extended
  Euclidean algorithm. Three originals carry a `mint` struct of about 60 lines.
- `Comb<P>.choose` returns 0 outside `0..=n`, so canada21p5 and shoi16p3
  need no `choose1` wrapper and no bounds tests in their sums.
- `crt` in `std.math` handles moduli that are not coprime and stays inside
  an `Int` by using `mul_mod`; the original redefines `long` as a 128-bit
  integer to multiply its residues.
- Euclidean `%`: `x - i` reduced into `Z` and `(carry + c) / 2` need no
  fix-up for negatives.
- `@memo fn ways(left, bit, carry, fixed)` is the whole of noip21p2: no
  table declaration, no loop order, no base-case fill over a 4-D array.
- `if let j = Off.try(i.to_idx - c + (m - c))` in canada21p5 replaces a
  parity test and two bounds tests on the transition's target.
