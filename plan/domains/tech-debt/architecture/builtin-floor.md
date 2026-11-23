# The builtin floor: classifying `ir_builtin_names`

2026-11-22. Rule (Q1/Q3 default): the compiler keeps intrinsics only for
storage, I/O, bits and numeric primitives; everything else is a std body
any user library could have written. `python3 scripts/debt-ratchet.py
--list ir_builtin_names` lists the names string-matched in
`crates/cpl-ir/src/lower/call.rs` (receiver arms included). A name leaves
the count only when no arm matches it, so a name shared by two arms
(`count` with `len`, `all` with `range`) needs both gone.

Measured 89 before this note; 88 after `choose2` moved (below).

| Class | Stays? | Names |
|---|---|---|
| Storage: types and constructors | intrinsic | `Bool` `Char` `F64` `I128` `Int` `U64` `Str` `List` `Map` `Set` `HashMap` `HashSet` `Some` `some` `MAX` `MIN` `zero` `one` `K` |
| Storage: access to lists, ranges, positions | intrinsic | `len` `size` `is_empty` `at` `try_at` `from_idx` `nth` `first` `last` `swap` `copy` `keys` `values` `start` `end` `bounds` `cuts` `before` `after` `wrap` `range` `count` `rev` `pairs` |
| I/O | intrinsic | `print` `read` `read_line` `read_output` `debug` `flush` `ask` `rand` |
| Bits | intrinsic | `popcount` `ctz` `clz` `bit_width` `bit_ceil` `bit` |
| Numeric primitives | intrinsic | `abs` `min` `max` `sqrt` `floor` `ceil` `round` `trunc` `atan2` |
| Derived: std body exists | move | `isqrt` (math) `sum` `prod` `all` `any` `sort` `sorted` `runs` `span` `first_ge` `first_gt` `last_le` (seq) `prefix` `adjacency` `AddAction` (algebra, prefix, graph) |
| Derived: std body missing | move, body first | `counts` `shuffle` `laws_hold` `AffineAction` `AssignAction` `FlipAction` |
| Moved | done | `choose2` `minmax` |

Totals: 66 intrinsic, 23 derived (2 moved). Floor target for v0.3: 66.

Reclassified after reading the arms: `rev` is an O(1) reverse view over a list, string, array or
range, and `pairs` on a domain is the domain's own pair list (`domain_pairs_list`); both are
storage views, not algorithms.

Two derived names need more than a body:
- `counts` builds a synthesized `Counter_N` struct over `HashMap`/`Map` whose missing keys read 0.
  Moving it needs a std `Counter<T>` type with that indexing (stdlib request).
- `shuffle` is `Mutation::Shuffle`. A Fisher-Yates body over `rand` would change the random stream,
  so seeded programs would print different output; that needs a decision before it moves.

## How a name moves

`choose2` is the template. Delete its arms in `cpl-check`
(`check/known.rs`, `prelude.rs` `value`/`value_doc`/`VALUE_NAMES`) and in
`cpl-ir/src/lower/call.rs`; add the name to `spec/std/prelude.cp` so
programs that used it without a `use` still compile; add a conformance
case; run conformance before and after. Interpreter and C++ then run the
same std body. The builtin had drifted from it: `choose2(-3)` was 6, the
documented answer is 0.

## Order and the perf gate

1. `isqrt`, `minmax`: O(1), no loop; check one generated C++ file.
2. `sum` `prod` `all` `any` `rev` `pairs`: loops; `just bench-zero-cost`
   before and after, same machine, and the corpus suite (`just
   bench-corpus`) for p50/max.
3. `sort` `sorted` `first_ge` `first_gt` `last_le` `runs` `span`
   `prefix` `counts` `adjacency`: hot in ports; move only with a bench
   row showing no regression over the builtin.
4. The `*Action` types and `laws_hold`: owned by stdlib's algebra work;
   the body must exist in `spec/std/algebra.cp` first.

A move that regresses the gate stays a builtin and is logged as debt
here with the measured ratio.

## Measured

- `choose2`: moved (`ab626a6ef`). Conformance 2141/2310 -> 2142/2311 (the new case).
- `minmax`: moved to `spec/std/algebra.cp` as a generic over `Ord`. 50M calls: 0.11-0.13 s before and after
  (interleaved, `g++ -O2`); generated C++ 2,326 -> 2,600 bytes. The 9 corpus files using it have the same
  `cpl check` error counts before and after.
- `isqrt`: tried and backed out; it stays a builtin for now. With `spec/std/math.cp`'s body,
  30M calls took 0.26 s against 0.07 s for the builtin (3.7x, three runs each, `g++ -O2`).
  Generated C++ grew from 2,334 to 3,631 bytes. Conformance went from 2143/2313 to 2122/2311.
  The body divides (`cpl::ediv`) and goes through `cpl::fsqrt`/`cpl::to_int`; the runtime helper
  multiplies. Next try: a body that multiplies, then re-measure. Also, `x.isqrt` needs a
  `self` parameter. The conformance drop is most likely not the change:
  that checkout had no release `cpl`, and the CLI cases fail without one (18 such failures in the
  first `minmax` run, none after building it). Not re-run for `isqrt`.
  Retry with a multiplying body (no division): 0.25-0.33 s against 0.16-0.19 s for the builtin,
  interleaved runs on a noisy machine (about 1.7-2x), so it stays a builtin. What is left is the checked
  `F64 -> Int` conversion (`cpl::to_int(double)` range check) and `std::floor`; the runtime helper
  casts. Blocked on perf: a conversion whose range is proven costs nothing (request in perf worklog).
