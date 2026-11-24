# Diagnostics catalogue

Status: draft for 0.3.0. Every error, warning, lint and trap has a
stable code that never changes meaning. A message names the place, the
rule and the fix; one diagnostic per root cause. Codes marked *(0.3)*
are specified here and not yet emitted; the rest are emitted by
`cpl` (`crates/cpl-diag`, `crates/cpl-lint`). A *(retired)* code is never emitted
and never reused; a *(reserved)* one is kept for a check that reports
under another code today. `crates/cpl-cli/tests/catalogue.rs` holds the
table to the compiler: every emitted code has a trigger that produces
exactly it and a fix that clears it.

Families: `E00` lexing, `E01` parsing, `E02` names, `E03` types and
domains, `E04` mutation, `E05` unsupported and removed, `E06` lending,
`E07` traits and generics, `E08` contexts, checks and fresh types, `E09`
numbers and layout; `W` warnings; `L` lints; `R` runtime traps.

## Errors

| Code | Name | Example message and fix |
|---|---|---|
| E0001 | tab_indent | tabs are not indentation; use 4 spaces |
| E0002 | bad_dedent | this line dedents to no enclosing block |
| E0003 | unterminated_string | this string is never closed |
| E0004 | unknown_char | `#` is not a comment; use `//` |
| E0005 | bad_number | `1e9.5` is not an integer literal |
| E0006 | unexpected_indent | this line is indented, but no block starts above it |
| E0007 | bad_char_literal | `'é'` is not ASCII |
| E0101-E0109 | parser structure | unclosed, expected expression/token/block/type/pattern/name, unmatched closer |
| E0110 | unparenthesized_coalesce | `a ?? b + c` needs parentheses |
| E0111 | mixed_comparison_chain | `a < b > c` or a chain with `==`; split it with `and` |
| E0112 | else_falls_through | the `else` block of `let p = e else:` can reach its end; end it with `return`, `break`, `continue` or a trap (R01.6.2) |
| E0201 | unresolved_name | with the nearest names and a `use` fix for stdlib names |
| E0202 | duplicate_definition | `S` is already defined in this file |
| E0203 | unknown_field | List<Int> has no `sortd` in the spec: no std module declares it; did you mean `sorted`? |
| E0204 | unknown_type | no type or domain named `Itn`, and not in the stdlib (yet) |
| E0205 | used_before_set | a capture used before it is initialized |
| E0206 | unresolved_module | no module `std.no_such_module`, and not in the stdlib (yet) |
| E0207 | private_name | `sa_is` is private to `std.strings` |
| E0208 | value_name_case | `let N = 5`: values are lowercase; `N` would read as a type |
| E0209 | definition_cycle | `f`'s parameter types depend on `f` itself: a domain sized by a call of `f` is a parameter type of `f` |
| E0301 | wrong_domain | `deg` is indexed by `V`, but `e` is an `E` |
| E0302 | type_mismatch | `x` is declared an Int, but `"abc"` is a Str |
| E0303 | wrong_arg_count | `add` takes 2 arguments (a, b), but `b` is missing |
| E0304 | not_callable | `x` is an Int, not a fn |
| E0305 | not_indexable | `x` is an Int and can't be indexed |
| E0306 | position_arithmetic *(retired)* | never emitted: `v + 1` on a position is an `Int` (R04.3.3) |
| E0307 | condition_not_bool | the condition needs a Bool, but `x` is an Int |
| E0308 | optional_compare | `?Int < Int` does not lift; use `?? ` or `if let` |
| E0309 | bad_operands | `+` can't combine a Str and an Int |
| E0310 | unknown_arg_name | `f` has no parameter named `cuont` |
| E0311 | bad_memo | `@memo(array)` needs bounded parameter types; `n: Int` is unbounded; use `@memo(map)` or a range |
| E0312 | debug_changes | `debug(...)` argument has side effects |
| E0313 | domain_escapes | a value of fresh type `V` leaves its block |
| E0314 | non_exhaustive_match | this `match` has no arm for `R` |
| E0315 | not_a_token_type *(0.3)* | `read<Bool>()`: `Bool` is not a token type; read a `Str` and compare it, or use a type `read` takes (R10.2.3) |
| E0316 | field_bound_from_input | `x: 0..=n` in `struct P`: a field's bounds are fixed before input is read, but `n` is read; bound it by a constant or keep the value outside the struct (R03.1.3) |
| E0317 | getter_needs_defaults | `@get fn f(self, k: Int)`: a getter is read like a field, so every parameter but `self` needs a default; give `k` one or drop `@get` (R02.6.6) |
| E0318 | unreachable_arm | unreachable arm: `less` above already matches everything; a lowercase name in a pattern binds, did you mean `Less`? (R01.7.3) |
| E0401 | assign_to_let | `x` is a `let`, so it can't be assigned |
| E0402 | missing_mut_at_call | write `mut seen` at the call |
| E0403 | mut_of_immutable | `a` is a `let`, so it can't be passed as `mut` |
| E0404 | top_level_var_in_fn *(retired)* | never emitted: a `fn` may read and write a top-level `var` |
| E0405 | assign_fixed_field | `e.w` is not `var`; declare `var w` or rebuild with `e with w: x` |
| E0406 | var_lambda_recursion | a `var`-bound lambda cannot call itself |
| E0407 | duplicate_variant_tag | two variants of one enum read as the same token; give one a different `@tag` |
| E0408 | assign_into_range *(0.3)* | `r[i] = x`: `r` is a `Range`, which has no storage to write; copy it with `List(r)` first (R07.2.5) |
| E0409 | fn_falls_through | `f` gives an `Int`, but this `if` has no `else`: a path through the body ends without a value |
| E0501 | unsupported | `yield` outside a generator |
| E0502 | non_ascii | `é` is not ASCII; a Str literal holds single-byte ASCII characters only |
| E0503 | removed_syntax | names the 0.3 spelling (`D.bounds` is `D.cuts`, `(+, max)` is `each(+, max)`, `.enumerate()` is `.keyed().items`, `@alt(main)` is `@alt(program)`, `@allow(x)` is `with lint(x: allow)`) |
| E0504 | not_implemented | planned syntax this version reads but does not implement yet; the help names today's spelling |
| E0505 | bad_library_lint | an `@lint` that cannot be used: `from` or `to` is not one expression or statement, `to` drops or invents a parameter, `to` does not call the fn, or the name is used twice in the module (R06.4.9) |
| E0601 | lent_twice | `a` is lent `mut` here while lent at line N; one place passed twice says `x` is lent to this call twice |
| E0602 | used_while_lent | `xs` is written while the view `w` (line N) still reads it |
| E0603 | lent_escapes | a lent value outlives its source |
| E0604 | bad_projection | `view` is declared `lends`, so every path through it must reach one `lend place` |
| E0605 | bad_index_impl | `K` can't be sized by the var `n`: it could change after arrays are made |
| E0606 | missing_impl | with the trait's own message (`SegTree` needs `Op<T>`: an associative `fn (T, T) -> T`) |
| E0607 | moved_while_captured | `xs` is captured by `f` (line N) and cannot move |
| E0608 | heap_copy | `let b = a` moved `a`; use `a.clone()` to copy |
| E0701 | ambiguous_method | names both candidates; write `Trait.m(x)` |
| E0702 | overload_by_type | two `f` with the same labels |
| E0703 | bare_op_tuple | `(+, max)` is not an operation; write `each(+, max)` or `Semiring(max, +)` |
| E0704 | row_missing_field | the record lacks field `w` required by `(w: W, ..)` |
| E0705 | law_unsupported_specialization | `Fenwick` takes no `act:`; use `SegTree(op, a, act:)` |
| E0706 | empty_fold_no_identity | folding with `Op` gives `?T`; unwrap or pass a `Monoid` |
| E0801 | context_not_constant | `with` needs a compile-time value; for a runtime modulus use `type Z = Mod(p)` |
| E0802 | with_not_first | a bare `with` line must come first in its block |
| E0803 | question_in_condition | `x?` in a condition; write `... ?? false` |
| E0804 | dropped_question | a `?` whose result is dropped |
| E0805 | fresh_type_leaks_test *(0.3)* | a position of a per-test domain stored above `@!multitest` |
| E0806 | inference_needs_annotation | `var x = None`; annotate `?Int` |
| E0901 | no_u8 | `U8` is not a type; use `0..=255` |
| E0902 | range_store_unprovable_const | a constant out of the range type |
| E0903 | int128_unavailable *(0.3)* | `I128` on a judge without `__int128` |
| E0904 | memory_over_profile *(0.3)* | peak memory at the input bounds exceeds the judge's limit (with the largest arrays) |
| E9998 | internal_unsupported *(retired)* | never emitted; unsupported constructs are E0501 |
| E9999 | internal | internal error in cpl while handling `textDocument/hover`: ...; this is a bug in cpl, not in your code |

## Warnings

| Code | Name | |
|---|---|---|
| W0101 | unused_value | a result is dropped (see `@droppable`) |
| W0201 | size_near_limit | output C++ above 32 KB, or half of a smaller `source_kb` (R13.1.1) |

## Lints

Each lint is a directory: `spec/lints/<name>/` for the lints `cpl lint`
runs, `spec/checker-lints/<name>/` for the ones `cpl check` runs. Its
`lint.toml` holds the code, the level and a one-line summary, and its
`before.cp`, `after.cp` and `near_miss.cp` are the cases the lint must
fire on, fix, and leave alone. [lints.md](lints.md) is the table of all of
them, generated from those files; `cpl explain L0540` prints one lint's
page.

## Runtime traps (checked builds)

| Code | Trap | Example message |
|---|---|---|
| R0001 | generic runtime error | a shift by 70 is outside 0..64 |
| R0101 | index out of range (raw `Int` or `List` index) | index 5 is outside a list of length 2 |
| R0102 | `D(x)` out of the domain | 7 is not in `D`, which is 0..=2 |
| R0201 | range store out of range | 14 is outside `0..10` |
| R0202 | overflow | integer overflow: `4000000000000000000 * 3` does not fit in an `Int` (64 bits) |
| R0203 | division by zero | division by zero: `5 / 0` |
| R0301 | `x!` on `None` | `!` on `None`: there is no value here |
| R0401 | assertion failed | assertion failed: k > 2 |
| R0402 | `todo` reached | `todo` reached |
| R0501 | bad input token (position and expected type) | expected a number, found `abc` (input line 1, column 1) |
| R0502 | input out of its read contract | 9 is outside `1..=5` (input line 1, column 1) |
| R0601 | stack exhausted | the stack ran out: `f` never stops calling itself? |

Every trap flushes output first and prints values in the file's base.
