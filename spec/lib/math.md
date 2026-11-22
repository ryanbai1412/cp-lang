# Standard library design: number theory, combinatorics, NTT and linear algebra

> **0.3.0 status.** This chapter is the family's design survey and API.
> Where it disagrees with the core chapters or the aligned rounds, they
> win: `core/03-numbers-layout.md` (four number types, no `I32`/`U32`,
> `Mod<P>` not `Mint`), `core/05-ownership.md` (constructors own their
> results), `core/07-sequences.md` (capabilities, views keep keys),
> `lib/ops-ranges.md` (`Op`/`Monoid`/laws, `each(...)`, no
> `RangeFenwick`, `MinHeap`/`MaxHeap` as `Heap(by:)`) and
> `lib/sequence-patterns.md`. The stubs in `std/` are the signatures of
> record. Reconciliation of this text is tracked in `delta/`.


This document designs the five mathematical modules of the CP Lang standard
library: `std.math` (integer number theory and sieves), `std.mod` (modular
arithmetic and combinatorics), `std.ntt` (number-theoretic and bitwise
convolutions), `std.poly` (formal power series over NTT), and `std.matrix`
(dense matrices, Gaussian elimination and GF(2) linear bases). It corresponds
to sections 4.3 through 4.6 of `plan/STDLIB.md`.

Every design choice follows `AGENTS.md` and `plan/STDLIB.md`: ordinary CP Lang
with no compiler privilege, guessable textbook names, options as named
arguments, documented time and memory cost on every public item, explicit
representation, and algebraic laws checked by `@test`.

## 1. Survey of contest patterns

A survey of Codeforces Div. 1/2, AtCoder ABC/ARC/AGC/WTF, ICPC World Finals,
KACTL (`number-theory/`, `numerical/`), AtCoder Library (`math.hpp`,
`modint.hpp`, `convolution.hpp`), and the 60+ math-bearing accepted
submissions in `proto/ports/ecnerwala/*_orig.cpp` shows seven recurring
contest shapes.

First, **basic integer arithmetic and divisibility**: `gcd`, `lcm`, Bezout
coefficients (`ext_gcd`), Euclidean and ceiling division, modular inverses
for dynamic non-type moduli (`inv_mod`), exact binomials (`choose`),
`choose2(n)` for pair counts, and `floor_sum` for lattice-point and
arithmetic-progression sums. In `proto/ports/ecnerwala/`, `gcd`/`ext_gcd`
appear in 11 submissions and `floor_sum` is a standard AtCoder Library
building block.

Second, **primality, sieves, factorization and multiplicative functions**:
three distinct cost regimes appear in contests. For many queries up to
`n <= 2e7`, a linear sieve (`Sieve(n)`) gives `O(1)` primality, `O(log x)`
factorization, and `O(d(x))` divisor enumeration, plus `O(n)` tables of any
multiplicative function (`phi`, `mobius`, divisor sums, or custom recurrences
like `cf2135e2`'s `odd_div_sum`). For a single number up to `1e12`, trial
division (`factor`, `divisors`, `phi`) is `O(sqrt x)` with no setup. For
numbers up to `2^62`, deterministic 12-base Miller-Rabin (`is_prime`) and
Pollard's rho with Brent batching (`factor` above `10^6`) handle arbitrary 64-bit
inputs. Rounding out elementary number theory are `crt` / `crt_all` (general
non-coprime moduli), `primitive_root`, `mod_log` (BSGS discrete logarithm),
and `mod_sqrt` (Tonelli-Shanks).

Third, **modular arithmetic with compile-time and runtime moduli**: `modnum`
appears in 60 ecnerwala submissions. The modulus is usually a prime constant
(`998_244_353` or `1_000_000_007`), sometimes a composite or runtime-read
integer, and occasionally a 60-bit prime for hashing. One type `Mod<P>`
serves all of them: the modulus is part of the type, takes zero storage per
value, and prevents mixing two moduli.

Fourth, **combinatorial counting tables tied to `Mod<P>`**: factorial and
inverse-factorial tables appear in 25 ecnerwala submissions (`cf2066d2`,
`cf2084e`, `cf2135e2`, `cf2150g`, etc.). Every such solution wants `fact`,
`inv_fact`, `inv(i)` in `O(1)`, `choose(n, k)` (with `0` outside `0..=n` and
Lucas's theorem when `n >= P`), `arrange`, `multichoose`, `multinomial`, and
`catalan`, without repeating the modulus literal after `type Z = Mod<P>`.

Fifth, **polynomial and bitwise convolutions**: NTT appears in 7 ecnerwala
submissions and 9 corpus ports (`lib.math.poly`). Contests need four closely
related operations: in-place `ntt`, polynomial multiplication over an
NTT-friendly `Mod<P>`, arbitrary-modulus multiplication via 3-prime CRT,
and exact signed integer convolution (requested by
`research/frictions/cf2201e.md`), all one `convolve` whose value type picks
the algorithm. Both linear
and cyclic convolutions (`cyclic: true`) occur regularly. Alongside NTT,
bitwise convolutions (`convolve(a, b, on: |)`, `on: &`, `on: ^`) and
subset/superset zeta-Mobius transforms (one `sos(a, over: Sub | Sup,
inverse:)`) cover `O(n 2^n)` bitmask DP.

Sixth, **dense matrices and Gaussian elimination**: linear recurrences and
walk counting (`Mat.pow`, `Mat.apply`), matrix-DP segment trees
(`Mat.product`), and linear algebra over prime fields `Mod<P>` and reals
`F64` (`det`, `rank`, `solve`, `solve_space`, `inverse`). Flat row-major
storage matters for cache locality; elimination is exact over `Mod<P>` and
uses partial pivoting with a configurable tolerance `eps` over `F64`.

Seventh, **GF(2) linear bases**: XOR subset maximization, membership,
k-th smallest XOR, and merging bases inside segment trees or DSU-on-tree
(`cf2115d`, `cf2262e`, `ucup5s05f`, CF 1100F). A compact `XorBasis` type
with a `Monoid` combinator eliminates a recurring 20-line template.

## 2. Coherent API surface

### 2.1 `std.math`: integer helpers, number theory and sieves

Every function documents its cost and its regime. Where two algorithms exist
for the same mathematical operation, the cost is visible in the name or type:
`Sieve(n).factor(x)` is `O(log x)` after `O(n)` setup, and standalone
`factor(x)` is `O(sqrt x)` for `x <= 1e6` and `O(x^(1/4) log x)` expected above.

```cp
// Divisibility, division, and exact combinatorics
fn gcd(a: Int, b: Int) -> Int                       // O(log min(|a|, |b|)); >= 0, gcd(0,0) = 0
fn lcm(a: Int, b: Int) -> Int                       // O(log min(|a|, |b|)); >= 0, traps on overflow
fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int)
                                                    // O(log min(|a|, |b|)); a*x + b*y == g >= 0
fn ceil_div(a: Int, b: Int) -> Int                  // O(1); b != 0
fn divmod(self: Int, d: Int) -> (Int, Int)          // O(1); Euclidean (self / d, self % d), 0 <= r < |d|, d != 0
fn fixed(num: Int, den: Int, k: Int) -> Str         // O(k); exact integer decimal formatting
fn choose(n: Int, k: Int) -> Int                    // O(min(k, n - k)); 0 outside 0..=n
fn choose2(n: Int) -> Int                           // O(1); n * (n - 1) / 2, 0 for n < 2

// Bit and digit helpers
fn bit_len(x: Int) -> Int                           // O(1); 0 for x == 0
fn digits(self: Int, base: Int = 10) -> List<Int>   // O(log_base x); lowest digit first
@get fn sign(self: Int) -> Sign                     // O(1); Neg, Zero or Pos
fn bit(self: Int, i: Int) -> Bool                   // O(1); bit 0 is lowest
@get fn is_pow2(self: Int) -> Bool                  // O(1); x > 0 and x & (x - 1) == 0
fn splitmix64(x: U64) -> U64                        // O(1); 64-bit mixing finalizer

// Modular integer arithmetic (dynamic modulus without Mod<P>)
fn mul_mod(a: Int, b: Int, m: Int) -> Int           // O(1); exact via I128 when needed, m >= 1
fn pow_mod(a: Int, e: Int, m: Int) -> Int           // O(log e); e >= 0, m >= 1
fn inv_mod(a: Int, m: Int) -> ?Int                  // O(log m); m >= 1, none if gcd(a, m) != 1
fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int)
                                                    // O(log min(m1, m2)); non-coprime allowed
fn crt_all(eqs: List<(Int, Int)>) -> ?(Int, Int)
                                                    // O(k log M); folds crt over a system
fn mod_log(a: Int, b: Int, m: Int) -> ?Int          // O(sqrt m); smallest x >= 0: a^x = b (mod m)
fn mod_sqrt(a: Int, p: Int) -> ?Int                 // O(log^2 p); prime p, x^2 = a (mod p)
fn primitive_root(p: Int) -> Int                    // expected O(p^(1/4) log p); prime p
fn floor_sum(n: Int, m: Int, a: Int, b: Int) -> Int // O(log m); sum_{i in 0..n} floor((a*i+b)/m)

// Primality and factorization (single-input)
fn is_prime(x: Int) -> Bool                         // O(log^2 x); deterministic Miller-Rabin, all Int
fn factor(x: Int) -> List<(Int, Int)>               // O(sqrt x) <= 1e6, O(x^(1/4) log x) expected above
fn divisors(x: Int) -> List<Int>                    // O(sqrt x); sorted
fn divisors_from_factors(fs: List<(Int, Int)>) -> List<Int>
                                                    // O(d(x) log d(x)); sorted divisors from factors
fn phi(x: Int) -> Int                               // O(sqrt x); Euler's totient

// Linear sieve and multiplicative tables
struct Sieve:                                       // O(n) time and memory
    spf: List<0..=2_000_000_000>                    // smallest prime factor, 0 for 0 and 1
    primes: List<0..=2_000_000_000>                 // all primes in 2..=n, ascending
    fn new(n: Int) -> Sieve                         // O(n)
    fn is_prime(self, x: Int) -> Bool               // O(1)
    fn factor(self, x: Int) -> List<(Int, Int)>     // O(log x)
    fn divisors(self, x: Int) -> yields Int         // O(1) per divisor; unordered
    fn phi(self, x: Int) -> Int                     // O(log x)
    fn mobius(self, x: Int) -> Int                  // O(log x)
    fn phi_table(self) -> List<Int>                 // O(n); totient[0..=n]
    fn mobius_table(self) -> List<Int>              // O(n); mu[0..=n]

fn sieve_multiplicative<T>(
    n: Int,
    prime_pow: (Int, Int, Int) -> T,
) -> List<T>
    // O(n) time and memory; computes f[0..=n] for any multiplicative f
    // with f(1) = T.one, f(a * b) = f(a) * f(b) for gcd(a, b) = 1,
    // and f(p^k) = prime_pow(p, k, p^k) for every prime power p^k <= n.
```

`sieve_multiplicative` takes only `prime_pow`; `T.one` and `*` come from the
value type, as for every other algebraic default, so `Int`, `Mod<P>` and user
structs with `One` and `Mul` all work. Two statistics at once use a record
type with its own `*`. It runs in strict `O(n)` time by tracking the largest
prime-power divisor `low[i]` of each `i`.

### 2.2 `std.mod`: modular arithmetic and combinatorics

`Mod<P>` is one type for both compile-time constants (`type Z = Mod<998_244_353>`)
and runtime moduli (`let P: Int = read(); type Z = Mod<P>`). `Comb<M>` is keyed
on the `Mod` type (`let comb = Comb<Z>(limit)`) so the modulus is written once.

```cp
struct Mod<P>(v: 0..P):
    let MOD = P
    let zero = Mod(0)
    let one = Mod(1 % P)
    fn new(x: Int) -> Mod<P>                        // O(1); Euclidean reduction into 0..P
    // x ** e via `impl Pow`: O(log |e|); negative e uses inv
    fn try_inv(self) -> ?Mod<P>                     // O(log P); none if gcd(v, P) != 1
    fn inv(self) -> Mod<P>                          // O(log P); traps if non-invertible
    // also: from_int, sqrt, log, inv_all; Comb.derange, Barrett, PrimePowerComb,
    // ModComb (see the std.mod stub)
    // impls: Add, Sub, Neg, Mul, Div, Pow, Show

struct Comb<M>:
    fn new(limit: Int = 0) -> Comb<M>               // O(limit) time and memory; exact size
    fn fact(self, n: Int) -> M                      // O(1) amortized; 0 for n >= M.MOD
    fn inv_fact(self, n: Int) -> M                  // O(1) amortized; 0 <= n < M.MOD
    fn inv(self, i: Int) -> M                       // O(1) amortized; 1 <= i < M.MOD
    fn choose(self, n: Int, k: Int) -> M            // O(1) for n < M.MOD; Lucas O(log_P n) for n >= M.MOD
    fn arrange(self, n: Int, k: Int) -> M           // O(1) amortized; 0 outside 0..=n
    fn multichoose(self, n: Int, k: Int) -> M       // O(1) amortized; stars and bars
    fn multinomial(self, ks: List<Int>) -> M        // O(ks.len) amortized; (sum ks)! / prod(k!)
    fn catalan(self, n: Int) -> M                   // O(1) amortized
```

Performance notes from the corpus (`cf2135e2`, `cf2150g`, `cf2066d2`,
`cf2062h`, `cf2084e`):

1. `Comb.new(limit)` sizes `f` and `g` to `limit + 1` directly and fills `g`
   in place without allocating a temporary `tail` buffer. Lazy growth via
   `ensure(n)` still doubles capacity (clamped at `P`), also filling `g` in
   place. This cuts peak memory on `n = 2e7` by ~350 MB.
2. `choose(n, k)` and `arrange(n, k)` take a single fast-path branch
   `if n < f.len` before checking `n >= P` or calling `ensure`, because
   `f.len <= P` is an invariant of `Comb<P>`.
3. Storage width: today `Mod<P>(v: Int)` uses 8 bytes per value because
   type-parameter range narrowing (`v: 0..P`) is not yet in `cpl-check`/
   `cpl-codegen`. That is the single largest remaining performance gap on
   dense `Mod<P>` tables (`cf2150g`, `cf2135e2`, `cf2062h`). Narrowing `0..P`
   to `uint32_t` when `P <= 2^31 - 1` is tracked for the core/codegen lane
   and requires no API change in `std.mod`.

### 2.3 `std.ntt`: NTT and bitwise convolutions

`std.ntt` promotes the radix-4 Montgomery NTT engine from
`examples/local-lib/math/poly.cp` into a clean, generic stdlib module and
adds cyclic convolution and bitwise transforms.

```cp
// Number-theoretic transform and polynomial convolution
fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false, bit_reversed: Bool = false)
    // O(n log n) in place; a.len a power of 2 dividing P - 1.
    // Uses radix-4 Montgomery butterflies for standard primes (998244353,
    // 469762049, 167772161, 754974721) and primitive_root(P) for any other
    // NTT-friendly prime P.

fn convolve<T, O, R>(
    a: List<T>,
    b: List<T>,
    on: O = (x, y) => x + y,      // ^ | & subset gcd lcm
    over: R = Semiring(+, *),     // Semiring(min, +) needs shape:
    shape: ?Curved = None,        // Both | Right | Neither
    cyclic: Bool = false,
) -> List<T>
    // The value type picks the algorithm: NTT for an NTT-prime Mod<P>, three
    // primes and CRT for any other Mod (runtime moduli too) or Int (exact when
    // every output fits), FFT for F64. Bitwise `on:` needs equal powers of two.

// Subset/superset zeta-Mobius transforms in place (n = 2^k)
enum SosOver = Sub | Sup
fn sos<S: IndexedMut>(mut a: S, over: SosOver = Sub, inverse: Bool = false)
```

Notice the uniform design: one `convolve` (ops-ranges §13), with `cyclic:`,
`on:` and `over:` as named arguments. All bitwise transforms infer the number
of bits from `a.len` (asserting `a.len.is_pow2`), avoiding a redundant `bits`
argument that can go out of sync with `a.len`.

### 2.4 `std.matrix`: dense matrices, Gaussian elimination and `XorBasis`

> The names of record are `std/matrix.cp`'s: `Mat.zeros(n, m)`, `Mat.from_rows(rows)`,
> `rows`/`cols`/`data`, `impl Pow`, and `A.mul_vec(x)` for `A.apply(x)`; there is no
> `impl Neg`. `XorBasis` holds 63-bit non-negative values; `new` and `add` are O(W).

`Mat<T>` stores entries in a single flat row-major `List<T>` of length
`n * m` so multiplication and elimination stay cache-friendly.

```cp
struct Mat<T>(n: Int, m: Int, a: List<T>):
    fn zero(n: Int, m: Int = n) -> Mat<T>           // O(n m)
    fn identity(n: Int) -> Mat<T>                   // O(n^2)
    fn of(rows: List<List<T>>) -> Mat<T>            // O(n m)
    fn get(self, i: Int, j: Int) -> T               // O(1)
    fn at(self, i: Int, j: Int) lends mut T         // O(1); writable cell: m.at(i, j) += x
    fn pow(self, e: Int) -> Mat<T>                  // O(n^3 log e); square matrix, e >= 0
    fn apply(self, v: List<T>) -> List<T>           // O(n m); A * v
    fn transpose(self) -> Mat<T>                    // O(n m)
    fn rows(self) -> List<List<T>>                  // O(n m)
    fn product(n: Int) -> Monoid<Mat<T>>            // O(n^2); monoid for SegTree<D, Mat<T>>
    // impls: Add, Sub, Neg, Mul (ikj order, skipping zero entries)

    // Exact Gaussian elimination over any exact field (e.g. Mod<P> with prime P):
    fn det(self) -> T                               // O(n^3); square matrix
    fn rank(self) -> Int                            // O(n m min(n, m))
    fn solve(self, b: List<T>) -> ?List<T>          // O(n m min(n, m)); one solution or none
    fn solve_space(self, b: List<T>) -> ?(x0: List<T>, basis: List<List<T>>)
                                                    // O(n m min(n, m)); particular + nullspace
    fn inverse(self) -> ?Mat<T>                     // O(n^3); square matrix, none if singular

// Floating-point elimination with partial pivoting (F64):
fn det_real(a: Mat<F64>, eps: F64 = 1e-9) -> F64    // O(n^3)
fn rank_real(a: Mat<F64>, eps: F64 = 1e-9) -> Int   // O(n m min(n, m))
fn solve_real(a: Mat<F64>, b: List<F64>, eps: F64 = 1e-9) -> ?List<F64>
                                                    // O(n m min(n, m))
fn inverse_real(a: Mat<F64>, eps: F64 = 1e-9) -> ?Mat<F64>
                                                    // O(n^3)

// GF(2) linear basis over 63-bit non-negative integers (x >= 0)
struct XorBasis:
    fn new() -> XorBasis                            // O(W)
    @get fn rank(self) -> Int                       // O(1); dimension of span
    fn add(mut self, x: Int) -> Bool                // O(log x); true if independent
    fn reduce(self, x: Int) -> Int                  // O(62); min x ^ s over s in span
    fn contains(self, x: Int) -> Bool               // O(62); reduce(x) == 0
    fn max_xor(self, init: Int = 0) -> Int          // O(62); max init ^ s over s in span
    fn min_xor(self, init: Int) -> Int              // O(62); alias for reduce(init)
    fn kth(self, k: Int) -> ?Int                    // O(62^2); k-th smallest in span (0-indexed)
    fn merge(mut self, o: XorBasis)                 // O(rank * 62); union of spans
    fn monoid() -> Monoid<XorBasis>                 // O(1); for SegTree<D, XorBasis>
```

Why `det_real` / `solve_real` are separate from `Mat<T>.det` / `Mat<T>.solve`:
floating-point elimination needs partial pivoting by `abs` and a tolerance
`eps`, whereas exact elimination over `Mod<P>` checks `== T.zero` and divides
directly. Keeping exact field methods on `Mat<T>` and pivoted real helpers
explicit avoids hidden `F64` specialization and respects CP Lang's rule that
float equality is rejected by default.

### 2.5 `std.poly`: formal power series over NTT

`std.poly` operates directly on `List<Mod<P>>` (lowest degree first, identical
to `std.ntt.convolve<P>`), with no wrapper struct so series compose directly
with `std.ntt`, `Comb<Mod<P>>`, and list comprehensions. Every truncated FPS
operation takes an explicit output length `n: Int` (computing modulo `x^n`,
returning a `List<Mod<P>>` of length `n`, with `n == 0` returning `[]`) and
requires an NTT-friendly prime `P` (`P - 1` divisible by a power of two at
least `2 * bit_ceil(n)`).

```cp
fn deriv<P>(a: List<Mod<P>>) -> List<Mod<P>>
    // O(a.len) time and memory; formal derivative a'(x), length max(0, a.len - 1)

fn integral<P>(a: List<Mod<P>>) -> List<Mod<P>>
    // O(a.len) time and memory; antiderivative with constant term 0, length a.len + 1
    // (or [] if a is empty); uses linear-time modular inverses 1..=a.len, no O(log P) div

fn inv<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>
    // O(n log n) time (~10n log n via half-size Newton doubling: 5 NTTs of size 2m per
    // doubling step m -> 2m), O(n) memory; returns a(x)^(-1) mod x^n.
    // Precondition: n >= 0; if n > 0, a.len > 0 and a[0] != 0.

fn log<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>
    // O(n log n) time (1 inv + 1 convolve + deriv + integral, ~13n log n), O(n) memory;
    // returns ln(a(x)) mod x^n = integral(a'(x) * a(x)^(-1)) mod x^n.
    // Precondition: n >= 0; if n > 0, a.len > 0 and a[0] == 1.

fn exp<P>(a: List<Mod<P>>, n: Int) -> List<Mod<P>>
    // O(n log n) time (Newton doubling g <- g * (1 - ln(g) + a) mod x^(2m), ~28n log n),
    // O(n) memory; returns exp(a(x)) mod x^n.
    // Precondition: n >= 0; if n > 0, a.len == 0 or a[0] == 0.

fn pow<P>(a: List<Mod<P>>, k: Int, n: Int) -> List<Mod<P>>
    // O(n log n) time (1 log + 1 exp when nonzero, O(n) when k == 0 or shifted past n),
    // O(n) memory; returns a(x)^k mod x^n for k >= 0, handling leading zeros a(x) = c x^d (...).
    // Precondition: n >= 0, k >= 0.

fn sqrt<P>(a: List<Mod<P>>, n: Int) -> ?List<Mod<P>>
    // O(n log n + log^2 P) time (Tonelli-Shanks mod_sqrt for leading term + Newton doubling
    // g <- (g + a * g^(-1)) / 2 mod x^(2m), ~20n log n), O(n) memory.
    // Returns some(g) with g(x)^2 == a(x) (mod x^n), or none if no square root exists mod P
    // (odd leading zero count or non-residue leading coefficient).
    // Precondition: n >= 0.
```

Cost summary (NTTs of length `N = bit_ceil(n)`):
- `deriv`, `integral`: `O(n)` time, `0` NTTs (`integral` computes `inv[1..=n]` in `O(n)` via `inv[i] = -(P / i) * inv[P % i]`).
- `inv`: ~`5` NTTs of size `2m` per doubling (`m = 1, 2, ..., N/2`), geometric total ~`10` NTTs of size `N`.
- `log`: `inv(a, n)` + 1 `convolve` of length `n` + `deriv`/`integral`.
- `exp`: Newton doubling calling `log` + 1 `convolve` at each doubling `m -> 2m`.
- `pow`: `O(n)` normalization + 1 `log` + 1 `exp` + `O(n)` rescaling (no `log k` factor).
- `sqrt`: 1 `mod_sqrt` (`O(log^2 P)`) + Newton doubling calling `inv` + 1 `convolve` at each doubling.

### 2.6 Everyday integer helpers (all in the prelude)

| Form | Meaning | Cost |
|---|---|---|
| `ceil_div(a, b)`, `x.divmod(d)` | ceiling division; Euclidean `(q, r)` with `0 <= r < abs(d)` | O(1) |
| `x.digits(base:)`, `x.to_str(base:)` | digits lowest first; the written form, highest first | O(log x) |
| `x.sign` | the side of zero, a `Sign` | O(1) |
| `floor_quotients(n, from: 1)` | each distinct `n / i` for `i >= from`, with its run of `i`, `(q, lo, hi)` | O(sqrt n) |
| `Ratio(p, q)` | an exact reduced rational | O(log) per op |
| `coef.eval(x)` | a polynomial at `x` over any ring, by Horner | O(len) |
| `Curved` | the `shape:` of a non-`(+, *)` `convolve` (`Both`, `Right`, `Neither`) | — |

## 3. Laws and invariants

Every module enforces its mathematical laws in `@test`:

- `std.math`: `gcd(a, b) >= 0`, `a * e.x + b * e.y == e.g == gcd(a, b)` for
  `e = ext_gcd(a, b)`; `crt(r1, m1, r2, m2)` returns `(r, lcm(m1, m2))` with
  `r % m1 == r1 % m1` and `r % m2 == r2 % m2`, or `None` iff no solution
  exists; `is_prime(x)` agrees with `Sieve` and rejects Carmichael numbers
  and strong pseudoprimes; `factor(x) == s.factor(x)` and
  their product equals `x`; `pow_mod(a, mod_log(a, b, m)!, m) == b % m`;
  `mod_sqrt(a, p)! ** 2 % p == a % p`; `sieve_multiplicative` satisfies
  `f[a * b] == f[a] * f[b]` for coprime `a, b`.
- `std.mod`: `Mod<P>` satisfies `laws_hold(Monoid<Mod<P>>.of(+), xs)` and
  `laws_hold(Monoid<Mod<P>>.of(*), xs)`; `x * x.inv == Mod.one` when
  `gcd(x.v, P) == 1`; `Comb<P>.choose` satisfies Pascal's identity and
  Lucas's theorem; `fact(n) * inv_fact(n) == one`; `inv(i) * Mod(i) == one`.
- `std.ntt`: `ntt(mut a, inverse: true)` is the exact inverse of `ntt(mut a)`;
  `convolve` agrees with the naive `O(n m)` loop for every value type, `on:`
  and `over:`, both `cyclic: false` and `cyclic: true`; `sos(inverse: true)`
  inverts `sos`.
- `std.poly`: `convolve(a, inv(a, n))[0..n] == [1, 0, ...]`; `exp(log(a, n), n) == a[0..n]`
  for `a[0] == 1`; `log(exp(a, n), n) == a[0..n]` for `a[0] == 0`;
  `pow(a, k, n)` agrees with repeated `convolve` (including leading zeros and `k == 0`);
  `convolve(g, g)[0..n] == a[0..n]` for `g = sqrt(a, n)!` and `sqrt(a, n) == None` for
  odd leading zero count or quadratic non-residue leading term; `deriv(integral(a)) == a`.
- `std.matrix`: `Mat<T>.product(n)` satisfies `laws_hold`; `(A * B).det() ==
  A.det() * B.det()`; `A * A.inverse()! == Mat.identity(n)`; for every `x0`
  and `v` in `solve_space(b)!.basis`, `A.mul_vec(x0 + c * v) == b`; `XorBasis`
  satisfies `laws_hold(XorBasis.monoid(), xs)`, `contains(x)` iff `x` is a
  subset XOR of inserted elements, and `kth(0..2^rank)` is strictly
  increasing and exhausts the span.

## 4. Domains, index spaces and what stays user code

`std.math` and `std.mod` operate on mathematical values (`Int`, `Mod<P>`),
not array positions, so their arguments are integers and moduli rather than
domains. Where a matrix or basis lives inside a range structure, it plugs
directly into CP Lang's domain-indexed structures via `Monoid`:
`SegTree<V, Mat<Z>>(Mat<Z>.product(k), init)` and
`SegTree<I, XorBasis>(XorBasis.monoid(), init)`.

What stays in `examples/local-lib/math/` or user code (until wider corpus
demand justifies promotion):

- Advanced polynomial algorithms (multipoint evaluation, interpolation,
  Taylor shift, Bostan-Mori, polynomial GCD, composition) are in `std.poly`
  (see its stub); the rest stays in `examples/local-lib/math/poly.cp`.
- Exact fractions (`Frac`) and homomorphisms stay in `examples/local-lib/math/`;
  `simplex` is in `std.matrix`.
- Custom-payload XOR bases (like `cf2115d`, where each basis bit also owns a
  player `Bool` deciding whether to maximize or minimize that bit) stay user
  code because the payload semantics are problem-specific.
- Arbitrary-precision `BigInt`: out of scope for v1; `I128` covers intermediate
  products.

## 5. Real problems written against this API

Each solution below is written at the bar of `design/EXEMPLARS-PORTS.md`:
the editorial invariant up front, standard algorithms named, no boilerplate.

### 5.1 Codeforces 2201 E — ABBA Counting

Split `t` of length `2m` into two halves. By inclusion-exclusion on the
repetition count `f | m` (period `L = m / f`), each `f` has Mobius-like
weight `prod_{p | f} (1 - p)`. Cyclic exact-integer convolution counts
agreements and doubly-fixed pairs for every shift at once.

```cp
// A valid ABBA string of length 2m has minimal period L | m (f = m / L).
// For each divisor f, fold both halves mod L and use cyclic integer
// convolution to count agreements and doubly-fixed pairs per shift.
@!multitest
use std.math.{divisors, factor}

type Z = Mod<998_244_353>

let n: 2..=400_000 = read()
let t: Str = read()
let m = n / 2

var pow2: [0..=m]Z = 1
for i in 1..=m: pow2[i] = pow2[i - 1] * 2

var ans: Z = 0
'divisor: for f in divisors(m):
    let len = m / f
    var s: [0..2][0..len]?Char = None
    for z in 0..2:
        for k in 0..f:
            for i in 0..len:
                let c = t[z * m + k * len + i]
                if c == '?': continue
                if let prev = s[z][i], prev != c: continue 'divisor
                s[z][i] = c

    var sign0: [0..len]Int = 0
    var sign1: [0..len]Int = 0
    var fix0: [0..len]Int = 0
    var fix1: [0..len]Int = 0
    var tot_fix = 0
    for i in 0..len:
        if let c = s[0][i]:
            sign0[i] = if c == 'a': 1 else: -1
            fix0[i] = 1
            tot_fix += 1
        if let c = s[1][len - 1 - i]:
            sign1[i] = if c == 'a': 1 else: -1
            fix1[i] = 1
            tot_fix += 1

    let diff_conv = convolve(sign0, sign1, cyclic: true)
    let both_conv = convolve(fix0, fix1, cyclic: true)
    var ways: Z = 0
    for i in 0..len:
        if diff_conv[i] == both_conv[i]:
            ways += pow2[len - tot_fix + both_conv[i]]

    var w: Z = 1
    for (p, _) in factor(f): w *= 1 - p
    ans += ways * w

print(ans)
```

Compare with `proto/ports/ecnerwala/cf2201e.cp`: the `cyclic_convolve`
wrapper, the `Mod<998244353>` encoding hack, and the `.v` field leak are all
gone.

### 5.2 Codeforces 2135 E2 — Beyond the Palindrome (Hard Version)

The editorial reduces the answer to a linear combination of binomials
weighted by the multiplicative function `odd_div_sum(x) = 2^{v_2(x)} *
sigma_1(x / 2^{v_2(x)})`.

```cp
// odd_div_sum(x) = 2^{v_2(x)} * sigma_1(x / 2^{v_2(x)}) is multiplicative:
// f(2^k) = 2^k, and for odd prime p, f(p^k) = (p^{k+1} - 1) / (p - 1).
use std.math.{sieve_multiplicative}

type Z = Mod<998_244_353>

let t: 1..=10_000 = read()
let ns: [0..t](1..=20_000_000) = read()
let lim = ns.max() + 1

let comb = Z.comb(lim)
let odd_div_sum = sieve_multiplicative(
    lim,
    prime_pow: (p, _, pe) => if p == 2: pe else: (pe * p - 1) / (p - 1),
)

for n in ns:
    var cnt: [0..=n]Z = 0
    for i in 1..=n + 1:
        var v = Z(odd_div_sum[i])
        if i % 2 == 1: v -= 1
        elif n % 2 == 1 and i % 4 == 2: v -= 2
        if (i + n) % 2 == 1:
            if i - 1 <= n: cnt[(n + i - 1) / 2] += 2 * v
            if i + 1 <= n: cnt[(n + i + 1) / 2] += 2 * v
        else:
            if i <= n: cnt[(n + i) / 2] -= 4 * v

    let ans: Z = sum(comb.inv_fact(i) * comb.inv_fact(n - i) * cnt[i] for i in 0..=n)
    print(ans * comb.fact(n) + Z(2) ** n)
```

Compare with `proto/ports/ecnerwala/cf2135e2.cp`: the 22-line hand-rolled
linear sieve disappears into a single `sieve_multiplicative` call, and
`Z.comb(lim)` writes the modulus once.

### 5.3 AtCoder DP R — Walk

Count directed walks of length `k` in an `n`-vertex graph (`n <= 50`,
`k <= 10^18`) modulo `10^9 + 7`: raise the adjacency matrix to the `k`-th
power and sum all entries.

```cp
// The (i, j) entry of A^k counts length-k walks from i to j.
use std.matrix.{Mat}

type Z = Mod<1_000_000_007>

let n: 1..=50 = read()
let k: 1..=1_000_000_000_000_000_000 = read()
let adj = [[Z(read<0..=1>()) for _ in 0..n] for _ in 0..n]
let walks = Mat<Z>.of(adj) ** k
print(sum(walks.get(i, j) for i in 0..n for j in 0..n))
```

Compare with `proto/ports/dpr.cp`: both hand-rolled matrix multiplication and
binary exponentiation vanish.

### 5.4 Codeforces 1100 F — Ivan and Burgers

Given an array `a[1..=n]` (`n <= 5e5`) and `q` range queries `[l, r]`, print
the maximum XOR of any subset of `a[l..=r]`. A `SegTree` over `XorBasis`
answers each query in `O(log n * 62)` (or offline prefix bases in `O(62)`).

```cp
// XorBasis forms a monoid under merge, so a SegTree answers range max-XOR
// queries directly.
use std.matrix.{XorBasis}

let n: 1..=500_000 = read()
domain I = 1..=n
let a: [I]0..=1_000_000 = read()

let init = [I] i =>
    var b = XorBasis.new()
    b.add(a[i])
    b
let seg = SegTree<I, XorBasis>(XorBasis.monoid(), init)

let q: 1..=500_000 = read()
repeat q:
    let (l, r): (I, I) = read()
    print(seg.fold(l..=r).max_xor())
```

### 5.5 Codeforces 2084 E — Blossom

Count the sum of `MEX` over all subarrays of all completions of a partial
permutation `a`. Using `Z.comb(n)` keeps the modulus in one place.

```cp
@!multitest

type Z = Mod<1_000_000_007>

let n: 1..=5000 = read()
domain Pos = 0..n
domain Val = 0..n
let a: [Pos]Int = read()

var pos: [Val]?Pos = None
for i in Pos if a[i] >= 0: pos[Val(a[i])] = i

let missing = prefix([Pos] i => if a[i] == -1: 1 else: 0)
let m = missing.prefix(Pos.last.r)
let comb = Z.comb(n)
// ... sweep Val, accumulating Z(freq[f]) * comb.arrange(f, need) * comb.fact(m - need)
```

---

## 6. Matched C++17 benchmarks (`bench/math/`)

`bash bench/math/run.sh` benchmarks the six core math workloads against matched
C++17 (same algorithm and layout, `g++ -std=c++17 -O2`, best of 5 runs, exact
stdout check, emitted C++ bytes):

| Workload | Size | CP Lang | C++17 | Ratio | Emitted |
|---|---|---:|---:|---:|---:|
| `modint` | 10M add/sub/mul + 200K div/pow (`Mod<998244353>`) | 0.067s | 0.070s | 0.96x | 8,998 B |
| `ntt` | `convolve` (`262,144 x 262,144`, Montgomery NTT) | 0.029s | 0.036s | 0.81x | 16,078 B |
| `sieve` | `Sieve(5_000_000)` + `phi_table` + `mobius_table` | 0.095s | 0.086s | 1.10x | 4,675 B |
| `matpow` | `80 x 80` `Mat<Mod<998244353>> ** 10^18` | 0.042s | 0.043s | 0.98x | 7,760 B |
| `crt` | `1,000,000` `std.math.crt` calls | 0.068s | 0.080s | 0.86x | 6,508 B |
| `comb` | `Comb(2_000_000)` + `5,000,000` `choose` queries | 0.081s | 0.071s | 1.15x | 10,132 B |

Aggregate: **p50 = 0.97x**, **max = 1.15x** (targets: `<= 1.1x` p50, `<= 1.5x` max).
Every emitted file is 8–25% of the 64 KB judge ceiling. Profiling history is in
`research/frictions/bench-math.md`.
