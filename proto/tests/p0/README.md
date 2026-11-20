# P0 regressions

One case per silent-wrong-answer bug the scouts and blind critics found in the prototype. `x.cp` is built by
`proto/cpc`, compiled under ASan + UBSan with `-DLOCAL`, and must print `x.out`; the cases marked in
`../test_p0_regressions.py` also run in the reference interpreter `proto/cpi`, which must agree.

- `shift64`: `1 << 40` is a 64-bit shift, never 32-bit undefined behaviour.
- `value_copy`: `var t = s` and `var next = ways` copy; a later write to one never shows in the other.
- `array_literal`: `[D]T = [a, b, c]` holds one value per position.
- `brute_not_main`: a parameterless `@brute` is an oracle only and never runs in the judge build.
- `float_format`: `{x:.3f}` rounds to a fixed precision and never prints `-0.000`.
- stress honesty (in the test script): a bug that needs size 12 is found, the summary reports coverage,
  outputs that differ only in line breaks are a mismatch, and an all-rejected run is not "OK".

## Synthesis #2 (`s2_*`)

One case per kernel rule of `research/language-architecture-synthesis-2.md`, plus the format specifiers. Each
`s2_x.cp` must print `s2_x.out` on both paths (cpc under ASan + UBSan, and cpi); an `s2_x_trap.cp` must stop
with a runtime error on both. The spellings the synthesis rejects are `../bad/s2_*.cp`.

- `s2_keep_bool`: `x min= y` / `x max= y` is a Bool, true when the place changed.
- `s2_read_flow`: `read()` / `ask()` take their type from the place, parameter or return type; `read::<T>()` otherwise.
- `s2_index_conv`: `V(x)`, `Int(v)`, `v.to_idx`, trapping `V.from_idx(i) -> V` and fallible `V.try_from_idx(i) -> ?V`.
- `s2_int_widen`: an Int beside an F64 widens in `+ - * / **`; `Int / Int` stays integer division.
- `s2_format`: `{v:W}`, `{v:0W}`, `{x:.D}`, `{x:.Df}`, `{n:x}`, `{n:b}`, and 64-bit shifts.
- `s2_char_step`, `s2_char_step_trap`: `Char + Int -> Char`, checked to stay ASCII.
- `s2_place`, `s2_place_trap`, `s2_array_literal_trap`: a list literal fills a `[D]T`; `x!` on a `var ?T` is a place.

## Synthesis #3 (`s3_*`)

One case per leap in `research/language-architecture-synthesis-3.md` (`0.2.2`). Positive cases print `s3_x.out` on both `cpc` (ASan + UBSan) and `cpi` (except `*_cpc` for cpc-only containers); `*_trap` must trap at runtime on both. Negative cases in `../bad/s3_*.cp` lock all four anti-spookiness invariants, strict nominal domain safety on `.get(?I)`, no scalar broadcast, and `const struct` / `if const` checks.

- `s3_multibind_shapes`: homogeneous `let n, m: T = read()`, `var lo, hi: Int = (0, m)`, and `[n]Int` / `[n - 1](...)` input shapes.
- `s3_narrow_n1`: strictly local syntactic Rule N1 narrowing (`if x != none`, diverging `if x == none: return`, `and`/`or`, `while cur != none`, redundant `!` tolerance).
- `s3_narrow_n2`, `s3_narrow_n2_containers_cpc`: Rule N2 non-empty narrowing (`List`, `MaxHeap`, `Deque`, and userland `@law some_when(not self.is_empty)`).
- `s3_array_get_from_idx`, `s3_from_idx_trap`: optional-accepting `a.get(?I)` (1D, 2D, `List`), trapping `D.from_idx(i) -> D`, and fallible `D.try_from_idx(i) -> ?D`.
- `s3_const_config`: `const struct` POD configs, `Rooted<V, const C: TreeCfg = DEFAULT>`, `if const` compile-time branching, and `()` field erasure verified in emitted C++17.
