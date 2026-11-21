# Spec chapter 7: numbers

Status: **draft v2** [proposed]. Fills review round 10 item 35.

## 7.1 Integer types

| type | meaning | overflow |
|---|---|---|
| `Int` | `-(2^63 - 1) ..= 2^63 - 1`; `INT64_MIN` is the niche for `none`, and `read` rejects it [r12 R5] | traps in debug, wraps in release [you] |
| `I32`, `U32`, `U64`, `I128`, … | fixed-width **storage** | traps in debug, wraps in release [you] |
| range type `lo..=hi` | a **contract**: assigning an unproven value traps in both builds | n/a |

- Pick by meaning: a range for values with a real bound, a fixed width for hot counters whose bound you can't
  state cheaply and for bit tricks.
- A range type's storage is the narrowest integer that holds it.
- **Widening is implicit; narrowing is a checked conversion** (`I32(x)`), everywhere, including returns. `I32.try(x) -> ?I32` is the
  fallible form; every type-call conversion has both [p1 S1].
  A type widens only into one holding every value it can hold: `U32 → Int` yes, `U64` and `Int` never mix
  implicitly [r11 N21].
- **The niche:** producing `Int`'s niche value (`INT64_MIN`) traps in debug like overflow; in release it is
  part of the wrap contract, so a wrapped value stored in `?Int` may read as `none` [r11 N20].
- When an interval result doesn't fit in i64 (`1..=1e9` cubed), the result is plain `Int` [r11 N23].
- `with checks(ranges: off)` removes range-contract checks in release; debug keeps them.
- Struct fields may use only static bounds, so layout never depends on input.
- Literals: an `e` literal (`1e6`, `1.5e5`) is an `Int` when its value is integral and fits, an error when it
  is integral and doesn't (`1e19`), and `F64` otherwise; `1_000_000` also works [r11 N28].

## 7.2 Arithmetic

- `/` is floor division and `%` is the non-negative remainder for a positive divisor. `ceil_div(a, b)`
  exists. `**` is power.
- **Interval arithmetic on range types** [r7 6]: `+ - * / %` compute result ranges, so `x % d` has type
  `0..d` when `d: 1..`, and indexing a `[0..d]T` with it needs no check. C++ gets a plain `%` when the
  ranges show `x ≥ 0`, and a floor-adjusted one otherwise.
- **Wrapping is explicit** [r7 17]: `wrapping_add`, `wrapping_neg`, … and helpers `lowest_bit`, `popcount`,
  `ctz`, `clz`, so bit tricks don't trap in debug.
- **Bit operators** `& | ^ ~ << >>`: a shift amount outside `0..width` traps in debug and is masked in
  release (`k & 63`), so no C++ UB; `>>` on signed values is arithmetic [r11 N22].
- Compound assignment: `+=`, `-=`, `*=`, `max=`, `min=` (chapter 3.4 for `?T`; 6.3 for `min=` as a condition).
- **Range-type join** [r16 R4]: `chain`, list literals and `if` branches give range types their hull, so
  `[(1..=h, 1..=w)]` chained with `[(h, w)]` is a sequence of `(1..=h, 1..=w)`.
- **Range lint** [r16 R24]: `a .. len - 1` / `a .. count - 1` and `a ..= len` / `a ..= count` warn "did you
  mean `..=` / `..`?", where Rust-style ranges' off-by-one shows up.
- `Bool` doesn't convert to numbers implicitly: `Int(b)`, `Z(b)`.

## 7.3 Modular arithmetic

- `type Z = Mint(mod: 998244353)`: specialized, constant modulus. `type Z = Mint(mod: p)` with `p: Prime`
  read from input: a fresh scoped type with Barrett reduction (chapter 1.5). One local edit between them.
- `?Z` is free: values ≥ the modulus are the niche.
- `Z(x)` converts from `Int`, reducing negatives by floor. `z.inv()` and `/` exist only for a prime modulus;
  the `Prime` type checks primality at the read in **both** builds (one deterministic Miller–Rabin) [r11 N24].
- Storage: modulus `< 2^31` → `u32`; larger → `u64` with an `I128` multiply, which the profile must allow
  [r11 N25].

- **Combinatorics** [p2 F8]: `Z.factorials(n)` builds `fact`/`inv_fact` with one `inv` and a backward pass; `f.binom(n, k)` is
  0 outside `0 ≤ k ≤ n`; `cp explain` shows the two `[0..=n]Z` tables.
- **Carries** [p2 F10]: `overflowing_add`/`overflowing_sub -> (T, Bool)`, `U64.lo32()`/`.hi32()`.

## 7.4 Floats

- `F64`. Float `==` is rejected by lint; `with lint(float_eq: allow)` allows it [you].
- `approx(a, b, abs:, rel:)` is an ordinary function and the only float equality.
- `F64(x)` converts explicitly; there is no implicit `Int → F64`. `Int(f)` is a compile error naming
  `floor` / `ceil` / `round` / `trunc` [r11 N26].
- Output precision comes from `with io(precision: k)` or `print(x, precision: k)`; the default is the judge
  profile's [r7 31].

## 7.5 Lowering

`Int` → `int64_t`; ranges → narrowest `intN_t`/`uintN_t`; debug overflow via `__builtin_*_overflow`; `Mint`
constant → `uint32_t` (or `uint64_t`) with compile-time modulus; runtime → the same plus Barrett constants
passed as a hidden argument.

## 7.5b Round-3 rules [p3]

- Arithmetic between equal fixed widths stays in that width (7.1 wrap/trap rules), so hashing and bit tricks
  work. Mixing with `Int` widens losslessly (`I32`/`U32` to `Int`; `U64` with `Int` is an error) and gives
  `Int`; storing back needs `I32(x)`.
- `Int(z)` on a `Mint` is the residue in `0..mod` (total).
- Stdlib: `pow_mod(b, e, m)`, `n.prime_factors()`, `gcd`/`lcm` on every integer width including `I128`,
  printing `I128`, `x.sign()`, `abs(x)`, and `Ratio<T>` (reduced, with `num`/`den`).

## 7.6 Open questions

1. Should `Int` overflow trap in release too, given the ≤ 5% check budget? Ryan decided: wrap in release.
2. `I128` availability depends on the judge compiler; the bundler must reject it on profiles without it.
