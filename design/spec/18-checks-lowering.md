# 18. Checks, stack and lowering

**Layer:** core C13–C15.

Status: [proposed].

**Why.** Safety checks that can't be proven stay in submissions and trap [you]; recursion runs on a big stack; the C++ output is predictable. Core, except lints, which are library-owned [p4].

## 18.1 Lowering

| construct | C++ |
|---|---|
| position `v: V` | `uint32_t` (narrowest unsigned that holds the max) |
| `?V` | same width, niche value `UINT32_MAX` |
| `[V]T` | `T*` + count in a heap buffer (never a VLA on the stack) |
| `A.Cut` | `uint32_t` holding the count |
| `Range<D>` | two ordinals, half-open |
| hidden domain size | an extra `uint32_t` parameter, only where generic code needs it |
| `Bits<D>` | `uint64_t` (or narrower when `D.count` is statically small) |
| `Mint(mod: const)` | value + compile-time modulus; runtime modulus: value + per-activation Barrett constants |

Checks remaining after typing: conversions from `Int` (`D.at`, reads), anonymous-array indexing, and range
contracts. All trap with the source line and values in your terms (one-based if `io(base: 1)`).

## 18.2 Checks and lints

- `with checks(bounds: off)` and `with checks(ranges: off)` remove the corresponding remaining checks in
  **release only**. Debug keeps them, so a program that passes debug has no UB in release on that input
  [r4 27].
- `with lint(float_eq: allow)` allows a lint in a scope [you]. Lints are contexts so they compose with
  everything else and need no comment syntax.
- `@unchecked` and `@checks(...)` attributes are gone.

## 18.3 Recursion and the stack [you, 2026-11-21]

- The language assumes a big stack. The bundler runs `main` on a thread whose stack size comes from the
  judge profile (default 1 GB); a profile can set it or disable the thread.
- There is no depth inference, no depth annotation and no depth counting, in either build. A stack overflow
  crashes like C++; the debug runtime's signal handler prints the call trace.
- Runtime-sized arrays and static arrays over 4 KB never go on the stack (Tourism's accepted C++ segfaulted
  at n = 10⁶ on 8 MB because of VLAs); small static arrays follow chapter 13.4.

## 18.4 Lowering

`Int` → `int64_t`; ranges → narrowest `intN_t`/`uintN_t`; debug overflow via `__builtin_*_overflow`; `Mint`
constant → `uint32_t` (or `uint64_t`) with compile-time modulus; runtime → the same plus Barrett constants
passed as a hidden argument.

## 18.6 [p4] lints are library
A lint is declared by the library that owns the names it mentions (the range lint by the `Range`/`len`
module), and configured through `with lint(…)`. The linter has no hard-coded stdlib names.
