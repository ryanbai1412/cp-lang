# Diagnostics catalogue

Status: draft for 0.3.0. Every error, warning, lint and trap has a
stable code that never changes meaning. A message names the place, the
rule and the fix; one diagnostic per root cause. Codes marked *(0.3)*
are specified here and not yet emitted by default (`delta/0.2.md`; some
are emitted under `CPL_SPEC=1`, the 0.3 edition); the rest are
emitted by 0.2.x (`crates/cpl-diag`, `crates/cpl-lint`). A *(retired)* code is never emitted
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
| E0003 | unterminated_string | |
| E0004 | unknown_char | `#` is not a comment; use `//` |
| E0005 | bad_number | `1e9.5` is not an integer literal |
| E0006 | unexpected_indent | |
| E0007 | bad_char_literal | `'é'` is not ASCII |
| E0101-E0109 | parser structure | unclosed, expected expression/token/block/type/pattern/name, unmatched closer |
| E0110 | unparenthesized_coalesce | `a ?? b + c` needs parentheses |
| E0111 | mixed_comparison_chain | `a < b > c` or a chain with `==`; split it with `and` |
| E0201 | unresolved_name | with the nearest names and a `use` fix for stdlib names |
| E0202 | duplicate_definition | |
| E0203 | unknown_field | |
| E0204 | unknown_type | |
| E0205 | used_before_set | a capture used before it is initialized |
| E0206 | unresolved_module | |
| E0207 | private_name | |
| E0208 | value_name_case *(0.3)* | `let N = 5`: values are lowercase; `N` would read as a type |
| E0301 | wrong_domain | `deg` is indexed by `V`, but `e` is an `E` |
| E0302 | type_mismatch | |
| E0303 | wrong_arg_count | |
| E0304 | not_callable | |
| E0305 | not_indexable | |
| E0306 | position_arithmetic *(retired)* | never emitted: `v + 1` on a position is an `Int` (R04.3.3) |
| E0307 | condition_not_bool | |
| E0308 | optional_compare | `?Int < Int` does not lift; use `?? ` or `if let` |
| E0309 | bad_operands | |
| E0310 | unknown_arg_name | |
| E0311 | bad_memo | `@memo(array)` needs bounded parameter types; `n: Int` is unbounded; use `@memo(map)` or a range |
| E0312 | debug_changes | `debug(...)` argument has side effects |
| E0313 | domain_escapes | a value of fresh type `V` leaves its block |
| E0314 | non_exhaustive_match | this `match` has no arm for `R` |
| E0315 | not_a_token_type *(0.3)* | `read<Bool>()`: `Bool` is not a token type; read a `Str` and compare it, or use a type `read` takes (R10.2.3) |
| E0401 | assign_to_let | |
| E0402 | missing_mut_at_call | write `mut seen` at the call |
| E0403 | mut_of_immutable | |
| E0404 | top_level_var_in_fn *(retired)* | never emitted: a `fn` may read and write a top-level `var` |
| E0405 | assign_fixed_field *(0.3)* | `e.w` is not `var`; declare `var w` or rebuild with `e with w: x` |
| E0406 | var_lambda_recursion | a `var`-bound lambda cannot call itself |
| E0407 | duplicate_variant_tag | two variants of one enum read as the same token; give one a different `@tag` |
| E0501 | unsupported | |
| E0502 | non_ascii | |
| E0503 | removed_syntax | names the 0.3 spelling (`D.bounds` is `D.cuts`, `(+, max)` is `each(+, max)`, `.enumerate()` is `.keyed().items`, `@alt(main)` is `@alt(program)`) |
| E0504 | not_implemented | planned syntax this version reads but does not implement yet; the help names today's spelling |
| E0601 | lent_twice | `a` is lent `mut` here while lent at line N |
| E0602 | used_while_lent | `xs` is written while the view `w` (line N) still reads it |
| E0603 | lent_escapes | a lent value outlives its source |
| E0604 | bad_projection | |
| E0605 | bad_index_impl | |
| E0606 | missing_impl | with the trait's own message (`SegTree` needs `Op<T>`: an associative `fn (T, T) -> T`) |
| E0607 | moved_while_captured | `xs` is captured by `f` (line N) and cannot move |
| E0608 | heap_copy | `let b = a` moved `a`; use `a.clone()` to copy |
| E0701 | ambiguous_method | names both candidates; write `Trait.m(x)` |
| E0702 | overload_by_type *(0.3)* | two `f` with the same labels |
| E0703 | bare_op_tuple *(0.3)* | `(+, max)` is not an operation; write `each(+, max)` or `Semiring(max, +)` |
| E0704 | row_missing_field *(0.3)* | the record lacks field `w` required by `(w: W, ..)` |
| E0705 | law_unsupported_specialization *(0.3)* | `Fenwick` takes no `act:`; use `SegTree(op, a, act:)` |
| E0706 | empty_fold_no_identity *(0.3)* | folding with `Op` gives `?T`; unwrap or pass a `Monoid` |
| E0801 | context_not_constant | `with` needs a compile-time value; for a runtime modulus use `type Z = Mod(p)` |
| E0802 | with_not_first | a bare `with` line must come first in its block |
| E0803 | question_in_condition | `x?` in a condition; write `... ?? false` |
| E0804 | dropped_question *(0.3)* | a `?` whose result is dropped |
| E0805 | fresh_type_leaks_test *(0.3)* | a position of a per-test domain stored above `@!multitest` |
| E0806 | inference_needs_annotation | `var x = None`; annotate `?Int` |
| E0901 | no_u8 | `U8` is not a type; use `0..=255` |
| E0902 | range_store_unprovable_const *(0.3)* | a constant out of the range type |
| E0903 | int128_unavailable *(0.3)* | `I128` on a judge without `__int128` |
| E0904 | memory_over_profile *(0.3)* | peak memory at the input bounds exceeds the judge's limit (with the largest arrays) |
| E9998 | internal_unsupported *(retired)* | never emitted; unsupported constructs are E0501 |
| E9999 | internal | |

## Warnings

| Code | Name | |
|---|---|---|
| W0101 | unused_value | a result is dropped (see `@droppable`) |
| W0201 | size_near_limit *(0.3)* | output C++ above 32 KB, or half of a smaller `source_kb` (R13.1.1) |

## Lints

Emitted by 0.2.x: L0001 unknown_lint, L0101 position_arithmetic,
L0201 var_never_reassigned, L0202 unread_field, L0203
shadows_top_level, L0204 turbofish_type_args, L0205 pow_method, L0206
getter_parens, L0207 is_in_cond_header, L0208 redundant_prelude_import,
L0301 float_equality, L0412 domain_keyed_map, L0413 memo_key_space,
L0414 unproven_list_index, L0415 loop_invariant_mod_div, L0501
search_loop, L0502 discarded_result, L0503 symmetric_write, L0504
loop_reduction, L0505 index_only_loop, L0506 repeated_literal, L0507
push_loop, L0508 none_check_unwrap, L0509 free_fold, L0510 by_twin,
L0511 tree_order, L0512 domain_spelling, L0513 prefix_twin, L0514-L0521
hand_rolled_{range_tree, sorted_range_method, string_algo, dsu, sieve,
binary_lifting, answer_search, prefix_sum}.

*(0.3)*:

| Code | Name | Fires on |
|---|---|---|
| L0601 | same_place_read_write | `a[l] += inc(mut a[l])`, `xs[0] = xs.pop()!` |
| L0602 | chain_walked_twice | a lazy chain value consumed twice |
| L0603 | map_recomputed | `.map(f)` view read many times by index |
| L0604 | mut_generator_rewalked | a generator holding a `mut` lend walked twice |
| L0605 | literal_slice_index | `w[0]` on a slice that keeps source keys; use `w.first!` or `w.nth(0)` |
| L0606 | wide_array | a large `[V]Int` whose writes all fit a narrower range |
| L0607 | padding | padding over 20 % of a large array's element |
| L0608 | parens_on_o1 / missing_parens | `xs.len()` or `g.transpose` (decision 140) |
| L0609 | cache_field_written_outside | an `@cache` field written outside its type's methods |

## Runtime traps (checked builds)

| Code | Trap |
|---|---|
| R0001 | generic runtime error |
| R0101 | index out of range (raw `Int` or `List` index) |
| R0102 | `D(x)` out of the domain |
| R0201 | range store out of range |
| R0202 | overflow |
| R0203 | division by zero |
| R0301 | `x!` on `None` |
| R0401 | assertion failed |
| R0402 | `todo` reached |
| R0501 | bad input token (position and expected type) |
| R0502 | input out of its read contract |
| R0601 | stack exhausted |

Every trap flushes output first and prints values in the file's base.
