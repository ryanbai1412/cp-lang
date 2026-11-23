# Rebuild round 9: holes

I read commit `65abbcf69` of ryanbai1412/cp-lang and nothing else: `spec/` (README, SUBSET, RATIONALE, GLOSSARY, diagnostics, `core/`, `lib/`, `tools/`) and the stubs in `std/`. I did not open `spec/conformance/`, `plan/`, `corpus/` or `spec/idioms/`, and I did not run the compiler. The ten solutions use only rows that SUBSET.md marks supported (std-bag, std-dsu, std-prefix, std-mod, std-seq, std-math, std-io, core chapters 1 to 10). No problem forced me outside the subset. The one hole outside it is the compiler stamp (R11.4). Problem statements came from the Luogu mirror, because codeforces.com returned HTTP 403 to the VM.

## Summary

| Class | Inside subset | Outside subset | Total |
|---|---|---|---|
| Contradiction | 6 | 0 | 6 |
| Silent | 11 | 1 | 12 |
| Missing stdlib | 4 | 0 | 4 |
| **Total** | **21** | **1** | **22** |

The ten holes most likely to hit a contestant, most likely first:

1. H-MATCH-EXPR: is `match` an expression (`let x = match ...`)? The grammar says no; `std/io.cp` uses it as one.
2. H-MAP-STUB: `Map`/`Set` are the default tables, but `std/` has no stub for them (constructor, `[:]`, methods).
3. H-DROP-RESULT: does `d.unite(a, b)` or `s.add(x)` as a statement warn? If it does, every DSU loop needs `let _ =`.
4. H-PREFIX-INDEX: `pre[c]` is how the spec reads a scan, but the `Prefix` stub has no index impl.
5. H-INT-POW: what `Int ** Int` means: overflow, negative exponent, cost.
6. H-SORTEDSET-RANGE-TODO: the documented ordered-range query `s.range(l..=r)` is a `todo` body.
7. H-KEYWORD-ARG-LABEL: `drain(..., while: f)` passes the keyword `while` as a label, and no grammar rule derives it.
8. H-COUNTED-LIST-ROWS: does `read<[m]List<T>>()` read count-prefixed rows the way `[V]List<V>` does?
9. H-POSITION-ORD-TRAIT: are domain positions `Ord`, so that `SortedSet<V>` and `v <= r` type-check?
10. H-ENUM-RUNTIME-DOMAIN: can an `enum` below `@!multitest` have fields of a runtime domain, and does it hoist above that domain?

## Contradiction

### H-MATCH-EXPR
- **Question:** Can `match` be used as an expression, e.g. `var (di, dj) = match d: ...` or `fn f() -> T = match x: ...`?
- **Readings:** (a) No. `spec/core/01-lexical-grammar.md` R01.10 (line 418) gives `expr := lambda | if_expr | binary`, so `match` is only `match_stmt` (line 397, under `compound`), and `fn_decl`'s `'=' expr` (line 349) cannot hold one. The §1.7 tour line `match d: L => -1, R => 1` is a statement. (b) Yes. `std/io.cp:16` is `pub fn yes(...) -> Str = match answers:` with value arms, and R01.2.2 resumes layout after `:` inside a bracketed `match`. Reading (b) suggests a block's last expression may be a `match`.
- **Chose:** (b) in `05-bouncy-ball.cp` (`var (di, dj) = match read<...>(): ...`), following the stdlib.
- **Cost if wrong:** a syntax error (E0103). The rewrite is four `if` lines or a lookup table. That costs about a minute, but the error comes on the most natural spelling of "map the direction token to a vector".
- **Subset:** inside (R01.10 supported; std-io supported).

### H-KEYWORD-ARG-LABEL
- **Question:** How is `SortedSet.drain`'s predicate passed, and is `while:` legal as an argument label?
- **Readings:** `std/bag.cp:516` and `:1201` declare `drain(mut self; after: ?K = None, before: ?K = None, while ok: K -> Bool)`. That is an external label `while` and an internal name `ok`, and R01.10's `param` (line 354, `'mut'? (pattern | 'self') (':' type)? ...`) has no form for it. `while` is a reserved word (R01.3, line 104), and `arg := NAME ':' expr` needs a `NAME`. Readings: (a) labels may be keywords and the two-name form is real but undocumented; (b) the call is `drain(after: x, ok: f)`; (c) the predicate is positional.
- **Chose:** (a), `live.drain(after: l.prev, while: k => k <= r)` in `01-range-update-point-query.cp`, as the doc example writes it.
- **Cost if wrong:** a compile error on the one std call that deletes a key range from a sorted set. The fallback is a manual cursor loop, about five lines.
- **Subset:** inside (std-bag supported; R01.10 supported).

### H-RANGE-START-END
- **Question:** What are a range's end cuts called: `r.l`/`r.r` or `r.start`/`r.end`?
- **Readings:** `spec/core/04-indices.md` R04.4.15 gives `r.l`, `r.r`, and line 358 says "Rejected: `.start`/`.end`/`.before`/`.after`". But `std/prefix.cp` uses `r.start`/`r.end` in `FillPrefix.fold` (89-92), `Prefix.fold` (147), `cyclic_sum` (158), the list prefix (390-394) and `Diff.add` (501-504). `spec/lib/sequence-patterns.md:231` gives `RingSlice` the fields `start` and `end`.
- **Chose:** `.l`/`.r` in my own code (`06-tea-tasting.cp` uses `i.l`, `stop.l`, `stop.prev`). I assumed `Diff.add` works as its doc says, whatever its body spells.
- **Cost if wrong:** a contestant who copies the stub style into a helper gets E0203 (unknown field). The bigger risk is that these std bodies do not check, so `Diff.add` and `Prefix.fold` fail on first use.
- **Subset:** inside (R04.4 supported; std-prefix supported).

### H-SUM-SPELLING
- **Question:** Is the sum of a function over a sequence `sum(f(x) for x in xs)` or `xs.sum(of: f)`?
- **Readings:** `std/README.md:229-234` says "`sum`, `prod`, `min` and `max` of a function stay `sum(f(x) for x in xs)`: one spelling per idea". `std/seq.cp:34` declares `pub fn sum<S: Seq, T, U>(self: S, of: T -> U = x => x)`, which gives a second spelling.
- **Chose:** neither was needed. I wrote `a[i].digits().sum()` (a fold of one named sequence, R07.9.1).
- **Cost if wrong:** low (a taste lint), but it is a trap for anyone writing a weighted sum.
- **Subset:** inside (R07.9 supported; std-seq supported).

### H-MOD-INV
- **Question:** Is `Mod` inverse a call `x.inv()` or a getter `x.inv`?
- **Readings:** `spec/lib/math.md:196` has `fn inv(self) -> Mod<P>  // O(log P)`, and `std/mod.cp:53-59` documents `x.inv()`. `std/mod.cp:144` writes `return a.inv ** (-e)` with no parentheses, which R01.8.7 and decision 140 forbid for an \(O(\log P)\) member.
- **Chose:** not needed by these ten. I would write `x.inv()`.
- **Cost if wrong:** a compile error in a counting problem that divides. It is cheap to fix, but this is the stdlib breaking its own "parentheses mean work" rule.
- **Subset:** inside (std-mod supported).

### H-POSITION-ORDINAL
- **Question:** What is the 0-based ordinal of a position (for `1..=n` positions, 0 for the first)?
- **Readings:** `spec/core/04-indices.md` R04.2.15 gives `Int(v)`, which is the integer itself (1..n for `1..=n`), and R04.2.16 gives `Int(v.l)` for a count of positions before it. The stubs use two undocumented names: `std/dsu.cp:54` `v.ord` and `std/graph.cp:291` `u.to_idx`, plus `V.nth(k)`. GLOSSARY.md promises one term per concept.
- **Chose:** not needed. I would write `Int(v.l)`.
- **Cost if wrong:** an off-by-one in a 1-based domain if a contestant guesses `Int(v)`, or E0203 on `.ord`/`.to_idx` if those are private.
- **Subset:** inside (R04.2 supported).

## Silent

### H-DROP-RESULT
- **Question:** Is `d.unite(v, a[v])` (it returns `Bool`) legal as a bare statement, or does it get the unused-result warning?
- **Readings:** R06.4.8 (`spec/core/06-control-functions.md:151`) says dropping a non-unit result is a lint unless the fn is `@droppable`. No stub in `std/` carries `@droppable`, so `Dsu.unite`, `SortedSet.add/remove` and `Marks.add` would all warn. The stdlib itself writes `for x in xs: let _ = s.add(x)` (`std/bag.cp:1025`, `:2366`). Readings: (a) every such call warns and `let _ =` is idiomatic; (b) a `Bool` "did it change" result is meant to be droppable and the annotation is missing.
- **Chose:** (a): `let _ = d.unite(...)` in `09-round-dance.cp` and `let _ = live.add(i)` in `01`.
- **Cost if wrong:** noise only, but it is on the most common DSU line in contest code, and `let _ =` reads as a hack ("calm at 3am").
- **Subset:** inside (R06.4 supported; std-dsu supported).

### H-COUNTED-LIST-ROWS
- **Question:** Does `read<[m]List<1..=5e4>>()` read `m` rows, each a count followed by that many values?
- **Readings:** R10.2.3 says "`[V]List<V>` reads count-prefixed rows", with a declared domain and position elements. Not stated: whether a counted `[m]` and non-position elements (a range type) follow the same rule, or whether `List<T>` alone reads the rest of the line.
- **Chose:** the count-prefixed rule applies to any `[D]List<T>` (`07-three-sevens.cp`).
- **Cost if wrong:** a silent misread. If `List<T>` read a whole line or a whole stream, every later token shifts and the answers are wrong with no trap.
- **Subset:** inside (R10.2 supported).

### H-ENUM-RUNTIME-DOMAIN
- **Question:** Can an `enum` declared at top level below `@!multitest` have fields typed by a runtime domain declared just above it? Does the item hoist above that domain?
- **Readings:** R01.9.1 says types "may be declared inside a function body and use its runtime domains", but the per-test region is not called a function body. R11.1.1 says items hoist within their block, except that a runtime `domain` runs where it is written. So the enum hoists, and its field types would then name `V` before `V` exists. R05 §11 says "the file body is a fn".
- **Chose:** legal, with the enum written after `domain V` (`01-range-update-point-query.cp`).
- **Cost if wrong:** a compile error. The workaround is `Int` fields plus `V(x)` at use, which loses the read-time validation the domain gives.
- **Subset:** inside (R01.9, R02.9, R11.1 supported).

### H-ENUM-ARRAY-READ
- **Question:** Does `read<[q]Query>()` read `q` tagged commands, each tag followed by its payload?
- **Readings:** R10.2.3 defines enum reads ("a command stream is `read<Cmd>()` in a loop") and array reads separately. Their composition and the integer `@tag(1)` on payload variants are not shown together.
- **Chose:** the array form, `for query in read<[q]Query>():`.
- **Cost if wrong:** a compile error, fixed with `repeat q: match read<Query>(): ...`.
- **Subset:** inside (R10.2 supported; R02.7 supported).

### H-INT-POW
- **Question:** What does `Int ** Int` do: exact power with an overflow check, any cost guarantee, and a negative exponent?
- **Readings:** R01.5 gives only the precedence, and R01.11 maps `x.pow(k)` to `x ** k`. `Mod` documents `Pow` with a negative exponent meaning the inverse (math.md §2.2). For `Int`, nothing says whether `2 ** -1` traps, is 0, or is a type error, or whether overflow follows R03.4.
- **Chose:** exact power, exponent at least 0, overflow checked like `*` (`04-maximum-set.cp`: `2 ** (k - 1)`, `2 ** (k - 2)` only when `k >= 2`).
- **Cost if wrong:** wrong answers if `**` were float-valued or wrapped silently; a trap if a negative exponent were evaluated.
- **Subset:** inside (R01.5 supported; R03.4 supported).

### H-CONST-EXPR-BOUND
- **Question:** Is `read<1..=2 ** 29>()` a static bound (storage width, proofs)?
- **Readings:** R01.4.1 makes `2e5` and `1e9 + 7` exact `Int` literals, and R01.10 `range_type := expr? '..=' expr?`. Whether `**` folds at compile time is not stated, and the statement writes \(2^{29}\).
- **Chose:** it folds (`03-vlad-pair.cp`).
- **Cost if wrong:** the bound becomes runtime-checked or a compile error. Rewriting as `536_870_912` costs a lookup.
- **Subset:** inside (R03.1 supported).

### H-POSITION-ORD-TRAIT
- **Question:** Are domain positions `Ord`, so that `SortedSet<V>` and `k <= r` on positions work?
- **Readings:** chapter 4 calls a domain "a finite, ordered index type", but no rule says its positions implement `Ord` (the bound `SortedSet<K>` needs). `?T` is explicitly not `Ord` (R02.4.5).
- **Chose:** positions are `Ord` (`01`: `SortedSet.of([v for v in V if ...])`, `k <= r`).
- **Cost if wrong:** a compile error. The workaround is `SortedSet<Int>` with `Int(v)` and `V(x)` round-trips.
- **Subset:** inside (R04.2 supported; std-bag supported).

### H-RUNTIME-RANGE-LOCAL
- **Question:** Can a local be annotated with a runtime range, `var i: 1..=n = read()`, with each assignment checked?
- **Readings:** R02.10.2 says `var x: lo..=hi` checks each assignment, and §9.3 mentions `read<1..=n>()`. R03.1.3 bans runtime bounds only for struct fields. A local with a runtime-bounded type is never shown.
- **Chose:** legal (`05-bouncy-ball.cp`), which reads and validates in one line.
- **Cost if wrong:** a compile error. The fix is plain `Int` locals, which lose the bound checks.
- **Subset:** inside (R02.10 supported).

### H-STR-LEN-EXACT
- **Question:** How do I read `s` and validate `|s| == n` from the previous token?
- **Readings:** R02.3.2 gives `Str(len: 1..=2e5)` (a static range) and nothing for an exact runtime length (`Str(len: n)`? `Str(len: n..=n)`?).
- **Chose:** plain `read<Str>()` and trusting the input (`02-remove-two-letters.cp`).
- **Cost if wrong:** none for correctness. The cost is lost input validation, the thing the type system promises for "constraints and formats from the statement".
- **Subset:** inside (R02.3 supported).

### H-SORTEDSET-CTOR
- **Question:** How do I build a `SortedSet` from a sequence?
- **Readings:** R07.8.1 says conversions are `List(s)`, `Set(s)`, `Str(s)`, and R02.6.3 says `T(args)` calls `T.new`. `SortedSet.new()` takes no arguments (`std/bag.cp:1015`), so `SortedSet(xs)` is an arity error. The working form is `SortedSet.of(xs: List<K>)` (`:1023`), which needs a `List`, not any `Seq`.
- **Chose:** `SortedSet.of([v for v in V if a[v] >= 10])`.
- **Cost if wrong:** a compile error and a search through the stub.
- **Subset:** inside (std-bag supported).

### H-REPEAT-VS-FOR
- **Question:** For a loop whose index is unused, is the idiom `repeat k:` or `for _ in 0..k:`?
- **Readings:** R01.3's contextual-word table says `repeat k:` is "the same as `for _ in 0..k:`". That gives two spellings, and nothing says which is canonical or whether `return` and `break` inside `repeat` behave as in `for` (R01.7.4 covers generators only).
- **Chose:** `repeat 4 * n * m:` with a top-level `return` inside (`05`).
- **Cost if wrong:** a lint at most.
- **Subset:** inside (R01.6, R01.7 supported).

### H-STAMP-VS-HEADER
- **Question:** Does a solution start with `// cpl 0.3.0` or with the problem comment?
- **Readings:** R11.4.1 says the stamp is "the first line of a solution ... above any header comment", added by the formatter, and that a file without one is valid.
- **Chose:** header comment first, no stamp (the files are unformatted source).
- **Cost if wrong:** none. `cpl fmt` adds it.
- **Subset:** outside (R11.4 is documented, not supported).

## Missing stdlib

### H-MAP-STUB
- **Question:** What are `Map`'s constructor, empty literal, index semantics and costs?
- **Readings:** `Map`/`Set` are the default tables (RATIONALE "Containers: 231"; `spec/lib/containers.md:172`, `:535` call them "Builtin"; R07.12.9 gives `m[k]` read-traps and `op=` defaults). But `std/` has no `struct Map`/`Set` stub. `std/seq.cp:578` uses `Map()`, and R01.4.5 offers `[:]`. Hashed vs ordered is decided by the compiler from "whether order is observed".
- **Chose:** `var last: Map<Int, Int> = [:]` with `last[p] = day` and `last[p]` reads (`07-three-sevens.cp`).
- **Cost if wrong:** compile errors on construction. The bigger cost is that the cost model (hash vs tree, \(O(1)\) vs \(O(\log n)\)) cannot be read anywhere, against "every function documents its cost".
- **Subset:** inside (R07.12 supported).

### H-PREFIX-INDEX
- **Question:** Does a `Prefix` support `pre[c]` (cut) and `pre[v]` (position)?
- **Readings:** R07.1.6, chapter 7 "Scans are prefix sums over cuts", and the `scan` doc in `std/prefix.cp:13-14` all write `pre[r.r] - pre[r.l]` and `pre[v]`. The `Prefix<D, T>` struct (`std/prefix.cp:101`) declares no index impl, only `prefix(b: Cut<D>)`, `fold`, `get` and `first_ge`/`first_gt`.
- **Chose:** `pre[i.l]`, `pre[last]` (`06-tea-tasting.cp`).
- **Cost if wrong:** a compile error (E0305) on the most common use of a prefix sum. The fix is `.prefix(c)`.
- **Subset:** inside (R07.1 supported; std-prefix supported).

### H-SORTEDSET-RANGE-TODO
- **Question:** How do I walk or delete the keys of a sorted set inside `l..=r`?
- **Readings:** `SortedSet.range(r)` (`std/bag.cp:1212`) is documented as replacing `first_ge`/`first_gt`/`last_le`/`last_lt`, but its body is `todo`, and so are `SortedRange`'s `first`, `last`, `len`, `each` and `rev` (`:2456-2490`). R01.6.7 says reaching `todo` traps. The family is "supported" in SUBSET.md. The implemented route is `drain(after:, while:)`, which removes keys, so it fits 1791F but not a read-only range walk. `read_all` (`std/io.cp:63`) is also `todo`.
- **Chose:** `drain` plus re-adding the keys that are still live (`01`).
- **Cost if wrong:** if a contestant writes the documented `live.range(l..=r)`, the program type-checks and traps on the first query.
- **Subset:** inside (std-bag supported; std-io supported).

### H-INTRINSIC-SCOPE
- **Question:** Which helpers does a program see without `use`: `bit_width`, `popcount`, `isqrt`, `bit_len`?
- **Readings:** `std/README.md:224` lists intrinsics (`popcount`, `ctz`, `bit_width`, `isqrt`, `min`, `max`, `abs`, ...) that have no stub, and none of them appears in `std/prelude.cp`. `bit_len` exists in `std/math.cp:405` but is not in the prelude, and R11.3.2 says a name the lib tables don't list needs a `use`.
- **Chose:** I avoided these and computed \(\lfloor \log_2(r/l) \rfloor\) with `(0..=20).last_true(j => l * 2 ** j <= r)!` in `04-maximum-set.cp`, but used `min`/`max` freely.
- **Cost if wrong:** E0201 on `bit_width(x)` or `bit_len(x)`, which is the one-liner most contestants would write.
- **Subset:** inside (R11.3 supported).
