# Rebuild round 3: holes

I read the spec at commit `e04eb0d70` of `ryanbai1412/cp-lang`, and only `spec/`: `README.md`, `core/`, `lib/`, `tools/`, `std/` and `diagnostics.md` (plus `GLOSSARY.md`, which `spec/README.md` makes normative). I did not open `spec/delta/`, `spec/conformance/`, `spec/idioms/` or anything outside `spec/`, and I did not run the compiler. Each hole below is a place where writing one of the ten programs needed a guess.

## Summary

There are 21 holes: 7 silent, 11 contradiction, 3 missing-stdlib.

The ten most likely to hit a contestant, most likely first:

1. H-RANGE-ELEMENT-ACCUMULATOR: a `Fenwick` or `kruskal` total over range-typed input (`1..=1e9`). It hits every sum structure fed by a typed read.
2. H-LITERAL-TO-MOD: `0` and `1` where a `Mod<P>` is expected. It hits every counting DP.
3. H-FENWICK-SUM-OR-FOLD: `fw.sum(r)` or `fw.fold(r)`. It hits every Fenwick query.
4. H-HOIST-RUNTIME-DOMAIN: does `domain V = 1..=read()` hoist? It hits every program that reads its sizes.
5. H-MAP-INDEX-ABSENT: does `m[k] = v` insert or trap? It hits every map-based solution.
6. H-FIND-VS-FIRST: is the first element passing a test `find(p)` or `first(p)`? It hits every search.
7. H-GENERATOR-EXPRESSION: `sum(f(x) for x in xs)` against the grammar. It hits every fold of a function.
8. H-MAP-NO-STUB: the `Map` API is guessed.
9. H-PRINT-TRAIT-NAME: `Print` or `Show`. It hits anyone printing a library type or writing a `Print` impl.
10. H-MODULUS-TYPE-ARG: is `Mod<1e9 + 7>` legal?

## Silent

### H-HOIST-RUNTIME-DOMAIN

- **Question:** R11.1.1 and R06.1.7 say items hoist within their block, and the grammar (core/01 §1.10, `item`, `domain_decl`) makes `domain V = expr` an item. Does `domain V = 1..=read<1..=2e5>()` at top level hoist above earlier `read()` statements, and so reorder the input? Does an `enum` whose fields use the runtime `V` hoist above `V`?
- **Readings:** (a) runtime-count domains run in place, like statements; (b) every item hoists, so reads reorder; (c) only a domain with a constant count hoists, matching the library-file rule in R11.1.2.
- **Chose:** (a), the shape of the core/00 example. I declared each enum after its domain.
- **Cost if wrong:** under (b), tokens are read in the wrong order and the program gives wrong answers or a parse trap on line 1.

### H-METHOD-NEEDS-IMPORT

- **Question:** does a method call on a free `self` fn (`xs.sorted()`, `(R, C).find(p)`, `.map`, `.adjacent`) need that fn's name in scope? `std/prelude.cp` lists `sum`, `count`, `adjacent` and others from `std.seq`, but not `sorted`, `sort`, `find`, `first`, `map`, `filter` or `min`.
- **Readings:** (a) methods resolve through the receiver's type, with no import needed; (b) only names in scope resolve, so `a.sorted()` needs `use std.seq.{sorted}`.
- **Chose:** (a). Under (b), every chapter example breaks.
- **Cost if wrong:** E0201 on the most common calls; each has a one-line `use` fix.

### H-PATHS-INDEX

- **Question:** lib/graphs-trees.md (Shortest paths table) reads distances as `p[v]`. In std/graph.cp, `struct Paths` has only `pub fn get(self, v: V)` and no `at` / `Indexed` / `IndexMut` impl, so nothing makes `p[v]` legal (core/12 W4 and W8 define `[]` through `at`).
- **Readings:** (a) `p[v]` works, through some implied `get`→`[]` rule; (b) you must write `p.get(v)`.
- **Chose:** (a), `monster[p]` in 04-monsters.
- **Cost if wrong:** a compile error with an obvious fix.

### H-RANGE-ELEMENT-ACCUMULATOR

- **Question:** when the input is typed by the statement's bound (`read<[V]1..=1e9>()`, roads `w: 1..=1e9`), what type does a sum-based structure accumulate in? `Fenwick(a)` infers `T = 1..=1e9`, so `T.zero` lies outside the range and sums exceed it. `kruskal` returns `total: W` with `W` the record's `w` type and starts from `var total = W.zero` (std/graph.cp:2802-2815). core/03 R03.1.7 answers this for `SegTree(+, a)` only (nodes widen to `Int`). R04.4.24 says arithmetic on range types yields `Int`, but a generic `T` or `W` parameter is not arithmetic.
- **Readings:** (a) generic sum parameters widen to `Int` as in R03.1.7; (b) `T` / `W` is the range type, so every store is a range store; (c) the contestant must read the values as `Int`, or write `Fenwick<I, Int>(...)`.
- **Chose:** (a), in 05, 07 and 09.
- **Cost if wrong:** under (b), checked builds trap on the first add (R0201) and release truncates to 4 bytes, a silent wrong answer on sums above `4.29e9`. This is the most likely wrong answer in the set.

### H-MAP-DEFAULT-KIND

- **Question:** for a plain lookup table, is the idiomatic type `Map<K, V>` or `HashMap<K, V>`? lib/containers.md §1.1 says `Map` lowers to `std::map`, or to a salted unordered map when order is never observed. §1.7 and §2.8 present a builtin `HashMap` as "the default for arbitrary hashable keys". core/02 and core/07 mention only `Map`.
- **Readings:** (a) `Map`, letting the compiler pick hashing; (b) `HashMap`, spelled because the user wants hashing.
- **Chose:** (a).
- **Cost if wrong:** speed only, O(log n) against O(1) expected; it matters at 2e5 lookups under tight limits.

### H-RECORD-PATTERN-SHORTHAND

- **Question:** the expression shorthand `(total:, take:)` exists (std/graph.cp `kruskal` body, `(C:, comp: ...)` in `components`), but the pattern grammar (core/01 §1.10 `pattern`) requires `NAME ':' pattern`. Can I write `let (total:, take:) = kruskal(es)` or `for (u:, v:) in edges`?
- **Readings:** (a) the shorthand works in patterns too; (b) only `(total: total, take: take)` or field access works.
- **Chose:** field access (`mst.total`, `r.u`).
- **Cost if wrong:** a parse error (E0106) if a contestant writes the shorthand.

### H-MODULUS-TYPE-ARG

- **Question:** can a type argument be an integer expression: `Mod<1e9 + 7>`, `Mod<998_244_353>`, `Mod<P>` with `const P = 1e9 + 7`? R01.4.1 makes `1e9 + 7` an exact `Int`, std/mod.cp's header uses the bare literal `Mod<998244353>`, and the grammar never defines `generic_args`.
- **Readings:** (a) any constant integer expression; (b) only a literal or a `const` name.
- **Chose:** a plain literal, `Mod<1_000_000_007>`.
- **Cost if wrong:** a compile error on the most common line of a counting problem.

## Contradiction

### H-LITERAL-TO-MOD

- **Question:** does an integer literal convert where a `Mod<P>` is expected (`var ways: [0..=n]Z = 0`, `ways[0] = 1`)?
- **Sources:** core/04-indices.md R04.4.22 (line 158) says the implicit conversions are a "closed list, none user-extensible", and the list (R04.4.23-27) has no literal-to-`Mod`. std/mod.cp:62-65 (`from_int`) says "This is the conversion an `Int` literal takes where a `Mod<P>` is expected, so `let x: Mod<7> = 10` is `Mod<7>.from_int(10)`".
- **Readings:** (a) the literal converts (mod.cp); (b) it doesn't (core), so you write `Z(0)`.
- **Chose:** explicit `Z(0)` and `Z(1)`, which are correct under both readings.
- **Cost if wrong:** under (b), the natural `= 0` is a type error in every modular DP. Under (a), nothing breaks, but the explicit form is noise.

### H-FENWICK-SUM-OR-FOLD

- **Question:** what is a Fenwick range query called?
- **Sources:** std/README.md:172 ("`t.sum(r)` where the operation is addition … (`Fenwick`, `Prefix`)"), lib/ops-ranges.md:365 and :457 ("`sum(r)`"), and std/fenwick.cp:10 ("`add`, `prefix` and `sum` are O(log n)") say `sum`. The only method in std/fenwick.cp is `fold` (:66).
- **Readings:** (a) `sum(r)`; (b) `fold(r)`.
- **Chose:** (b), the stub.
- **Cost if wrong:** E0203 on every Fenwick query.

### H-FIND-VS-FIRST

- **Question:** what is the first element that passes a test called?
- **Sources:** core/07-sequences.md R07.9.5 (line 275), the R04.2.20 table in core/04 (`V.find(f)`) and std/seq.cp:917 (`find(p) -> ?T`) say `find(p)`. std/README.md:126-127 says "`find(x)` is the index of a value; the element a test holds for is `first(p)`", and std/seq.cp:189 has `first` as a predicate-free getter.
- **Readings:** (a) `xs.find(p)`; (b) `xs.first(p)`.
- **Chose:** (a), in 04-monsters.
- **Cost if wrong:** a compile error, or under the README reading, `find` looks for a value and silently has the wrong type.

### H-GENERATOR-EXPRESSION

- **Question:** how do you sum a function over a range?
- **Sources:** std/README.md:126 and :232-234 say "`sum`, `prod`, `min` and `max` of a function stay `sum(f(x) for x in xs)`" and give `count(x > 2 for x in xs)`. The grammar (core/01 §1.10, `arg` at line 404, `primary`) has no bare generator expression; comprehensions are bracketed only. std/seq.cp:35 offers a third spelling, `xs.sum(of: f)`.
- **Readings:** (a) the free generator form; (b) `xs.map(f).sum()` (core/07); (c) `xs.sum(of: f)`.
- **Chose:** (b), the only one the grammar surely parses (02-dice).
- **Cost if wrong:** a parse error, or a taste lint (L0509) on whichever form is not canonical.

### H-MAP-INDEX-ABSENT

- **Question:** what does `m[k]` do when `k` is absent, and does `m[k] = v` insert?
- **Sources:** std/README.md:143 says "`m[k]` traps when `k` is absent and `m.get(k)` is `?V`". core/12-powers.md W8 (line 207) says "`Map` uses `at_mut` and inserts the default (decision 128)", and core/13-lowering.md:176 says `m[k] += 1` on `Map` "inserts default".
- **Readings:** (a) a write inserts and a read traps; (b) both insert a default; (c) both trap, so inserting needs another verb.
- **Chose:** (a) for `seen[a[i]] = i` (03).
- **Cost if wrong:** under (c), a trap on the first insert; under (b), reads of absent keys silently give `0`-like defaults.

### H-PRINT-TRAIT-NAME

- **Question:** what trait makes a type printable?
- **Sources:** core/10-io.md:84, :86, :92 and §10.5 (lines 120-127) use `Print` (`fn print(xs: Print..., ...)`). Every stdlib impl uses `Show`: std/mod.cp:148 (`impl Show for Mod<P>`), plus algebra.cp:100, bitset.cp:1202, geometry.cp:183 and math.cp:1462. Neither trait is declared anywhere in spec/.
- **Readings:** (a) they are one trait under two names; (b) `Mod` is not `Print`, so `print(ways[n])` fails.
- **Chose:** (a), printing `Mod` directly (02).
- **Cost if wrong:** a compile error printing any library value, and a user `impl Print` / `impl Show` that the printer ignores.

### H-RUNTIME-MODULUS-SPELLING

- **Question:** how do you spell a type for a modulus read at runtime? (I hit this while choosing the constant form for 02.)
- **Sources:** core/02-values-types.md:206 (`type Z = Mod(read<Prime>())`), core/09-contexts-checks.md:35, diagnostics.md:81 (E0801 fix: `type Z = Mod(p)`) and core/13-lowering.md:40 use parentheses. std/mod.cp:6 and lib/math.md:174 use `let P: Int = read(); type Z = Mod<P>`, which also breaks R01.3.2 (E0208: values are lowercase). core/03-numbers-layout.md:276 says `Mod<p>`.
- **Readings:** (a) `Mod(p)`; (b) `Mod<p>`; (c) `Mod<P>` with an uppercase value.
- **Chose:** not needed for 02, which uses the constant `Mod<1_000_000_007>`. I'd write (b).
- **Cost if wrong:** a compile error. Under (c) the error comes from the casing rule, which hides the real question.

### H-REPEAT-LOOP

- **Question:** is there a `repeat k:` loop?
- **Sources:** lib/search-dp.md:76 writes `repeat k - 1: best = layer(best)`. `repeat` is not among the 33 keywords (core/01 lines 90-92), not a contextual word (core/01 table after line 103), and not in the grammar's `compound` statements.
- **Readings:** (a) `repeat` exists; (b) write `for _ in 0..k:`.
- **Chose:** (b), for the query loops in 05 and 07.
- **Cost if wrong:** a parse error on a very common loop.

### H-MIN-ARITY

- **Question:** does `min` take more than two arguments?
- **Sources:** core/11-modules-prelude.md:59 (R11.3.3) says "`min` and `max` stay the binary builtins". std/README.md:126 says the free form is for "two or more values (`min(a, b)`)".
- **Readings:** (a) binary only; (b) variadic.
- **Chose:** (a), nested `min(min(x, y) + 1, z)` in 06.
- **Cost if wrong:** wordier code under (a), and E0303 under (b) when someone writes `min(x, y, z)`.

### H-DSU-UNITE-DROPPED

- **Question:** is it fine to call `d.unite(a, b)` as a statement?
- **Sources:** core/06-control-functions.md R06.4.8 (line 139) says dropping a non-unit result is the unused-result lint (W0101) unless the fn is `@droppable`; `Dsu.unite` returns `Bool` and is not `@droppable` (no `@droppable` appears anywhere in spec/std). The stub's own examples drop it: std/dsu.cp:48 `d.unite(2, 3); d.find(3) == 2`.
- **Readings:** (a) it warns and you write `let _ = ...`; (b) it is fine.
- **Chose:** a bare statement (01).
- **Cost if wrong:** a warning on the most common DSU line; no wrong answer.

### H-COMPONENTS-NOT-IN-PRELUDE

- **Question:** is `components(g)` in the prelude?
- **Sources:** core/11-modules-prelude.md R11.3.2 says every free function in a chapter's main tables is in the prelude, and lib/graphs-trees.md:202 lists `components(g)` in the Structure table. std/prelude.cp:23 imports `condense`, `scc`, `topo` and others from `std.graph`, but not `components`.
- **Readings:** (a) prelude; (b) it needs `use std.graph.{components}`.
- **Chose:** neither. 01 uses `Dsu`, which is in the prelude.
- **Cost if wrong:** E0201 with an import fix.

## Missing stdlib

### H-MAP-NO-STUB

- **Question:** what is the `Map` / `HashMap` API? These are "builtin" (lib/containers.md §1.1, §1.7) with no stub in spec/std; std/GAPS.md:49 and :69 say the bodies "wait on the `Map` stub". I had to guess the empty literal (`[:]` from R01.4.5 with an annotation, or `Map<Int, I>()`), `get(k) -> ?V` (only seen in core/01 §1.7 `m.get(k)` and std/README.md:143), and the insert spelling.
- **Readings:** listed above; for insert, `m[k] = v` or `m.insert(k, v)` (std/seq.cp `insert` is `List`-only).
- **Chose:** `var seen: Map<Int, I> = [:]`, `seen.get(k)`, `seen[k] = v` (03).
- **Cost if wrong:** compile errors on any map program. No doc says what each operation costs.

### H-PRELUDE-FIND-DANGLING

- **Question:** where does the prelude's `find` come from? std/prelude.cp:92 onward imports `find` from `std.strings`, but std/strings.cp has no top-level `pub fn find`, only struct methods (lines 2266, 2834). std/seq.cp:917's `find(p)` is not in the prelude.
- **Readings:** (a) `find` resolves to seq's linear scan; (b) it resolves to a string search that doesn't exist; (c) E0201.
- **Chose:** (a) for `(R, C).find(...)` (04).
- **Cost if wrong:** a confusing error, or the wrong overload of a name that core/02 R02.11.3 says can't be overloaded.

### H-GRID-PATH-LETTERS

- **Question:** how do you turn a grid path into move letters? std/grid.cp has `Dir4.from(letters, ch)` (letter → direction) but no inverse (direction → letter) and no "direction from `p` to its neighbour `q`". Paths of cell positions (`Paths.path`) carry no direction.
- **Readings:** (a) `zip(Dir4.all(), "URDL")` and test `p.step(d) == Some(q)`; (b) compute deltas from `Int(q.0) - Int(p.0)` by hand; (c) read the private field `d.k` and index `"URDL"`.
- **Chose:** (a) (04).
- **Cost if wrong:** no compile error, but a common silent bug: letters listed in the wrong order (`"UDLR"` against `Dir4.all()`'s up, right, down, left) give a wrong path.
