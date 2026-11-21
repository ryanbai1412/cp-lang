# 11. Value types and numbers

**Layer:** core C2–C3. Status: [proposed].

## 11.1 Overview

**What.** Value types are integers (`Int`, fixed widths, range types), `F64`, `Bool`, `Byte`, tuples,
structs, enums and `?T` (chapter 23). All have value semantics (chapter 16).

**Why range types are core.** A range type `lo..=hi` is both a contract (checked on assignment) and the
input to interval arithmetic, which is what lets `x % d` index a `[0..d]T` with no check and lets `read`
validate input for free. That needs the type checker; a library type couldn't propagate bounds.

**Cost.** A range type is stored in the narrowest integer holding it. Interval arithmetic is compile-time
only. Contract checks on unproven assignments remain in submissions [you] unless
`with checks(ranges: off)`.

**Rejected.** Refinement types with an SMT solver (slow, unpredictable diagnostics); plain `int`/`long long`
(the bug class); unbounded integers by default (cost).

## 11.2 Enums [p1 S5]

**Layer.** Core: `enum` with payloads and exhaustive `match`. `labels(…)` is sugar for a payload-free enum deriving `Index` (library impl).

- A labeled domain is the plain enum: `domain Side = labels(left, right)` gives `Side.left`, a `[Side]T`
  array, `match` exhaustiveness and named-argument resolution (`order: desc`).
- `enum Shape: kind(Kind), angle` adds payload variants; it is not an index space.

## 11.3 Integer types

| type | meaning | overflow |
|---|---|---|
| `Int` | `-(2^63 - 1) ..= 2^63 - 1`; `INT64_MIN` is the niche for `none`, and `read` rejects it [r12 R5] | traps in debug, wraps in release [you] |
| `I32`, `U32`, `U64`, `I128`, … | fixed-width **storage** | traps in debug, wraps in release [you] |
| range type `lo..=hi` | a **contract**: assigning an unproven value traps in both builds | n/a |

- Pick by meaning: a range for values with a real bound, a fixed width for hot counters whose bound you can't
  state cheaply and for bit tricks.
- A range type's storage is the narrowest integer that holds it plus its niche: unsigned when `lo ≥ 0`, signed
  otherwise. Arithmetic always happens in `Int`, so unsigned storage never wraps in C++ [2impl Q22].
- **Loop variables:** in `for i in a .. b` (or `..=`), `i` has the range type `a .. b` when `a` and `b` are literals,
  `let` bindings or `.count`/`.len` of a `let`; its storage is the static hull. Otherwise `i: Int` [2impl Q15].
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

## 11.4 Arithmetic

**Layer.** The operators are trait calls (`Add`, `Rem`, …, chapter 14) whose impls for primitive integers are library code over the checked/wrapping intrinsics (`20`). Interval arithmetic on range types is core: it's typing, not code. The range lint is owned by the `Range` library [p4].

- `/` is floor division and `%` is floor remainder: `a == (a / b) * b + a % b`, and `a % b` has the sign of `b`
  (`7 % -3 == -2`). `ceil_div(a, b)` exists [2impl Q18].
- Division or remainder by zero traps in both builds ("division by zero"); it is not a removable check.
- `**` is power. `0 ** 0 == 1`; a negative exponent on an integer traps in both builds ("negative exponent";
  use `Z(a).pow(-k)` or floats) [2impl Q10].
- **Interval arithmetic on range types** [r7 6]: `+ - * / %` compute result ranges, so `x % d` has type
  `0..d` when `d: 1..`, and indexing a `[0..d]T` with it needs no check. C++ gets a plain `%` when the
  ranges show `x ≥ 0`, and a floor-adjusted one otherwise.
- **Wrapping is explicit** [r7 17]: `wrapping_add`, `wrapping_neg`, … and helpers `lowest_bit`, `popcount`,
  `ctz`, `clz`, so bit tricks don't trap in debug.
- **Bit operators** `& | ^ ~ << >>`: a shift amount outside `0..width` traps in debug and is masked in
  release (`k & 63`), so no C++ UB; `>>` on signed values is arithmetic [r11 N22].
- Compound assignment: `+=`, `-=`, `*=`, `max=`, `min=` (chapter 23 for `?T`; 15.1 for `min=` as a condition).
- **Range-type join** [r16 R4]: `chain`, list literals and `if` branches give range types their hull, so
  `[(1..=h, 1..=w)]` chained with `[(h, w)]` is a sequence of `(1..=h, 1..=w)`.
- **Range lint** [r16 R24]: `a .. len - 1` / `a .. count - 1` and `a ..= len` / `a ..= count` warn "did you
  mean `..=` / `..`?", where Rust-style ranges' off-by-one shows up.
- `Bool` doesn't convert to numbers implicitly: `Int(b)`, `Z(b)`.

## 11.5 Floats

**Layer.** `F64` and its intrinsics are core; `approx`, precision and the float-equality lint are library.

- `F64`. Float `==` is rejected by lint; `with lint(float_eq: allow)` allows it [you].
- `approx(a, b, abs:, rel:)` is an ordinary function and the only float equality.
- `F64(x)` converts explicitly; there is no implicit `Int → F64`. `Int(f)` is a compile error naming
  `floor` / `ceil` / `round` / `trunc` [r11 N26].
- Output precision comes from `with io(precision: k)` or `print(x, precision: k)`; the default is the judge
  profile's [r7 31].
