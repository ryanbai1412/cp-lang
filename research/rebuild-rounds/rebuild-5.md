# Rebuild round 5: holes

I read the CP Lang spec at commit 66f8d9c93 and nothing else: `spec/README.md`, `spec/core/`, `spec/lib/`, `spec/tools/`, `std/` and `spec/diagnostics.md`. I did not open `plan/domains/spec/delta/`, `spec/conformance/`, `spec/GLOSSARY.md`, `research/archive/spec-status.md`, `spec/idioms/`, `spec/lints/` or anything outside `spec/`, and I did not run the compiler. The ten solutions (`01-labyrinth.cp` to `10-planets-and-kingdoms.cp`) are what the text says is idiomatic. Wherever I had to guess, even when the intent was clear, the guess is logged below.

## Summary

There are 31 holes: 9 silent, 18 contradiction, 4 missing-stdlib.

The ten most likely to hit a contestant, most likely first:

1. H-RANGE-WEIGHT-SUM: path sums over `w: 1..=1e9` arcs are stored back into `?W` slots.
2. H-MULTI-NAME-READ: `let n, m: T = read()` may read once and give both names one value.
3. H-FRESH-DOMAIN-BASE: no rule says what `Int(c)` is for a domain returned by `scc`, `components` or `compress`.
4. H-SCAN-RESULT: `xs.scan(op)` is a `Prefix` in one chapter and a lazy generator in another.
5. H-SEGTREE-TYPE-PARAMS: two different meanings for `SegTree<...>` type arguments, and the node width under `act:` is unclear.
6. H-DROP-RESULT: no stated way to discard a `Bool` result such as `floyd_warshall`'s.
7. H-DOMAIN-COUNT-NAME: `V.count`, `V.len` and `V.K` all mean the number of positions.
8. H-ROOTED-ORDER: `t.order`, the tree-DP idiom, has no stub.
9. H-SEGTREE-POSITIONAL: `SegTree(act:, a)` puts a positional argument after a named one.
10. H-APPLY-TAG: range add is written `st.apply(r, add(1))` in one place and `apply(r, x)` in another.

## Silent

### H-MULTI-NAME-READ
- Question: Does `let n, m: 1..=2e5 = read()` call `read()` once per name, or once in total?
- Readings: (a) once per name, so it reads two tokens; (b) once, with both names bound to that value. R01.6.1 says `var a, b: Int = 0` "fills each with the value", which supports (b).
- Chose: avoided the form and read headers as tuples: `let (n, m) = read<(1..=1000, 1..=1000)>()`.
- Cost if wrong: under (b), `n == m` on every test and the answers are silently wrong. It hits the first line of most programs.

### H-TOP-LEVEL-RUNTIME-TYPES
- Question: May a top-level `enum` whose payload uses a runtime domain (`Ask(k: V)` with `domain V = 1..=n`) be declared in a program, and where?
- Readings: R01.9.1 allows types inside a *function body* that use its runtime domains. R11.1.1 says items hoist but runtime `domain`s do not, so a hoisted enum could reference `V` before `V` exists. The rules do not cover a program's top level.
- Chose: declared `enum Query` after `domain V` (05) and assumed it is not hoisted above it.
- Cost if wrong: a compile error. The fix is moving the enum into a nested fn, or using two plain reads.

### H-ENUM-RANGE-FIELD
- Question: Can a `Range<V>` be an enum payload field that `read<Query>()` fills from `l r`?
- Readings: R10.2.3 says `Range<T>` reads two ends as `l..=r` and that an enum reads its variant's fields. R03.1.3 says struct fields need static bounds and allows positions of `V`, but says nothing about `Range<V>`.
- Chose: `@tag(1) Add(at: Range<V>, u: 1..=1e9)` (05).
- Cost if wrong: a compile error. The fix is two `V` fields and `l..=r` at the use site.

### H-FRESH-DOMAIN-BASE
- Question: What does `Int(c)` give for a position of a fresh domain returned by a library call (`let (domain K, kingdom) = scc(g)`), and how does such a position print?
- Readings: (a) 0-based, because the `scc` body declares `domain C = 0..k`, but the caller cannot see that; (b) a counted domain, so `io.base` applies (R10.2.2); (c) unspecified.
- Chose: (a). I print `Int(kingdom[v]) + 1` (10).
- Cost if wrong: every label is off by one, which is a wrong answer on test 1. The same question applies to `components`, `compress`, `two_edge_components` and `blocks`.

### H-DROP-RESULT
- Question: How do you call a function only for its effect when it returns a `Bool` (e.g. `floyd_warshall(mut d)` with non-negative weights)?
- Readings: (a) a bare call, which is lint W0101 (a warning, R06.4.8); (b) `let _ = f()`, which no rule says silences W0101; (c) `assert f()`, which is a trap: asserts are skipped in release (R09.3.7), so the call would vanish on the judge.
- Chose: (b), `let _ = floyd_warshall(mut d)` (04).
- Cost if wrong: (b) may still warn. A contestant who picks (c) gets correct local runs and wrong answers on the judge.

### H-ARRAY-WIDEN
- Question: Does a declared type widen an array element-wise, as in `var d: [V, V]?Int = g.matrix()` where `matrix()` returns `[V, V]?(1..=1e9)`?
- Readings: R04.4.22-24 list the implicit conversions and include "a range type to a wider one", but only for scalars. Arrays are not mentioned.
- Chose: annotated the array as `?Int` (04), as the escape from H-RANGE-WEIGHT-SUM.
- Cost if wrong: a compile error. The fallback is reading `w: Int` and losing input validation.

### H-FN-AS-NEXT
- Question: How does a plain nested generator fn `fn moves(p: (R, C)) -> yields (to: (R, C), ch: Char)` satisfy `trait Next<V>`, which declares a method `out`?
- Readings: `lib/graphs-trees.md` says "any fn from a vertex to a `Seq` of arcs" is a `Next`. Neither `core/08` nor `core/12` (W7) shows a fn value implementing a trait.
- Chose: passed `moves` directly to `bfs` (01).
- Cost if wrong: a compile error. The fix is a lambda `p => moves(p)` or a materialized `[Cell]List` of arcs.

### H-ALIAS-CONSTRUCT
- Question: With `type Z = Mod<1e9 + 7>`, is `Z(a)` a valid way to build a residue from an `Int`?
- Readings: `std/mod.cp` documents `Mod<P>(x)`. Nothing says whether a `type` alias can be called. The implicit `Int` to `Mod<P>` conversion (R04.4.28) needs an expected type.
- Chose: `Z(a) ** e` (09).
- Cost if wrong: a compile error. The fix is `let base: Z = a`.

### H-DEPENDENT-HEADER-BOUNDS
- Question: How do you bound a header token by an earlier token on the same line (CSES gives `m <= n(n-1)/2`)?
- Readings: a tuple read fixes every bound statically; two separate reads allow `read<0..=n * (n - 1) / 2>()`, but that splits the header.
- Chose: static loose bounds (`1..=250_000`) in a tuple read (04).
- Cost if wrong: weaker input validation only. Answers are unaffected.

## Contradiction

### H-RANGE-WEIGHT-SUM
- Question: When arcs carry `w: 1..=1e9`, what type holds a path distance?
- Readings: R04.4.24 (`core/04-indices.md`) says a path distance built from range weights "is `Int` unless declared narrower". `std/graph.cp` `Graph.matrix` (line 269) returns `[V, V]?W` with `W` the arc's `.w` type, and `floyd_warshall` (line 1367) stores sums back into `?W`. A store above `1e9` traps in checked builds and truncates to 4 bytes in release (R03.1.4, R09.3.3).
- Chose: forced `var d: [V, V]?Int = g.matrix()` (04); see H-ARRAY-WIDEN.
- Cost if wrong: silent wrong answers on the judge for every shortest-path problem that reads validated weights.

### H-SCAN-RESULT
- Question: What does `xs.scan(op)` return?
- Readings: `core/07-sequences.md` line 294 says a `Prefix<D, T>` of n + 1 exclusive folds indexed by cuts. `lib/ops-ranges.md` §6 line 340 says "the running folds, lazily: a generator". `std/prefix.cp` line 31 agrees with core.
- Chose: avoided it and wrote a running `best max= now` loop (03).
- Cost if wrong: a type error, or an off-by-one between exclusive and inclusive folds in sweeps.

### H-SEGTREE-TYPE-PARAMS
- Question: What do explicit type arguments on `SegTree` mean, and what node type does `SegTree(a: read<[V]1..=1e9>(), act: add)` get?
- Readings: R03.1.7 (`core/03-numbers-layout.md` line 97) writes `SegTree<0..=4e9>(+, a)`, with the first argument as the node type, and its §1 table says that with `act:` "nodes stay `Int`". `std/segtree.cp` line 16 declares `SegTree<D, T, F>` (domain first) and its examples write `SegTree<0..4, Int>`. `new(a: ?[D]T)` binds `T` to the element type `1..=1e9`.
- Chose: no type arguments, trusting R03's "act: keeps `Int`" (05).
- Cost if wrong: under the stub reading, sums above `1e9` truncate in release, a silent wrong answer.

### H-SEGTREE-POSITIONAL
- Question: How do you build a tag-only tree ("range apply, point read")?
- Readings: `lib/ops-ranges.md` §8 line 384 writes `SegTree(act:, a)`, and `std/segtree.cp` line 269 has the example `SegTree(act: assign, [1, 2, 3])`. Both put a positional argument after a named one, which R01.10.1 forbids. Read positionally, `a` would also fill the first parameter, `op`.
- Chose: `SegTree(a: read<[V]1..=1e9>(), act: add)` (05).
- Cost if wrong: a compile error, or the array bound to `op`.

### H-SEGTREE-NO-OP
- Question: Does `SegTree` with `act:` and no `op` work?
- Readings: the `new` doc (`std/segtree.cp` line 31) says it applies tags and reads points. The body (line 46) does `Monoid<T>.of(op!)`, which traps when `op` is `None`.
- Chose: the doc (05).
- Cost if wrong: a trap at construction. The fix is passing `+` as `op`.

### H-APPLY-TAG
- Question: What is the tag argument of a range add?
- Readings: `core/01-lexical-grammar.md` §1.7 line 234 writes `st.apply(l..r, add(1))`, and `lib/graphs-trees.md` writes `t.apply_path(a, b, lazy, add(x))`. `std/segtree.cp` line 124 declares `apply(r: Range<D>, f: F)` with `F = Int`, and `lib/ops-ranges.md` §4 says `add` acts as `x + f * n` with `f` the bare number.
- Chose: the bare number, `st.apply(at, u)` (05).
- Cost if wrong: a compile error.

### H-DOMAIN-COUNT-NAME
- Question: What is the number of positions of a domain called?
- Readings: R04.2.2 says `V.count`. `std/graph.cp` line 90 uses `V.len` (`pub type Tree<V> = [0..V.len - 1](u: V, v: V)`). `topo` in `std/graph.cp`, `std/fenwick.cp` line 23 and `std/README.md` line 215 use `D.K`. The stubs also index with `u.to_idx` and `b.ord`, which no core chapter defines.
- Chose: `V.count` (10), and plain `read<[n - 1](u: V, v: V)>()` instead of `Tree<V>` (06).
- Cost if wrong: a compile error. A contestant who copies the stubs hits E0203.

### H-RANGE-END-NAMES
- Question: What are the ends of a range called?
- Readings: R04.4.15 says `r.l` and `r.r`, and `core/04-indices.md` line 329 explicitly rejects `.start` and `.end`. `std/fenwick.cp` line 67 implements `fold` as `self.prefix(r.end) - self.prefix(r.start)`.
- Chose: `.l` / `.r` (not needed in the ten files).
- Cost if wrong: a compile error for anyone who copies the Fenwick code.

### H-CUTS-VALUE
- Question: Is `D.cuts` a domain or a range value, and how do you name all positions?
- Readings: R04.2.4-5 say `V.cuts` is a domain, `V.range` is the whole range and `for v in V` walks positions. `std/README.md` line 214 says "`D.cuts` is the value `Range<D.cuts>`… `D.all` is the positions".
- Chose: core (`for v in V`, `[v for v in V]`).
- Cost if wrong: a compile error on `D.all`.

### H-NEXT-METHOD
- Question: What is the method of `trait Next<V>` called?
- Readings: `core/08-traits-generics.md` §8.1 declares `fn next(self, v: V) -> yields Arc`. `lib/graphs-trees.md` and `std/graph.cp` line 28 declare `fn out(self, u: V) -> yields Self.Arc`, where `next` means target vertices (`g.next(u)`).
- Chose: `out`; I wrote no impl, only a closure (01).
- Cost if wrong: a user graph type that implements `next` does not satisfy `Next`.

### H-TURBOFISH
- Question: How do you pass a type argument to a lowercase generic fn?
- Readings: R01.3.4 says `read<[n]Int>()` with no turbofish. The `topo` doc in `std/graph.cp` (lines 519-520) says "pass the domain: `topo::<V>(u => ...)`".
- Chose: no type arguments; `topo(g)` on a `Graph` (08).
- Cost if wrong: a syntax error for closure graphs.

### H-SORT-DESC-FLAG
- Question: Is descending order a flag or a key?
- Readings: `core/07-sequences.md` line 324 says "Descending is a key, not a flag: `by: x => desc(x.score)`". `std/seq.cp` line 226 declares `sort(..., desc: Bool = false)`.
- Chose: not needed (an ascending `.sorted()` in 03).
- Cost if wrong: low; both may compile, but there are two spellings for one idea.

### H-MOD-CONSTRUCTOR
- Question: Does `Mod<P>(x)` reduce `x`, or store it as is?
- Readings: R02.6.2-3 say a generated constructor `T(args)` stores its arguments and a custom one is a static fn. `std/mod.cp` line 23 says "`Mod<P>(x)` calls `new`", while `new` itself (line 30) is `Mod(x % P)`, which would recurse under that reading.
- Chose: it reduces (09). The value is in `1..=1e9` either way.
- Cost if wrong: an unreduced residue breaks `==` and printing for inputs `>= P` or `< 0`.

### H-CONST-CASE
- Question: Can a constant be named in uppercase (`const P = 1e9 + 7`, `Z.MOD`)?
- Readings: R01.3.2 says an uppercase value binding is an error. R01.3.3 says "a capitalized name is a variant or constant". The grammar comment (`core/01-lexical-grammar.md` line 409) writes "`Mod<P>` with `const P`". `std/mod.cp` line 17 declares `let MOD = P` and documents `Z.MOD`.
- Chose: `type Z = Mod<1e9 + 7>` and `Z.MOD - 1` (09), avoiding a user `const`.
- Cost if wrong: E0208 on the modulus line, the most common line in counting problems.

### H-BORDERS-PRELUDE
- Question: Is `borders` in the prelude?
- Readings: R11.3.2 says every free fn in a chapter's main tables is in the prelude, and `lib/strings.md` §3.2 line 167 lists `borders`. `std/prelude.cp` imports `period` and `z_function` but not `borders`.
- Chose: `z_function` (07), which is in the prelude.
- Cost if wrong: E0201 on `borders(s)`; the fix is `use std.strings.{borders}`.

### H-CHILDREN-SHAPE
- Question: Is `t.children(v)` indexed?
- Readings: `lib/graphs-trees.md` line 255 says it is an "`Indexed` view of `v`'s children". `std/tree.cp` line 263 declares `-> yields V` (walk-only).
- Chose: not needed (06 uses `t.parent`).
- Cost if wrong: `t.children(v)[0]` or `.len` fails to compile.

### H-RULE-ID-DUP
- Question: Which rule is R04.4.28?
- Readings: `core/04-indices.md` uses R04.4.28 twice: for `Int` to `Mod<P>` (line 184) and for integer ranges as domains (line 194).
- Chose: cite by line.
- Cost if wrong: low; diagnostics and reviews cite the wrong rule.

### H-POW-SPELLING
- Question: What is the fast power of a `Mod<P>`?
- Readings: `lib/ops-ranges.md` §6 line 341 says `pow(x, k, op = *)` "is the one fast power" for numbers and `Mod<P>`. R01.11.1, `lib/math.md` and `std/mod.cp` (`impl Pow`) say `x ** k`.
- Chose: `Z(a) ** e` (09).
- Cost if wrong: low, two spellings; see H-POW-GENERIC for `pow` itself.

## Missing stdlib

### H-ROOTED-ORDER
- Question: Where is `t.order`, the parents-before-children walk that `lib/graphs-trees.md` uses for "tree DP without recursion"?
- Readings: the lib table lists `t.order` (O(1)) and shows `for v in t.order.rev`. `std/tree.cp` `Rooted` has no `order` member (it has a private `bfs` list).
- Chose: `for v in t.order.rev` (06), the documented idiom.
- Cost if wrong: E0203. The fallback is `t.up(...)` or a walk over `t.Pre.rev`.

### H-SEGTREE-INDEX
- Question: Does `st[k]` read a point of a `SegTree`?
- Readings: `lib/ops-ranges.md` §7 and `core/01` §1.7 (`print(st[i])`) say yes. `std/segtree.cp` has `get(i)` and `set(i, x)` but no `Index` impl or place.
- Chose: `print(st[k])` (05).
- Cost if wrong: E0305; the fix is `st.get(k)`.

### H-DIR-LETTER
- Question: How do you turn a grid step into its letter (`U`, `R`, `D`, `L`) for path output?
- Readings: `std/grid.cp` has `Dir4.from(letters, ch)` (letter to direction) but nothing for direction to letter, and `Dir4`'s `k` field is not public.
- Chose: iterate the letters and carry each one on the arc: `for ch in "URDL": ... p.step(Dir4.from("URDL", ch)!)` and `yield (to: q, ch:)` (01).
- Cost if wrong: none for correctness; it adds a few tokens to every grid-path problem.

### H-POW-GENERIC
- Question: Where is the generic `pow(x, k, op:)`?
- Readings: `lib/ops-ranges.md` §6 documents it for matrices, permutations and `Affine` maps. `std` has only `std/poly.cp`'s `pow` over power series, and `std/prelude.cp` imports no `pow`.
- Chose: `**` on `Mod` (09).
- Cost if wrong: E0201 for anyone following ops-ranges.
