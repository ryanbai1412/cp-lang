# Rebuild round 10: holes

I read commit `887ad2abe` and, for the language, only `spec/` (README, SUBSET, RATIONALE, GLOSSARY, diagnostics, `core/`, `lib/`, `tools/`; not `spec/conformance/`) and the stdlib stubs in `std/`. I did not open `plan/`, `corpus/` or the compiler sources, and I ran nothing. The repository's agent rules made the tooling show me `AGENTS.md` too. It has no language rules, and I used none from it. The problem statements came from Luogu's CF mirror, because Codeforces returns 403 to scripts. Line numbers below are at `887ad2abe`.

## Summary

15 holes: 3 contradiction, 10 silent, 2 missing-stdlib. 14 are inside the Milestone 1 subset (SUBSET.md row `supported`), and 1 is outside (H-SET-BUILTIN: no SUBSET row covers the builtin `Set`/`Map`).

The ten most likely to hit a contestant, most likely first:

1. H-LITERAL-UNION: the statement's `+`/`-` tokens are either a literal union or an enum, and the spec says both.
2. H-SPLIT-OVERLOAD: `s.split(x)` has two stubs, `Str`/`Char -> List<Str>` and a predicate that yields slices.
3. H-LIST-POP: `List.pop()` appears throughout the spec but has no stub.
4. H-WHILE-LET-SHAPE: `while let x = q.pop()` (spec) vs `while let Some(x) = q.pop()` (std).
5. H-DISCARD-RESULT: a dropped `Bool` from `set.add(x)` or `bag.remove(x)`. It may warn, and it is unclear whether `let _ =` silences it.
6. H-SET-BUILTIN: the spec calls `Set`/`Map`/`HashMap`/`HashSet` builtins, but they have no stub, so their API is unknown.
7. H-POS-PRINT-BASE: does printing a `1..=n` position write `1..n`, or something in the "file's base"?
8. H-COUNT-EXPR: is a counted array type `[n - 1]` with an expression allowed for the n-1 tree edges?
9. H-NEG-RANGE-TYPE: is `[V]-1e9..=1e9` a valid element type, or does it need parentheses?
10. H-FN-AS-NEXT: can a lambda be the `Next<V>` for `components`/`reachable` (e.g. grid neighbours)?

## Contradiction

### H-LITERAL-UNION
- **Question:** How do you type a token set from the statement, such as `+ l r` / `- l r` in 1883D?
- **Readings:** (a) A literal union `'+' | '-'` or `"add" | "del"`. `spec/core/02-values-types.md:58-60` (R02.2.2) calls these scalar types that `read` validates, and `spec/core/01-lexical-grammar.md:157-159` (R01.4.5, decision 135) lists them as types. (b) A payload-free or tagged enum. `spec/core/02-values-types.md:194-196` (R02.7.2, decision 356) says "there is no union of string literals". (c) Char unions survive and only string unions are gone.
- **Chose:** (b), `enum Op = @tag("+") Add(l, r) | @tag("-") Del(l, r)` read with `read<Op>()` (R10.2.3). It is the later decision, and it reads the payload too.
- **Cost if wrong:** It is a compile error at most, but it is the first line a contestant writes for any query-stream problem.
- **Subset:** inside (core-02, core-10 supported).

### H-SPLIT-OVERLOAD
- **Question:** What does `s.split('B')` on a `Str` return?
- **Readings:** (a) `std/strings.cp:239` `pub fn split(self: Str, sep: Char) -> List<Str>`, which keeps empty pieces. (b) `std/seq.cp:3149` `pub fn split<S: Indexed, T>(self: S, sep: T -> Bool) -> yields Slice<S>`. A `Str` is `Indexed`, so both apply with the same labels. That is the overload RATIONALE.md (Values and types, decision 127) says was given up, and that `spec/diagnostics.md:79` E0702 `overload_by_type` rejects.
- **Chose:** (a). 1873G needs the empty pieces between `BB` and at the ends, and (a) documents that it keeps them.
- **Cost if wrong:** If it resolves to (b) with a `Char` argument, it is a type error. If (b) dropped empty pieces, 1873G would print the wrong answer whenever the string starts or ends with B.
- **Subset:** inside (std-strings, std-seq supported).

### H-WHILE-LET-SHAPE
- **Question:** Does `while let`/`if let` on a `?T` bind the unwrapped value directly, or match `Some(x)`?
- **Readings:** (a) It binds directly: `spec/core/01-lexical-grammar.md:248` `while let x = q.pop(): ...` and `spec/core/04-indices.md:39` `if let q = P.at(...)`. (b) It matches a pattern: `std/graph.cp:660` and `:706` `while let Some(u) = q.pop_front():`. Under (a), `Some(u)` would match against an already-unwrapped `T`, so the two are not both valid unless both forms are accepted.
- **Chose:** (a), the spec's form: `while let p = stack.pop():` (01) and `if let p = t.parent(v):` (02).
- **Cost if wrong:** It is a compile error, in the most common loop shape for BFS, DFS and stacks.
- **Subset:** inside (core-01 supported).

## Silent

### H-POS-PRINT-BASE
- **Question:** What does `print(strong)` write for a `List<V>` with `domain V = 1..=n` (1857D wants 1-based indices)?
- **Readings:** (a) Each position's integer, `Int(v)` (R04.2.15: "for `1..=n`, that's 1..n"). (b) The position shifted by the "file's base" (R10.3.2, "positions print in the file's base"). R10.2.2 says the base applies only to counted domains, but R10.3.2 doesn't repeat that limit.
- **Chose:** (a), so the output is 1..n.
- **Cost if wrong:** Every printed index is off by one, which is wrong answer on test 1.
- **Subset:** inside (core-04, core-10 supported).

### H-DISCARD-RESULT
- **Question:** Can you call `set.add(x)`, `bag.add(x)` and `bag.remove(x)`, which return `Bool` with no `@droppable` (`std/bag.cp:120`, `:157`, `:1064`), as plain statements?
- **Readings:** (a) It is a warning only (W0101; `spec/core/06-control-functions.md:151-154`). (b) It needs `let _ = s.add(x)`. The text says `let _ = f()` "drops it too", which can mean it also silences the lint or that it also triggers it. (c) It needs `with lint(discarded_result: allow)`.
- **Chose:** (a), plain statements in 07 and 10.
- **Cost if wrong:** Warning noise, or a lint failure if warnings are errors. The idiomatic fix isn't stated.
- **Subset:** inside (std-bag supported).

### H-FN-AS-NEXT
- **Question:** Can a neighbour function (`p => p.adj4.filter(...)`) be passed where `components`/`reachable`/`dfs` take a `Next<V>`?
- **Readings:** (a) Yes: `std/graph.cp:5` says they take "a stored `Graph`, a `[V]List` of arcs, or any fn". (b) No: the trait `Next<V>` (`std/graph.cp:28`) requires a method `out`, and nothing says a fn implements it. `std/graph.cp:465` adds a "blocked" note about generic `V` in these free fns.
- **Chose:** Neither; 1829E does its own flood fill with a stack. The `components(...)` one-liner was the more idiomatic choice if (a) holds.
- **Cost if wrong:** Grid problems are written longer than needed, or don't type-check if a contestant assumes (a).
- **Subset:** inside (std-graph, std-grid supported).

### H-COUNT-EXPR
- **Question:** Is `read<[n - 1](u: V, v: V)>()` valid for the n-1 tree edges?
- **Readings:** (a) A counted array type takes any `Int` expression. (b) Only a name (`[n]`, `[m]`, `[q]` in every example) works, so you write `domain E = 0..n - 1` first.
- **Chose:** (a) (02).
- **Cost if wrong:** A compile error and one extra line. It comes up in every tree problem.
- **Subset:** inside (core-04, core-10 supported).

### H-NEG-RANGE-TYPE
- **Question:** Does `read<[V]-1e9..=1e9>()` parse as an array of a range with a negative low end?
- **Readings:** (a) Yes. In a type position, `]-1e9` starts the element type. (b) It needs `[V](-1e9..=1e9)`, but parenthesized types aren't shown either. (c) Write `[V]Int` and give up the range.
- **Chose:** (a) (03).
- **Cost if wrong:** A parse error on a common bound ("-10^9 <= a_i <= 10^9").
- **Subset:** inside (core-01, core-03 supported).

### H-ADJ4-CELL
- **Question:** Does `p.adj4` work on a position `p` of `domain Cell = (R, C)`? `std/grid.cp:221` declares it on the tuple `(R, C)`.
- **Readings:** (a) Yes: a product position is a tuple of factor positions (R04.2.10), so it is the receiver type. (b) No: you must write `(r, c).adj4` after destructuring.
- **Chose:** (a) (01).
- **Cost if wrong:** A compile error, fixed with one line.
- **Subset:** inside (std-grid supported).

### H-STEP-INCLUSIVE
- **Question:** Does `d..=n by d` step an inclusive range?
- **Readings:** (a) Yes. `by` applies "wherever a range is taken" (R01.8.4). (b) Only `a..b by k` is shown, so you write `d..n + 1 by d`.
- **Chose:** (a) (04).
- **Cost if wrong:** A compile error, or the last multiple `n` is skipped (wrong answer whenever the best trap is at `n`).
- **Subset:** inside (core-01, core-04 supported).

### H-CHAR-ARRAY-1D
- **Question:** Does `read<[I]Char>()` read an unspaced string such as `101100` as n characters?
- **Readings:** (a) Yes: `[D]T` reads `D.count` values, and `Char` skips whitespace, then reads one character (R10.2.3). (b) Only `[R, C]Char` is said to read rows with no separators, so 1-D input needs `read<Str>()` and 0-based `Int` indexing.
- **Chose:** (a) (07), so the string is keyed by the same `I` as the queries' `l r`.
- **Cost if wrong:** If it read whitespace-separated tokens, it would misread the input on test 1.
- **Subset:** inside (core-10 supported).

### H-ROOTED-BUILTIN
- **Question:** Which signature does `Rooted(g, root: r)` follow?
- **Readings:** (a) The stub `std/tree.cp:90` `pub fn new(next: Next<V>, root: V)`, with positional `next`. (b) A compiler built-in: the stub is marked "blocked: `Rooted` resolves to a compiler built-in" (`std/tree.cp:88`), and its doc also shows `Rooted(root: r, next: u => ...)`, so the labels may differ from the stub.
- **Chose:** `Rooted(graph, root: V.first!)`, as in `spec/lib/graphs-trees.md` (Trees) and R02.6.2.
- **Cost if wrong:** A compile error. Also, the stub is not the authority it is meant to be.
- **Subset:** inside (std-tree supported).

### H-YES-PROFILE
- **Question:** What does `yes(b)` print under the `codeforces` judge profile?
- **Readings:** (a) `YES`/`NO`: R10.2.3 gives this for "the default profile", and `spec/tools/judge-profiles.md:34` shows `answers = ["YES", "NO"]` in an example table. (b) Something profile-specific. The built-in `codeforces` profile's `answers` value isn't listed.
- **Chose:** (a) (10). Codeforces accepts any case, so the risk is low.
- **Cost if wrong:** Wrong answer on a checker that compares exactly. Not this problem.
- **Subset:** inside (core-10 supported).

## Missing stdlib

### H-SET-BUILTIN
- **Question:** What is the API of `Set<T>`, `Map<K, V>`, `HashMap` and `HashSet`?
- **Readings:** `spec/lib/containers.md:63` and `:176` call them builtins (lowered to `std::set`/`std::map`/`std::unordered_*`). R07.8.1 `Set(s)`, R07.11.3 (`group` returns `Map`) and R07.12.9 (map indexing) use them, and `std/hashmap.cp:255` has a field `m: HashMap<K, V>`. None of them has a stub anywhere in `std/`, so `add` vs `insert`, `len`, `contains` and iteration order are all guesses.
- **Chose:** `SortedSet<(Int, Int)>` (`std/bag.cp:1007`, fully stubbed) for 1849C's distinct-results count.
- **Cost if wrong:** A contestant reaching for `Set` has no reference. A wrong method name is a compile error, and the wrong ordered/hash choice costs time.
- **Subset:** outside (no SUBSET.md row names the builtin containers; std-bag, which I used, is supported).

### H-LIST-POP
- **Question:** Does `List<T>` have `pop() -> ?T`?
- **Readings:** (a) Yes: the spec uses it in `spec/core/01-lexical-grammar.md:248`, `spec/core/04b-keys.md:191`, `spec/core/05-ownership.md:371` and `spec/lib/ops-ranges.md:634`. (b) It doesn't exist: `std/seq.cp` stubs `List.push` (`:2455`), `pop_if` (`:1280`) and `pop_while` (`:1293`), but no `pop`. Only `Deque` (`std/deque.cp:144`) and the heaps have one.
- **Chose:** (a), `var stack = [s]` ... `stack.pop()` (01).
- **Cost if wrong:** A compile error in every DFS, stack or monotone-stack solution, or a switch to `Deque`.
- **Subset:** inside (core-07, std-seq supported).
