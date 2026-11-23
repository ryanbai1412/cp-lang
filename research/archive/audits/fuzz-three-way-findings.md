# Three-way degenerate-input fuzzing: findings

`crates/cpl-cli/tests/fuzz_three_way.rs` runs small programs on seeded inputs
taken from the edges, three ways: `cpl run`, the C++ of `cpl build` compiled
with `-fsanitize=address,undefined -fno-sanitize-recover=all`, and a plain
reference written in Rust in the test. All three must agree: the same output
with status 0, or a runtime error with status 3. A sanitizer report fails the
case even when the output matches.

Run it with `cargo test -p cpl-cli --test fuzz_three_way`. The known traps
below are `#[ignore = "KNOWN WA-TRAP: ..."]` tests; to see them fail, add
`-- --ignored`. Un-ignore each one in the commit that fixes it.

## What it covers

- **Euclidean `/` and `%` on `Int`:** 14 edge values (0, ±1, ±2, ±3, ±7, ±1e9,
  `Int.MAX`, `Int.MIN`, `Int.MIN + 1`), 12 pairs per seed.
- **`I128` and `U64`:** Euclidean `/` and `%` on `I128` edges up to ±2^127 and
  ±1e30. `U64` `/`, `%`, and `+` and `-` that wrap at 0 and `U64.MAX`.
- **`Mod<998244353>` and `Mod<7>`:** `new` of 0, ±1, 6, ±7, `P - 1`, ±`P`,
  `P + 1`, `Int.MAX` and `Int.MIN`, plus `*`, `-`, `Z(-1)`, and `**` with
  exponents from -3 to 40 (negative only when the value has an inverse).
- **Domains of size 0 to 5:** `D.K`, `for x in D`, `[D]Int` and `sum`.
  `D.first` and `D.last` on non-empty domains, and on the empty one.
- **Slices `a[lo..hi]`:** lists of 0 to 6 elements, with `lo` and `hi` from -2
  to `len + 2`. That covers negative, reversed, empty and past-the-end slices;
  every invalid slice must stop the program.
- **Optionals:** lists of `?Int` that are all `none`, partly `none` or empty;
  `max`, `min`, `??`, and `x!` on `none` or past the end.
- **`Fenwick`, `SegTree(min)`, `LazySegTree(AddMin)` and `Dsu`:** domains of
  size 0, 1, 2 and 5, with 14 random operations each, including
  single-element and reversed (empty) ranges.

That is 8 tests and about 120 inputs, which run in 1.4 s warm. The sanitized
binaries are cached under a hash of their C++, the g++ flags and
`g++ --version`, so a warm run compiles nothing.

## Known traps on HEAD

Each trap below is an input where `cpl run` stops cleanly with an `R0001`,
but the C++ hits undefined behavior. A judge builds without sanitizers, so
there it is a silent wrong answer, or an answer that depends on the
optimizer.

1. **`D.first` and `D.last` of an empty domain**
   (`the_ends_of_an_empty_domain_stop`). `cpl run` stops with "`D` has no
   position 0: it is empty". The C++ prints `0` and exits 0. This is a silent
   wrong answer.

   ```
   let n: 0..=5 = read()      // input: 0
   domain D = 0..n
   print(D.first, D.last)
   ```

2. **`x % Int.MIN`** (`int_mod_by_int_min_is_defined`). Every answer fits in
   an `Int`: for example, `-1 % Int.MIN` is `Int.MAX`. But `cpl::emod` in the
   C++ runtime computes `(b ^ s) - s`, which is `|b|`, and for `b == Int.MIN`
   that overflows. UBSan reports "signed integer overflow: 9223372036854775807
   - -1". gcc happens to wrap to the right answer at `-O2`, but the code is
   undefined.

3. **Division and `%` by zero** (`int_division_by_zero_stops`). `cpl::ediv`
   and `cpl::emod` check `b == 0` only under `#ifdef LOCAL`. The plain build
   that a judge compiles divides by zero (SIGFPE, or UB that the compiler may
   fold away), while `cpl run` stops with an `R0001`. By contrast, slices
   check their bounds in every build.

4. **`Int.MIN / -1`** (`int_min_over_minus_one_stops`). This has the same
   cause: the overflow check is under `#ifdef LOCAL`.

5. **`I128.MIN / -1`** (`i128_min_over_minus_one_stops`). The `I128` path has
   no check at all, even under `LOCAL`. UBSan reports "division of
   0x8000...0000 by -1 cannot be represented in type '__int128'".

## Gaps and policy questions found along the way

- **`f.prefix(V.l)` and `s.fold(..)`** do not build:
  `E0501 ... can't handle '.l' of a domain as a value yet`, and `... a range
  with neither end yet`. The docs use both spellings.
- **Reversed ranges are inconsistent:** a reversed slice `a[3..1]` stops the
  program, but `Fenwick.sum(V(3)..=V(1))`, `SegTree.fold` and
  `LazySegTree.apply` / `fold` on a reversed range silently treat it as empty
  (the identity, or nothing applied). The stdlib tests pin this down
  (`f.sum(V(3)..V(1)) == 0`). It is a design choice to make once, for every
  range.
- **`Z(0) ** -1`** stops with "`!` on `none`: there is no value here". It
  should say that 0 has no inverse modulo P.

Everything else agreed on every seed: Euclidean `/` and `%` on all other
`Int` and `I128` edges, wrapping `U64`, `Mod<P>`, non-empty domain ends,
empty domains, every slice, optionals, and all four structures on n = 0, 1,
2 and 5.
