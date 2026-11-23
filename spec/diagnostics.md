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
| E0401 | assign_to_let | `x` is a `let`, so it can't be assigned |
| E0402 | missing_mut_at_call | write `mut seen` at the call |
| E0403 | mut_of_immutable | `a` is a `let`, so it can't be passed as `mut` |
| E0404 | top_level_var_in_fn *(retired)* | never emitted: a `fn` may read and write a top-level `var` |
| E0405 | assign_fixed_field | `e.w` is not `var`; declare `var w` or rebuild with `e with w: x` |
| E0406 | var_lambda_recursion | a `var`-bound lambda cannot call itself |
| E0407 | duplicate_variant_tag | two variants of one enum read as the same token; give one a different `@tag` |
| E0408 | assign_into_range *(0.3)* | `r[i] = x`: `r` is a `Range`, which has no storage to write; copy it with `List(r)` first (R07.2.5) |
| E0501 | unsupported | `yield` outside a generator |
| E0502 | non_ascii | `é` is not ASCII; a Str literal holds single-byte ASCII characters only |
| E0503 | removed_syntax | names the 0.3 spelling (`D.bounds` is `D.cuts`, `(+, max)` is `each(+, max)`, `.enumerate()` is `.keyed().items`, `@alt(main)` is `@alt(program)`, `@allow(x)` is `with lint(x: allow)`) |
| E0504 | not_implemented | planned syntax this version reads but does not implement yet; the help names today's spelling |
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

Emitted by 0.2.x: L0001 unknown_lint, L0101 position_arithmetic, L0201
var_never_reassigned, L0202 unread_field, L0203 shadows_top_level, L0204
turbofish_type_args, L0207
is_in_cond_header, L0208 redundant_prelude_import, L0209
redundant_arg_name, L0210 compound_self_ref, L0211 state_above_multitest, L0301 float_equality, L0412 domain_keyed_map, L0413
memo_key_space, L0414 unproven_list_index, L0415 loop_invariant_mod_div,
L0501 search_loop, L0502 discarded_result, L0503 symmetric_write, L0504
loop_reduction, L0505 index_only_loop, L0506 repeated_literal, L0507
push_loop, L0508 none_check_unwrap, L0509 free_fold,
L0511 tree_order, L0512 domain_spelling, L0514
hand_rolled_range_tree, L0516-L0521
hand_rolled_{string_algo, dsu, sieve, binary_lifting, answer_search,
prefix_sum}, L0522 copy_sort_writeback, L0523 chmin_chmax_if, L0524 bool_ternary_print, L0525 peek_then_pop, L0526 ceil_div_formula, L0527 neg_mod_fixup, L0528 reverse_index, L0529 float_sqrt_for_int, L0530 sentinel_table, L0531 manual_mod_reduce, L0532 nested_list_grid, L0534 grid_neighbour_step, L0535 grid_bounds_check, L0536 sentinel_scalar, L0537 manual_adjacency_list, L0539 dx_dy_arrays, L0553 stale_entry_skip, L0563 map_as_multiset, L0571 negated_heap, L0572 sort_for_extreme, L0573 fermat_inverse, L0574 char_offset_index, L0575 push_pop_pair, L0576 pop_min_loop.

*(0.3)*:

| Code | Name | Fires on |
|---|---|---|
| L0540 | manual_bit_test | `(x >> i) & 1 == 1`; use `x.bit(i)` |
| L0541 | sort_desc_by_negation | `xs.sorted(by: x => -x)`; use `desc: true` |
| L0542 | last_via_len | `xs[xs.len - 1]` read; use `xs.last!` |
| L0543 | manual_pair_loop | `for j in i + 1..n` inside `for i in 0..n`; use `pairs()` |
| L0544 | manual_digit_loop | `while y > 0:` with `y /= 10`; use `y.digits()` |
| L0545 | hand_rolled_pow | a square-and-halve loop; use `pow_mod` or `Mod<P>` |
| L0546 | hand_rolled_gcd | `f(b, a % b)`; use `gcd` |
| L0547 | loop_argmax | `for i ..: if a[i] > a[j]: j = i`: `D.max(by: i => a[i])` names the index of the largest |
| L0548 | manual_mex | `while seen[m]: m += 1`: `(lo..=hi).find(m => not seen[m])` names the search |
| L0549 | copy_comprehension | `[x for x in a]`: `a.collect()` copies a sequence into a `List` |
| L0550 | queue_head_index | `var head = 0; while head < q.len: q[head]`: `bfs` or a `Deque` |
| L0551 | split_range_check | `x >= 0 and x < n`: chain it as `0 <= x < n` |
| L0552 | contains_then_add | `if k in m: m[k] += v else: m[k] = v`: `m[k] += v` starts a missing key at 0 |
| L0554 | memo_by_hand | `if k in memo: return memo[k]`: `@memo(map)` on the fn |
| L0555 | lca_by_hand | `while depth[u] > depth[v]: u = parent[u]`: `Rooted.lca` |
| L0556 | cross_by_hand | `xs[i] * ys[j] - xs[j] * ys[i]`: points as `Pt`, `p.cross(q)`, `area2` |
| L0557 | binom_by_hand | `fact[a] * inv(fact[b]) * inv(fact[a - b])`: `Comb<Z>(n).choose(a, b)` |
| L0558 | runs_by_hand | `if c == x: run += 1 else: run = 0`: `s.runs()` yields `(val, len)` |
| L0559 | diff_by_hand | `d[l] += x; d[r] -= x`: `Diff<D, T>()` with `add(l..r, x)` and `finish()` |
| L0560 | components_by_dfs | `for u ..: if not seen[u]: count += 1; visit(u)`: `components(adj)` |
| L0561 | trial_division | `while d * d <= x: if x % d == 0`: `divisors(x)` / `factor(x)` from `std.math` |
| L0562 | count_until | `while p < n: p *= 3; k += 1`: `(0..).find(k => 3 ** k >= n)!` names the smallest `k` |
| L0564 | read_loop | `for _ in 0..k: xs.push(read())`: `let xs: [0..k]T = read()` reads them all |
| L0565 | cycle_by_colors | `color[u] = 1; ... color[v] == 1 ...; color[u] = 2` in a DFS: `find_cycle(g)` |
| L0566 | sliding_min_by_hand | a monotone deque popped at the back and advanced past `i - k`: `Window(min)` / `Window(max)` |
| L0567 | depth_by_hand | `depth[v] = depth[par[v]] + 1`: `Rooted.from_parents(par).depth(v)` |
| L0568 | small_to_large_by_hand | `if a.len < b.len: (a, b) = (b, a)` then `for x in b: a.add(x)`: `a.absorb(mut b)` |
| L0569 | inversions_by_fenwick | `inv += i - bit.prefix(x); bit.add(x, 1)` in a loop: `inversions(xs)` |
| L0570 | interval_dp_by_length | `for len in ..: for l in ..: let r = l + len - 1`: `[I.ranges]` read in its order |
| L0577 | query_index_shift | a position read as `1..=n` and indexed as `a[l - 1]`: declare the array over `1..=n` |
| L0578 | tuple_points | `xs` and `ys` read as parallel coordinate arrays and indexed together: `[D] i => Pt(xs[i], ys[i])` |
| L0579 | counter_by_map | `cnt[x] = (cnt.get(x) ?? 0) + 1` on a `Map<K, Int>`: `Counter<K>` (`std.hashmap`), where a key it never saw reads 0 |
| L0580 | submask_loop | `while s > 0: ..; s = (s - 1) & m`: `for s in submasks(m)` (`std.math`) |
| L0581 | ternary_search_by_hand | a `while` loop probing `lo + (hi - lo) / 3` and `hi - (hi - lo) / 3`: `ternary_min` / `ternary_max` (`std.search`) |
| L0582 | argsort_loop | a list of `(a[i], i)` pairs sorted for their order: `(0..n).sorted(by: i => a[i])` |
| L0583 | palindrome_by_hand | `s[i]` against `s[n - 1 - i]` over `0..n / 2`: `is_palindrome(s)` (`std.strings`) |
| L0584 | monotone_stack_by_hand | a stack of positions popped while `a[st.last!] < a[i]`: `a.next_smaller(by: x => -x)` (`std.seq`) |
| L0585 | subset_masks_by_hand | `for mask in 0..(1 << n)` testing `(mask >> i) & 1` to pick `a[i]`: `for sub in a.subsequences()` (`std.seq`) |
| L0586 | adjacent_by_index | `if i + 1 < n and a[i] .. a[i + 1]` (or `v != V.last!`) in a loop over positions: `for (x, y) in a.adjacent` (`std.seq`) |
| L0587 | fact_tables_by_hand | `fact[i] = fact[i - 1] * i % P` in a loop: `Comb<Mod<P>>(n)` with `comb.fact(i)`, `comb.inv_fact(i)`, `comb.choose(a, b)` (`std.mod`) |
| L0588 | manual_heap_drain | `for _ in 0..k:` using `h.pop()!`: `for x in h.drain().take(k)` (`std.heap`) |
| L0589 | group_into_map | `for x in xs: (m[k] ??= []).push(v)`: `let m = xs.group(by: x => k, of: x => v)` (`std.seq`) |
| L0590 | sliding_sum_by_hand | `cur += a[i] - a[i - k]` over `k..n`: `Prefix(a)` and `p.fold(j - k..j)` (`std.prefix`) |
| L0591 | while_not_empty_pop | `while not q.is_empty:` then `let x = q.pop()!`: `while let x = q.pop():` |
| L0592 | manual_starts_with | `s[0..p.len] == p`: `s.starts_with(p)`; `s[s.len - p.len..] == p`: `s.ends_with(p)` |
| L0593 | test_count_loop | `let t = read()` then `for _ in 0..t:` around the program: `@!multitest` |
| L0594 | map_contains_then_index | `if k in m:` then `m[k]` in the branch: `if let v = m.get(k):`; `if k not in m: return` then `m[k]`: `let v = m.get(k) else: return` |
| L0595 | coalesce_test_then_unwrap | `while (e ?? c) > y:` then `let w = e!`: `while let w = e, w > y:` (and `if`) |
| L0596 | found_flag_pair | `var found = false` set to `true` only beside `best = x`: `var best: ?T = None` |
| L0597 | unreachable_code | statements after `return`, `break` or `continue` in the same block: delete them |
| L0598 | unused_binding | a local `let` in a `fn` whose name is never read: delete it, or `_` to keep the computation |
| L0599 | int_div_possible_zero | `a / b` or `a % b` where `b`'s declared range includes 0: guard it or narrow the range |
| L0302 | int_shift_overflow_at_bounds | `x << k` where `k`'s declared range lets the shift leave `Int`: narrow `k`, shift a `U64`, or use `Bitset` |
| L0303 | reversed_range_literal | `for i in n..0` where the declared start is always above the end: the loop never runs; reverse the range with `.rev` |
| L0304 | domain_empty_first_last | `D.first!` / `D.last!` where `D`'s declared bounds let it be empty: the unwrap traps on that input; guard with `if let` or tighten the bound |
| L0305 | unwrap_on_possibly_empty | `a.max()!`, `a.min()!`, `a.first!` or `a.last!` where `a`'s declared length can be 0: the unwrap traps on that input; handle `None` or tighten the bound |
| L0306 | int_overflow_at_bounds | `a * b`, `a + b` or `a ** k` whose operands' declared ranges let it leave `Int`, shown by an input that does: widen an operand to `I128`, use `Mod<P>`, or tighten the bounds |
| L0307 | always_true_bounds_check | `if x >= 0:` with `x: 0..=n`, or `n == 0` with `n: 1..=10`: the declared ranges settle the test, so it or its branch does nothing; drop it or widen the range |
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
