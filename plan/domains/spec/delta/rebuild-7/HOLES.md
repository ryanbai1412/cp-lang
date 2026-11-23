# Rebuild round 7: holes

I read commit `906a23e0b` of `ryanbai1412/cp-lang`, extracted with `git archive 906a23e0b spec std`, and opened only `spec/` (without `spec/conformance/`) and the root `std/` stubs. I did not open `plan/` or `corpus/`, and I did not run the compiler. codeforces.com returned HTTP 403 to this machine, so the I/O contracts and constraints in the file headers come from my own knowledge of the ten statements. 1324D is a single-test problem, so its file has no `@!multitest` (see H-SINGLE-TEST). Each hole names the files it touched. "Subset" cites the `spec/SUBSET.md` row. Per `SUBSET.md` line 17, library families follow their chapters' status lines. No `spec/lib/` chapter claims Milestone 1 support, so stdlib-only facts count as outside the subset.

## Summary

| Class | Total | Inside subset | Outside subset |
|---|---|---|---|
| Contradiction | 7 | 7 | 0 |
| Silent | 24 | 21 | 3 |
| Missing stdlib | 2 | 1 | 1 |
| **All** | **33** | **29** | **4** |

The ten holes most likely to hit a contestant, most likely first:
1. H-LAMBDA-TUPLE-PARAMS
2. H-SORT-DESC
3. H-BSEARCH-NAME
4. H-INT-RANGE-CUT
5. H-COUNT-STR
6. H-TALLY-TYPE
7. H-POS-TO-INT
8. H-STR-EXACT-LEN
9. H-ABS
10. H-PRINT-TUPLE-LIST

## Contradiction

### H-TALLY-TYPE
- **Question:** What does `s.tally(by:)` return? (02)
- **Readings:** (a) `Counter<K>` (`spec/core/07-sequences.md` line 323, R07.11.4); (b) `HashMap<K, Int>` (`std/seq.cp` line 1793).
- **Chose:** I used only `.values`, which both types have (`std/hashmap.cp` lines 222 and 455).
- **Cost if wrong:** Code that relies on Counter's missing-key-reads-0 behaviour (`c[k]` for an absent `k`) traps on a HashMap, or the reverse.
- **Subset:** inside (R07.11 Ordering and grouping, supported).

### H-SORT-DESC
- **Question:** How do you sort in descending order? (06)
- **Readings:** (a) as a key: "Descending is a key, not a flag: `by: x => desc(x.score)`" (`spec/core/07-sequences.md` line 326, R07.11); (b) as a flag: `sort(..., desc: Bool = false)` and `sorted(..., desc: Bool = false)` (`std/seq.cp` lines 225 and 273).
- **Chose:** `a.sorted(by: desc)` (the key form, from the spec).
- **Cost if wrong:** Both forms typecheck against the stub. If a contestant writes `by: desc, desc: true`, the two cancel and the sort comes out ascending with no error, which gives a silently wrong answer.
- **Subset:** inside (R07.11, supported).

### H-BSEARCH-NAME
- **Question:** What is the binary-search primitive called, and what does it return? (04)
- **Readings:** (a) `r.bsearch(ok)` returns the cut where `ok` flips and "always exists" (`spec/core/04-indices.md` line 226, R04.4); (b) `bsearch_first(r, ok) -> ?I` returns the first position (`std/README.md` lines 285 and 291, also lines 131-137 and 744).
- **Chose:** (a), `(i + 1..c.len).bsearch(j => c[j] > -c[i])`.
- **Cost if wrong:** A compile error. Worse, the `?I` reading needs `?? hi` handling for "not found", while the cut reading has no not-found case, so translating between the two invites an off-by-one.
- **Subset:** inside (R04.4, supported).

### H-COUNT-STR
- **Question:** Is `s.count(...)` on a string given a `Char` or a predicate? (08, design stage)
- **Readings:** (a) `count(self: Str, c: Char)` (`std/strings.cp` line 76); (b) `count(self: S, p: T -> Bool)` (`std/seq.cp` line 94). Functions are not overloaded (R06.1.6, `spec/core/06-control-functions.md` around line 45), so both cannot apply to `Str`.
- **Chose:** I avoided the question: 08 counts white cells with `Prefix` over a 0/1 list.
- **Cost if wrong:** A compile error or ambiguity on the most common string fold.
- **Subset:** inside (R06.1 Functions, supported).

### H-FIND-FIRST
- **Question:** What is the name of "the first element satisfying p"? (considered for 04 and 10)
- **Readings:** (a) `xs.first(p)`, with `find(x)` as "the index of a value" (`std/README.md` lines 124-127); (b) `find(p) -> ?T` (`std/seq.cp` line 952).
- **Chose:** Unused in the final code. Had I needed it, I would have written `find(p)` because the stub defines it.
- **Cost if wrong:** A compile error, or with reading (a), an index where a value was meant.
- **Subset:** inside (R07.9 Folds, supported).

### H-WINDOWS-COST
- **Question:** Is `windows(k)` a lazy view or does it copy each window? (08, design stage)
- **Readings:** (a) `fn windows<T>(self: List<T>, k: Int) -> yields List<T>`, O(k) per window (`std/README.md` line 300); (b) `windows<S: Indexed>(self: S, k: Int) -> yields Slice<S>`, a view (`std/seq.cp` line 481, R07.7).
- **Chose:** Neither. Even with views, refolding each window costs O(nk) and times out on 1690D, so I used `Prefix`.
- **Cost if wrong:** Under reading (a), `s.windows(k).map(w => ...)` allocates n lists of length k.
- **Subset:** inside (R07.7 Adapters, supported).

### H-SUM-OF
- **Question:** How do you write the sum of a function of each element? (02, 04, 10)
- **Readings:** (a) `xs.sum(of: f)` (`std/seq.cp` line 34); (b) `std/README.md` section 3: the sum of a function "stays `sum(f(x) for x in xs)`".
- **Chose:** (b), the generator form, in all three files.
- **Cost if wrong:** Low. It is a style or lint difference only, as long as both forms compile.
- **Subset:** inside (R07.9, supported).

## Silent

### H-LAMBDA-TUPLE-PARAMS
- **Question:** When a sequence yields tuples, does a lambda with two parameters destructure each tuple? (09: `blue.items.all((i, x) => ...)`)
- **Readings:** (a) yes, `(i, x) =>` matches the `(Int, T)` element; (b) no, the lambda takes one parameter: `p => p.1 >= p.0 + 1` or `((i, x)) =>`.
- **Chose:** (a).
- **Cost if wrong:** A compile error on every `.items` fold. This is very common in contest code.
- **Subset:** inside (R06.1 Functions, supported).

### H-INT-RANGE-CUT
- **Question:** On a plain `Int` range, is the cut returned by `bsearch` an `Int` usable in arithmetic (`c.len - cut`)? Does an empty range return its single cut? (04)
- **Readings:** (a) the cut is `Int` (R04.4.30 base 0; line 226 "always exists"); (b) the cut is a distinct cut type that needs `Int(cut)`.
- **Chose:** (a).
- **Cost if wrong:** A compile error. Mixing cuts and positions is exactly what the spec exists to prevent, so this needs a stated rule.
- **Subset:** inside (R04.4, supported).

### H-POS-TO-INT
- **Question:** For `domain I = 1..=n`, is `Int(i)` the written value (1..n) or the 0-based offset? (02) `std/graph.cp` line 291 uses `u.to_idx` for the offset.
- **Readings:** (a) the written value; (b) the offset; (c) `a[i] - i` works directly, since position arithmetic yields `Int` (R04.3).
- **Chose:** (a), `Int(i)`. The answer is the same under (a) and (b) because a constant shift does not change which differences are equal.
- **Cost if wrong:** Off-by-one in any problem where the shift does not cancel.
- **Subset:** inside (R04.2 Domains, supported).

### H-STR-EXACT-LEN
- **Question:** How do you declare a string of exactly length `n` over a given alphabet? (01, 08, 09, 10)
- **Readings:** (a) `Str of "01"` alone; (b) `Str(len: n..=n) of "01"`; (c) some other combination. The spec shows `of` and `Str(len: range)` separately (`spec/core/02-values-types.md` lines 68-70) but never together.
- **Chose:** (a), so the length goes unvalidated.
- **Cost if wrong:** Malformed input is not caught, and the "state the constraint once" promise is lost.
- **Subset:** inside (R02.3 Strings, supported).

### H-RUNTIME-RANGE-BOUND
- **Question:** Can a range type have a runtime bound, as in `let k: 1..=n = read()`? (07, 08) In 02 I wrote the static `1..=2e5` instead of `1..=I.count`.
- **Readings:** (a) allowed, as a fresh runtime-parameterized type (R02.9); (b) only literal bounds are allowed.
- **Chose:** (a).
- **Cost if wrong:** A compile error. The fix is to widen to the static bound.
- **Subset:** inside (R02.9 Fresh types, supported).

### H-PRINT-TUPLE-LIST
- **Question:** Does `print(talks, sep: "\n")` on a `List<(P, P)>` print each tuple as "i j" on its own line? (03)
- **Readings:** (a) yes: inner fields are joined by `io.sep` (R10.3.5) and elements by `sep:`; (b) `sep:` also applies inside each tuple.
- **Chose:** I avoided the question with an explicit `for (i, j) in talks: print(i, j)`.
- **Cost if wrong:** Wrong output format.
- **Subset:** inside (R10.3 Printing, supported).

### H-POSITION-PRINT
- **Question:** Does printing a position of `domain P = 1..=n` print the written 1-based value? (03)
- **Readings:** (a) the written value, since the domain keeps its base (R10.2.2); (b) the offset.
- **Chose:** (a).
- **Cost if wrong:** Every person index in the output is off by one.
- **Subset:** inside (R10.3, supported).

### H-LET-TUPLE-UNWRAP
- **Question:** Is `let (x, i) = h.pop()!` legal, destructuring an unwrapped optional tuple? (03)
- **Readings:** (a) yes; (b) it needs a separate `let e = h.pop()!` first.
- **Chose:** (a).
- **Cost if wrong:** A compile error.
- **Subset:** inside (R02.5 Tuples and records, supported).

### H-COMPREHENSION-FILTER
- **Question:** Do list comprehensions take an `if` filter, as in `[e for p in P if c]`? (03, 09, 10) `for` loops have filters (R01).
- **Readings:** (a) yes; (b) you must use `.filter(...)`.
- **Chose:** (a).
- **Cost if wrong:** A compile error.
- **Subset:** inside (R07.8 Making containers, supported).

### H-DIMS-EXPR
- **Question:** Can the count in an array read type be an expression, as in `read<[V.count - 1](u: V, v: V)>()`? (05) The spec example uses a declared domain `[E]`.
- **Readings:** (a) any `Int` expression; (b) only a name or domain.
- **Chose:** (a).
- **Cost if wrong:** A compile error. The fix is to read `m` into a `let` first.
- **Subset:** inside (R10.2 Reading, supported).

### H-GRAPH-DEG
- **Question:** Does `Graph.undirected(edges).deg(x)` give the degree, with the edge record fields named `u` and `v`? (05)
- **Readings:** (a) yes (`std/graph.cp` line 291); (b) `Tree` or a hand count is the idiomatic way.
- **Chose:** (a).
- **Cost if wrong:** A compile error, or a field-name diagnostic.
- **Subset:** outside (`spec/lib/graphs-trees.md` has no status line, and no `SUBSET.md` row covers it).

### H-YES-ANSWERS
- **Question:** Is `yes(b, answers: ("Ayush", "Ashish"))` the idiom for a two-way answer that is not yes/no? (05)
- **Readings:** (a) the `answers:` argument (`std/io.cp` line 16); (b) `with io(answers: ...)` (idioms principle 11); (c) an inline `if` expression.
- **Chose:** (a).
- **Cost if wrong:** Style only.
- **Subset:** inside (R10.3, supported).

### H-IF-EXPR-ELIF
- **Question:** Is `if a: x elif b: y else: z` valid as an expression? (06)
- **Readings:** (a) yes; (b) only `if/else`, so it must nest or use `match`.
- **Chose:** (a).
- **Cost if wrong:** A compile error.
- **Subset:** inside (an R01 grammar row, all supported; I did not pin down the exact row).

### H-ARRAY-FILL
- **Question:** Does `[C] _ => None` build a `[C]?Int` filled with `None`? (07) The form appears in `spec/idioms/principles.md` as `[0..=n] _ => 0`.
- **Readings:** (a) yes; (b) plain `= None` broadcasts the value.
- **Chose:** (a).
- **Cost if wrong:** A compile error.
- **Subset:** inside (R02.8 Arrays and lists, supported).

### H-MIN-ASSIGN-OPT
- **Question:** For `?Int`, does `best[c] min= run + 1` treat `None` as the identity on both sides, with `None + 1 = None`? (07)
- **Readings:** (a) yes (R02.4: lifting, and min with `None` as identity); (b) `min=` requires a non-optional value.
- **Chose:** (a).
- **Cost if wrong:** A compile error, or wrong answers if `None` were treated as the minimum.
- **Subset:** inside (R02.4 Optionals, supported).

### H-REV-DOMAIN
- **Question:** Can you write `C.rev` on a domain to walk it backwards? (07)
- **Readings:** (a) yes, a domain is a bidirectional range; (b) you need `C.first!..` or `for c in C.rev()`.
- **Chose:** (a).
- **Cost if wrong:** A compile error.
- **Subset:** inside (R07.7, supported).

### H-BLANK-LINES
- **Question:** Do token reads skip the blank lines that 1547E puts between tests? (07)
- **Readings:** (a) yes, newlines count as the "whitespace" skipped before a token (`spec/core/10-io.md` line 90); (b) blank lines need `read_line()`.
- **Chose:** (a).
- **Cost if wrong:** A read error.
- **Subset:** inside (R10.2, supported).

### H-PREFIX-LIST
- **Question:** Does `Prefix(list)` accept a `List<Int>`, and does `fold(l..l + k)` take `Int` cuts? (08) The stub takes `[D]T` (`std/prefix.cp` line 110).
- **Readings:** (a) yes, since `List<T>` is `Indexed<Int, T>`; (b) it needs a `[n]Int` array.
- **Chose:** (a).
- **Cost if wrong:** A compile error.
- **Subset:** outside (a library family with no supported row).

### H-RANGE-PRECEDENCE
- **Question:** Do `..` and `..=` bind looser than `+` and `-`, so `0..=n - k` is `0..=(n - k)` and `i + 1..c.len` is `(i + 1)..c.len`? (04, 08)
- **Readings:** (a) looser; (b) tighter.
- **Chose:** (a).
- **Cost if wrong:** A wrong range, which is a silent off-by-one.
- **Subset:** inside (R01.5 Operators and precedence, supported).

### H-DOMAIN-MAP-SORTED
- **Question:** Does `I.map(f).sorted()` work on a domain and return a `List` keyed from 0? (04)
- **Readings:** (a) yes; (b) you need a comprehension first.
- **Chose:** (a).
- **Cost if wrong:** A compile error, or wrong keys.
- **Subset:** inside (R07.11, supported).

### H-SINGLE-TEST
- **Question:** 1324D has a single test, but the brief says all ten are multitest. Which wins?
- **Readings:** (a) follow the statement and leave out `@!multitest`; (b) add `@!multitest`.
- **Chose:** (a).
- **Cost if wrong:** Under (b), the program would read `n` as the test count and produce wrong answers.
- **Subset:** inside (R10.1 Program shape, supported).

### H-NEG-RANGE-TYPE
- **Question:** Does `[n]-1e9..=1e9` parse as an array of the range `-1e9..=1e9`? (09)
- **Readings:** (a) yes; (b) it needs parentheses.
- **Chose:** (a).
- **Cost if wrong:** A parse error.
- **Subset:** inside (R01.4 Literals, supported).

### H-RETURN-ONE-LINE
- **Question:** Do both statements in `if c: print(0); return` belong to the `if`, and does top-level `return` end only the current test? (10) See `spec/core/10-io.md` line 37.
- **Readings:** (a) yes to both; (b) `return` ends the whole program.
- **Chose:** (a).
- **Cost if wrong:** The remaining tests are skipped.
- **Subset:** inside (R10.1, supported).

### H-LIB-SUBSET-STATUS
- **Question:** Are `Heap`, `Graph` and `Prefix` (03, 05, 08) inside Milestone 1?
- **Readings:** (a) inside, because they are in the prelude; (b) outside, because `SUBSET.md` line 17 defers library families to their chapter status lines, and those are "design survey and API" or absent.
- **Chose:** (b).
- **Cost if wrong:** Over-reporting outside-subset use.
- **Subset:** outside (no row).

## Missing stdlib

### H-ABS
- **Question:** Where is `abs` for `Int`? (10) `std/` has `abs` only as methods on `Ratio` and `BigInt` (`std/math.cp` lines 1464 and 1570). `math.cp` calls a free `abs(p)`, but no stub declares it.
- **Readings:** (a) a core builtin; (b) `x.abs`; (c) `max(x, -x)`.
- **Chose:** (a), `abs(g - mid)`.
- **Cost if wrong:** A compile error on a basic operation.
- **Subset:** outside (no row covers it).

### H-READ-ALL-EOF
- **Question:** Can a program read until end of input? R10.1 names `@!multitest(until_eof)` and `read_all` (`spec/core/10-io.md` lines 33-34), but `std/io.cp` defines `read_all = todo` ("blocked: no end-of-input primitive"). None of these ten problems uses it.
- **Readings:** (a) usable; (b) not yet usable.
- **Chose:** Not used.
- **Cost if wrong:** EOF-terminated problems cannot be written.
- **Subset:** inside (R10.1, supported, which makes the missing stub worse).
