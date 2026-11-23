# Std intercepts (short-lived audit, 2026-11-22)

Every place the compiler matches a std call, method chain or std type and runs
hand-written code instead of the std body. Delete a row when its general fix
lands; delete the file at zero. Evidence: edge programs run with `cpl run`
against `cpl build` C++ at `-O2`, under ASan/UBSan, and with `--checks all`.

**Count: 21** found, 20 open (17 in IR lowering, 3 in codegen, 0 in the interpreter, which
only evaluates the IR forms below). Not counted: the storage, I/O, bits and
numeric floor (`abs`, `sqrt`, `floor`, `popcount`, `swap`, `len`, `nth`/`at`/`get`
on core types, `shuffle`, kept for its seeded stream), and `drop`/`take`/
`windows`/`pairs`, which already run std bodies (their traps are std's asserts).

One finding is general, not per row: Int overflow traps in `cpl run` but wraps
in the C++ at every level (`--checks all` and `-DLOCAL` too), for plain `a + b`
as for `sum`, `**` and `prefix`. Each row below agrees with plain arithmetic.
That is the judge-build traps question with Ryan, not an intercept bug.

| # | Matches (where) | Why | Interp vs C++ | General fix | Owner |
|---|---|---|---|---|---|
| 1 | `(a..b).sum()`, `(a..b).step(k).sum()` -> `Builtin::RangeSum` / `cpl::range_sum` (lower/call.rs `try_range_int_sum`) | perf: O(1) closed form | agree on empty, negative, `MAX-1..MAX` and `by <= 0`. **Missed here, found by coordinator:** a sum past `Int` (`(3..m).step(2).sum()`, m = 1e12) printed different wrong values in the two engines; compiler now computes both in 128 bits and traps (compiler worklog, `tests/parity/range_int_sum_overflow`) | `Range.sum` std body with the closed form (U64 math exists) | compiler (converting now) |
| 2 | free `sum`/`count`/`all`/`any`/`min`/`max` of a seq or generator, the methods, and `fold(min\|max)` -> `Reduce` loops (`reduce_opt`, `fold_seq`) | perf (fused loop), the generator form (decision 53), a struct's own `zero`/`+` | agree on empty, overflow, ties | std bodies over `Seq`; a generator passed to a std fn as a lazy seq, fused by inlining | compiler + stdlib |
| 3 | `argmax`/`argmin(gen)` (`arg_extreme`) | generator form | agree on ties (first wins) | same as 2: std `max_by` over the walk | compiler + stdlib |
| 4 | `first(gen)`, `xs.find(f)` (`lower_first`, `find_first`) | generator form, perf | not probed | same as 2 | compiler + stdlib |
| 5 | `sorted(xs[, by:, desc:])`, `sort(mut xs)` -> `Sort` / `cpl::sort_by`, `sort_cmp` | perf: `std::sort`/`stable_sort` | agree: stability, `desc` | keep one sort-in-place intrinsic (storage); `sorted` and `by:`/`desc:` as std bodies over it | compiler + stdlib |
| 6 | free `runs(seq)` (`lower_runs`) beside std.seq `runs` | history | agree | delete the lowering; std's body | compiler |
| 7 | `span(a, b)` (`lower_span`): positions or bounds, either order; std's takes two cuts and traps on `a > b` | history; two meanings | each engine agrees with itself; the two meanings differ (stdlib worklog) | stdlib picks one meaning, one std body | stdlib, then compiler |
| 8 | `counts(xs)` (`lower_counts`): hidden map type, missing key reads 0 | missing feature | agree (`c[7]` = 0) | std `Counter<T>` with a user-declarable default-on-missing index | stdlib + compiler |
| 9 | `prefix(a)` picks a `std.prefix` shape by the argument's shape (lower/prefix.rs `prefix_shape_struct`) | missing feature: overloading on argument shape | agree; overflow as plain `+` | type-directed overloads a user library can declare | compiler |
| 10 | `adjacency(edges[, into: Bitset])` (`lower_adjacency`, `instantiate_bitset`) | missing feature (output type chosen by argument) | not probed | std body in graph.cp; same overload mechanism as 9 | compiler + stdlib |
| 11 | `xs.map`/`filter`/`partition` on lists and arrays (`lower_map`/`lower_filter`/`lower_partition`) | perf (no closure call) | not probed | std bodies; devirt + inlining make the closure free | compiler + perf |
| 12 | list `first_ge`/`first_gt`/`last_le`/`last_lt`/`find(val)` (`seq_search_method`): `?Int`; std's are on arrays, `?D` | history | agree with each other; a negative element answers silently (std-prefix conformance case wants a trap) | one std body per search on `Indexed` | stdlib + compiler |
| 13 | `xs.reach(while:, cyclic:)` two-pointer -> `Builtin::Reach` / `cpl::reach` | perf, history | not probed | std body | stdlib + compiler |
| 14 | `xs.adjacent` (`lower_adjacent`): built as a list | history | not probed | std view body | stdlib + compiler |
| 15 | `gcd`, `min`, `max` named as fn values -> `BareOp` (lower/mono.rs) | perf (no fn value) | agree (`gcd(0, 0)`, negatives, `MIN`) | std fns as values; devirt makes them direct calls | compiler |
| 16 | free `bit(k)` lowering arm (call.rs `"bit"`) was dead: std's `Int.bit(self, i)` takes the name | history | both reject `bit(k)` (E0303) | deleted with this table's fix commit; delete the row when no doc names `bit(k)` | done |
| 17 | `Map`/`HashMap`/`Set`/`HashSet` constructors and types by name (call.rs, mod.rs, mono.rs, stmt.rs; 17 sites in `backend_std_names`) | std `HashMap` mapped onto the core map | not probed | `HashMap` is a std struct over core storage, or a core type with no std name | compiler |
| 18 | `Bitset` by generic name in lowering (`is_bitset_of`, `instantiate_bitset`) | serves 10 and 19 | - | goes with 10 and 19 | compiler |
| 19 | `(a & b ...).count`/`.len`/`.is_empty` on std `Bitset` -> one word loop, matched by mangled fn-name suffix `_count`/`_bitand`... (codegen expr/builtin.rs `try_fused_bitset_op`) | perf: no temporaries | agree (`count` 17/67, `is_empty`) | general temporary elision (expression fusion over any struct of words), or std `count_and(a, b)` | perf |
| 20 | std `Bitset` over a small static domain -> fixed `std::array` words, matched by struct name `Bitset` (codegen scope.rs, names.rs) | perf | not probed | a fixed-size array type std can declare (`[K]U64` over a const domain) | perf + compiler |
| 21 | `IntervalMap.assign` -> `cpl::interval_assign` in C++ only; the interpreter runs the std body (codegen emit.rs `is_interval_map_assign`) | perf: one map iterator | agree on 2,000 random assigns with `on_old`, vs an array | ordered-map cursor API (seek, erase range) so the std body is as fast | perf + stdlib |
