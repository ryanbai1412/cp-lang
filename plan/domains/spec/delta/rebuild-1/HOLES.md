# Rebuild round 1: holes in the spec

An independent implementer read only `spec/core`, `spec/lib` (skimmed),
the `std/*.cp` stubs it needed and `spec/conformance`, then wrote ten
solutions (`01-*.cp` to `10-*.cp`) and 42 snippet questions
(`QUESTIONS.md`). This file lists every place where the reader had to
guess. Each entry gives the question, the candidate readings, the rule IDs
involved and the reading picked. Nothing here was checked with `cpl`.

Count: 57 holes: 28 guesses where the spec is silent or ambiguous
(section A), 23 contradictions between chapters, stubs or examples
(section B), and 6 stdlib gaps (section C: one entry listing
the sequence names the stubs lack, plus 5 names a solution wanted).

## The ten most serious

1. H-LENT-IN-VAR: ch5 says a lent value may be kept in a `var`, a field
   or a container if the storage dies first; ch7 says it never may.
2. H-BSEARCH-DEBUG-PROBES: checked builds call a binary-search predicate
   extra times, so a predicate with effects (an interactive query) acts
   differently in checked and release builds.
3. H-MISSING-SEQ-STUBS: `filter`, `map`, `sort`, `sorted`, `first`,
   `last`, `rev`, `items`, `reverse` and `dedup` have no stub in
   `std/seq.cp`, so their signatures, costs and keys are unknown.
4. H-NONE-CASE: chapter 1 makes the empty optional `None`, but examples in
   chapters 4, 4b and 5 write `none`.
5. H-ARRAY-SCALAR-FILL / H-TABLE-VS-FILL: `var a: [V]Int = 0` fills an
   array (conformance relies on it), but no rule says so; `[s.keys]0`
   reads as either a type or a table.
6. H-DIJKSTRA-W: the weight type of a range-typed weight field, the type
   of `dist`, and how the literal `0` source distance type-checks are all
   unstated.
7. H-COUNTED-IS-DOMAIN: is the counted `[m]` (`[0..m]`, keyed by `Int`) a
   `Domain`? `Graph.directed` needs `E: Domain`, and the overview program
   relies on it.
8. H-NOT-PRECEDENCE: the precedence table puts `not` below comparisons;
   the grammar puts it at unary level, above them.
9. H-INT-RANGE-CUTS: an `Int` range is an "anonymous domain whose
   positions and cuts are both Int", but the integer a cut maps to is not
   defined (`(1..=5).bsearch(x => x >= 3)` could be 2 or 3).
10. H-ONE-NAME-MANY-FORMS: `first`, `last`, `fold` and `min` each appear
    in two or three shapes (getter, predicate, fold with or without
    `init`), but R06.1.6 and R08.5.2 forbid overloading.

## A. Guesses: the spec is silent or ambiguous

H-NONE-CASE. Is the empty optional `None` or `none`?
Readings: (a) only `None`, so `none` is an unknown name; (b) `none` is a
prelude alias; (c) `none` is a keyword. R01.3.1 and R01.3.6 say variants
are PascalCase, with `None` the variant of `?T`. But lowercase `none`
appears in ch4 (`var dist: [V]?Int = none`), 04b (`[a.keys]none`, "`none`
when empty" in doc comments), ch5 exercises 30 and 35, and `none` is a
word in 10+ stubs (`bag.cp`, `bitset.cp`, `dsu.cp`, `graph.cp`). It is
also the `print` option name `none:`. Conformance uses `None` (19 files)
and uses `none` only as the option. Picked (a).

H-ARRAY-SCALAR-FILL. Does `var a: [V]T = x` with scalar `x` fill the
array? R02.8.3 lists only tables, typed reads, comprehensions and
`[D].filled(x)`. R04.2.18 mentions in passing that a literal "also fills
`[d]V.cuts = 0`". Ch5 writes `var seen: [V]Bool = false`, and conformance
`core-04/anon-range-array.cp` relies on `var dp: [0..=5]Int = 0`. Readings:
(a) a general rule (any `T` value fills an expected `[D]T`); (b) literals
only; (c) an error. This rule decides whether `= None` fills `[V]?Int`.
Picked (a), and used it in 03, 09 and 10.

H-TABLE-VS-FILL. What is `[s.keys]0` (R04b.1.5) and `[a.keys]none`
(04b `next_greater`)? Readings: (a) a value: a table over the range filled
with the constant; (b) the type `[s.keys]T` with the literal type `0`, as
in `[V]0..=255`. In type position the grammar makes it (b); in value
position the table grammar is `[D] i => e`, which needs `=>`. Picked
`[r] _ => x` in 08 to avoid it.

H-LITERAL-BOUND. R04.2.18: an integer literal is a position "outside a
static bound it is a compile error". For `domain V = 1..=n` with
`n: 1..=2e5`, which bound is meant: the smallest count (1) or the largest
(2e5)? Is `a[1]` legal, and is `a[5]`? Picked: a literal is legal when it
lies inside the domain at its largest count (a trap at runtime otherwise),
and is proven when it lies inside the domain at its smallest count. QUESTIONS
Q13.

H-INT-RANGE-CUTS. ch4 Ranges says "answer spaces that aren't a domain
are integer ranges, an anonymous domain whose positions and cuts are both
`Int`". For `1..=5`, what Int is the cut before 3: 2 (positions before it,
as `Int(c)` defines for cuts, R04.2.16) or 3 (the position after it,
matching `0..n` ranges)? This decides what `bsearch` and `.mid` return on
`Int` ranges. Picked 2, by R04.2.16. Hit in 05 (`tails.keys.bsearch`, where
both readings agree because the keys start at 0) and Q22.

H-RANGE-READ. R10.2.3: "`Range<T>` reads two ends and checks order".
Inclusive (`l..=r`) or half-open? Statements almost always give inclusive
`l r`. Picked: not used; read `(V, V)` instead (01).

H-BSEARCH-DEBUG-PROBES. ch4: "Debug builds check `ok` on both sides of
the answer (O(1))", so checked builds call the predicate more times than
release builds do. In an interactive task (06) each call is a query, so
the query count and the judge dialogue differ by build. That breaks R09.2
("valid programs behave identically in both modes"). Readings: (a) the
probe is skipped when the predicate has effects (how is that detected?);
(b) it is the contestant's problem; (c) the probe reuses answers already
computed. Picked (b) and noted it in 06. Needs a rule.

H-DIJKSTRA-W. With `w: 1..=1e9`, what are `W`, `dist: [V]?W` and the
type of `0` in `from: [(V.first!, 0)]` (ch0 overview, `graph.cp`
`dijkstra`)? `0` is outside `1..=1e9`, and path sums exceed it. The
overview says sums are "proven not to overflow 2e5 * 1e9", which implies
`W` is widened somewhere, but no rule says where. Readings: (a) `W` is
the join of the weight type and `0..`, widened to `Int`; (b) `W` is the
weight type and `dist` stores wider. Picked (a).

H-COUNTED-IS-DOMAIN. Is `[m]` (= `[0..m]`, R02.8.1) a domain? R04b.1.5
says an array over a range of `K` is "indexed by `K`", which for a counted
dimension is `Int`. But `Graph.directed<E: Domain, ...>(edges: [E]R)`
needs `E: Domain`, and the overview passes `read<[m](u: V, v: V, w: ...)>()`.
Readings: (a) every `[0..n]` is an anonymous domain over `Int`; (b) only
declared domains, so the overview needs `domain E = 0..m`. Picked (a), as
ch4 Ranges says "an anonymous domain".

H-DOMAIN-COUNTED-DECL. R10.2.2 mentions `domain E = [m]` as a counted
domain declaration. The grammar's `domain_decl := 'domain' NAME '=' expr`,
and `[m]` as an expression is a one-element list. Picked: `[m]` is
special syntax in a domain declaration only.

H-ROW-PATTERN. Does a record pattern without `..` match a record with
more fields? The grammar allows an optional `, ..`. Ch5 exercise 30 writes
`for (to: u) in g[v]`, but `graph.cp` examples show the arcs as
`(to: 2, e: 0)`. Readings: (a) closed: `(to: u)` fails on `(to, e)`
arcs; (b) open: extra fields are ignored. Picked (a) and wrote
`(to: u, ..)` in 03, which makes ch5 exercise 30 an error.

H-PRODUCT-DESTRUCTURE. Can a position of a product domain `(R, C)` be
destructured (`let (r, c) = p`) or projected (`p.0`)? Ch4 shows
`for (r, c) in P` and named factors `s.comps` only for named products.
Picked: a tuple pattern works on any product position (09).

H-OPT-COMPARE. `u != from` with `u: V` and `from: ?V` (03). R02.4.5 says
`==` is structural on `?T`, and R02.4.2 allows the implicit `T` to `?T`.
Does the conversion apply to an operand of `==`, or only at bindings and
arguments? Picked: it applies.

H-RECORD-MONOID. `Rooted.reroot(op: M, lift, finish)` needs a monoid
over the DP value. Sum-of-distances needs a pair `(cnt, sum)`, but
R08.1.6 derives only `Eq`, `Ord`, `Hash`, `Print` and `Read`, not `Add`,
and no rule lifts `+` to tuples or records. Picked: wrote the DP by hand
(03). Wanted: a derived or library `Add` for records of `Add` fields.

H-OP-NAMES. R01.5.5 lists `+`, `*`, `min`, `max`, `gcd`, ... as operation
values. `std/algebra.cp` and `std/prelude.cp` also export `add`, used as
`act: add` (segtree stubs) and `t.reroot(add, ...)` (tree.cp example).
Are `+` and `add` the same value? Is `act: +` legal? Picked: `add` is the
named form of `+`, and `act:` takes either.

H-NODE-WIDTH-ACT. ch3 says that with `act:` "there is no bound and nodes
stay `Int`". For `SegTree(min, a, act: add)` with `a: [V]-1e9..=1e9`, is
the node type `Int` without a written annotation (04)? Picked yes.

H-ENUM-TAG-INT. `@tag(1) Add(...)`: R02.7.2 and R10.2.3 say an enum reads
one token and matches tags. Is the integer tag compared as a number or as
text, so does `01` match `@tag(1)`? Picked numeric (04).

H-DROPPED-RESULT. `d.unite(a, b)` returns `Bool`, and 08 drops it. R06.4.8
(`@droppable`) implies an unused-result lint exists, but no chapter
defines it or says whether a dropped `Bool` is a warning or an error.
Only a dropped `?` result is an error (R02.4.7). Picked: a lint, not an
error.

H-YES-WORDS. `yes(b)` uses `io.answers`, which defaults to
`profile.answers` (§9.1). What does the default profile say: `YES`/`NO`,
`Yes`/`No`? Picked `YES`/`NO` (08).

H-LOWER-GENERIC-ARGS. R01.3.4: "`Name<` opens type arguments only when
`Name` is capitalized". The same rule then gives `read<[n]Int>()` as the
explicit form, and the spec writes `s.parse<Mod<P>>()`, `wire.ask<T>`,
`f<T>(...)`. When does `lower<` open type arguments? Picked: when what
follows parses as type arguments closed by `>` immediately followed by
`(`.

H-NEG-RANGE-TYPE. How does `[V]-1e9..=1e9` (or `read<-5..=5>()`) parse?
`range_type := expr? '..=' expr?`, but `[V]` followed by `-` could also
start a binary expression in `dims`. There are no grouping parentheses in
types, because `(T)` is a tuple. Picked: a unary minus is allowed at the
start of a range type (01, 04).

H-INLINE-IF-STMT. `if c == tails.len: tails.push(x) else: tails[c] = x`
(05). `block := ':' simple_stmts NL` needs a newline before `else`, and
the inline `if_expr` branches are expressions, not assignments. R01.7
shows `if c: a elif d: b else: e` as the inline form. Picked: legal as a
statement.

H-MINEQ-COND. R01.6.4 lets `min=` act as a condition
(`if dist[v] min= dist[u] + w:`), but the grammar has `cond := expr` and
`assign` is not an expression. Picked: legal (09).

H-PRELUDE-MEMBERSHIP. R11.3.2: the prelude holds every name in a lib
chapter's "main tables". `lib/strings.md` mentions `z_function` in prose,
but `std/prelude.cp` does not import it. A reader can't tell what counts
as a main table without running `scripts/spec-stub.py`. Picked: trust
`prelude.cp`, so 07 writes `use std.strings.{z_function}`.

H-TAKE-DEFAULT. ch5 §2: `xs[i].take()` "leaves the type's default
behind". No `Default` trait or per-type default is defined. What is left
in a `List<V>` slot, or in a struct with no field defaults?

H-POSITIONS-AFTER. ch4 Open admits there is no direct spelling for "the
positions after `v`" (`for j in v.r..` walks cuts). Hit when writing pair
loops; used `qs.keys`-style loops instead.

H-SORTED-KEYS. Rule R04b.1.6 says materializing keeps keys when it can,
and 04b Open 1 leaves `sorted()` of a slice undecided. R07.11.2 says
`sorted` returns "new `List`". 08 sorts a whole `[0..m]` array, so both
readings agree there, but sorting a slice does not.

H-DIAG-CODES. No chapter in `spec/core` maps diagnostic codes to their
meanings. Only E0601 and E0407 appear inline. Codes in QUESTIONS.md were
taken from conformance file names. (`spec/diagnostics.md` sits outside the
directories this test allowed.)

## B. Contradictions between chapters, stubs and examples

H-LENT-IN-VAR. ch5 R8 (R05.7.8 to R05.7.10) and its §13 table: a lent
value may be bound, stored in a local `var` or field, or pushed into a
container if the storage dies no later than its sources. Exercises 20 and
35 rely on it. ch7 R07.1.5: views are "never stored"; R07.5.10: a lent
value "never goes in a `var`, field, container, global or escaping
closure". Picked ch5, since it calls itself the whole model and
authoritative.

H-LENDS-ARROW. The grammar has `ret := '->' ('yields' | 'lends' lend_src?)? type`,
so `-> lends T`. Ch5 §6, exercises 26, 27 and 33, and ch7 R07.5.10 write
`fn window(xs: [V]Int, l: V) lends [V]Int` with no arrow. Picked: both
accepted (QUESTIONS Q9).

H-EQ-BLOCK. 04b "Contest code" writes `fn solve(r: Range<V>) -> Int =`
followed by an indented block. R01.11.1 lists that exact spelling as
rejected (decision 136).

H-ENUM-ONE-LINE. ch4 writes `enum Dir: N, E, S, W`. The grammar and ch2
have `enum Dir = L | R | U | D` or `enum X:` followed by an indented
list. QUESTIONS Q36 treats the ch4 form as a syntax error.

H-NOT-PRECEDENCE. §1.5 puts `not` at level 12, below comparisons, so
`not a == b` is `not (a == b)`. The grammar has `unary := ('-' | '+' |
'~' | 'not') unary`, which binds tighter than `==`. Picked §1.5.

H-IS-PRECEDENCE. The grammar puts `is pattern` among the postfix
operators; §1.5 doesn't list `is`. `self is Ask` and `x is p and c` work
either way, but `-x is P` and `a + b is P` don't.

H-KEYWORDS. `todo` is a primary expression and is called "an expression"
(R01.6.7), but it is not among the 33 keywords or the contextual words.
`context`, `marker` (R11.1.2) and `uses` (R09.1.3) are not in the
contextual-word table either.

H-DECORATOR-PUB-ORDER. The grammar has `decorator* ('pub'? 'fn' ...)`, so
`@get pub fn`. `seq.cp:759` writes `@get pub fn keys`, but `seq.cp:835`
(`pub @get fn rebase`), `flow.cp:19` and `hashmap.cp:122` write
`pub @get fn`.

H-ARGMAX. ch7 "What goes away" removes `argmin`/`argmax` ("a fold over
`.keys`"), and §Folds says "No `argmin` ... twins". But 04b library code
defines `pub fn argmax`, examples use `a[r].argmax()!` (04b B1, ch7
§Slices), ch5 exercises 9 and 32 use `rows.argmax(by:)` and
`evens.argmax()`, ch8 uses `argmax` as its generics example, and
`seq.cp:1490/1499` stub both. Picked: they exist.

H-ONE-NAME-MANY-FORMS. R06.1.6 and R08.5.2 forbid overloading. But
`w.first` is a getter returning `?T` (R04b.1.4), `s.first(p)` is a
predicate scan (R07.9.5), `r.first` on ranges is `?V` (R04.4.14) and
`V.first` is `?V`. `fold` is `s.fold(op) -> T | ?T` in R07.9.2,
`fold(f, init)` in `seq.cp:1010`, and a free `fold` exported from
`algebra` in the prelude. `min`/`max` are binary builtins and methods
(R11.3.3 accepts that one explicitly). Readings: `first` is one fn with
an optional predicate (`p: ?(T -> Bool) = None`) whose result type varies;
or two names. Picked: one fn with a defaulted predicate. The stubs don't
say.

H-PREFIX-API. ch4 writes `pre.fold(j..i)`; ch7 writes `pre[r.r] -
pre[r.l]` and `prefix(a).sum(r)` ("`+` only"); `prefix.cp:69` stubs
`Prefix.sum(r)` and no `fold`. One job, three spellings, and which one
exists is unclear. Picked `pre[r] - pre[l.l]` (01).

H-COLLECT-KEYS. R04b.1.6: `.collect()` of an indexed view gives
`[w.keys]T`. `seq.cp:859`: `collect<S: Seq, T>(self: S) -> List<T>`, which
is always a `List`.
Resolved: R04b.1.6 and R07.4.4 now say `.collect()` builds a `List`
keyed from 0 (decision 186), which is what `seq.cp` declares; keys are kept
with `[w.keys] k => w[k]`.

H-Z-KEYS. R04b.5.3 says `z_function` returns `[K]Int` keyed by
`s.keys`. `strings.cp:297` has `z_function<S: Indexed, K>(s: S) -> [K]Int`,
where `K` is not tied to `S` (it should be `S: Indexed<K, T>`).

H-STR-FIND-STUB. R04b.5.1 says `find` becomes `-> ?K` on any
`Indexed<K, Char>`; `strings.cp:55` still has `find<P>(self: Str, p: P) -> ?Int`.

H-SEGTREE-DOMAIN-ARG. Ch5 exercise 20 and R8 write
`SegTree(V, op: (x, y) => ...)`. `SegTree.new(op:, a:, fill:, act:, over:)`
has no positional domain parameter; the stub examples write the domain as
a type argument (`SegTree<0..4, Int>(max, fill: 5)`).

H-DSU-SAME-MUT. `Dsu.find(self)` uses an `@cache` parent (ch5 §10a) so it
takes a read `self`, but `dsu.cp:85` has `same(mut self, ...)`, while
`ParityDsu.same(self)` and `PotentialDsu.same(self)` take read `self`. A
`let d` can call `find` but not `same`.

H-ORDER-NAME. ch8 `trait Ord: fn cmp(self, other: Self) -> Order`, but
its prose says `cmp` returns an `Ordering` (`Less`, `Equal`, `Greater`),
and `algebra.cp:311` and the prelude export `Ordering`.

H-MONOID-ID. §1.9 shows `fn id() -> T` and `law left_id(x: T) =
op(id(), x) == x`; ch8 shows `fn id(self) -> T` and
`self.op(self.id(), x)`.

H-STD-FUNCTION. ch5 §13a: "Closures of different types in one container
cost a visible `std::function`." R06.2.5 and R13.3.2: "no `std::function`
and no indirect call".

H-MINT. ch5 R1 lists `Mint` among plain types; everywhere else the type is
`Mod<P>` and chapter 2 says there are exactly four number types.

H-DERIVE. R10.3.5 starts "`#derive`-free". `#` isn't syntax (R01.1.2),
and R08.1.6 derives structurally, so the phrase points at nothing.

H-REBASE-STUB-FORM. R04b.1.7 makes `w.rebase` a getter with a fresh
`t.Idx` domain; `seq.cp:835` returns `Rebased<S>`, with no `Idx`
associated domain visible.

H-STALE-OPEN. R04b.5.8 renames `nth_element` to `select`; conformance
`nth-element-renamed` agrees, and `seq.cp:165` has `select`. Fine, but
04a and 04b Open still argue about `.enumerate()`, which 07 says
decision 142 settled. The Open sections are stale.

## C. Stdlib names wanted or promised but missing

H-MISSING-SEQ-STUBS. These names are used throughout ch4 and ch7, but
`std/seq.cp` has no `pub fn` stub for them: `filter`, `map`, `sort`,
`sorted`, `reverse`, `dedup`, `deduped`, `first`/`first(p)`,
`last`/`last(p)`, `rev`, `items`. `count`, `min`, `max`, `sum`, `take`,
`drop`, `nth`, `windows`, `take_while`, `first_true`, `last_true`,
`bsearch`, `rekey`, `rebase`, `with_domain` and `lis` do exist. Without
stubs, their key behaviour (does `sorted` keep keys?) and costs are
undocumented.

Wanted while writing the solutions:
- a derived or library `Add` for records and tuples (H-RECORD-MONOID);
  rerooting sum-of-distances needs it;
- a grid BFS helper over a product domain with 4/8 neighbours. `grid.cp`
  has `Dir4` and `Dir8`, but a reader can't find a `bfs` that takes a
  `[R, C]Char` with walls, so 09 is hand-rolled;
- `Range<V>` reads with an inclusive/half-open choice (H-RANGE-READ);
- a documented way to get a domain for a counted read (`domain E = [m]`,
  H-DOMAIN-COUNTED-DECL).

## Readings used in the solutions, by file

- 01: H-NEG-RANGE-TYPE, H-PREFIX-API, H-RANGE-READ (avoided).
- 02: H-COUNTED-IS-DOMAIN, H-DIJKSTRA-W.
- 03: H-ARRAY-SCALAR-FILL, H-ROW-PATTERN, H-OPT-COMPARE, H-RECORD-MONOID.
- 04: H-NEG-RANGE-TYPE, H-ENUM-TAG-INT, H-OP-NAMES, H-NODE-WIDTH-ACT.
- 05: H-INT-RANGE-CUTS, H-INLINE-IF-STMT.
- 06: H-BSEARCH-DEBUG-PROBES, H-LOWER-GENERIC-ARGS (`wire.ask`), and
  whether a literal-union return type (`-> "Y" | "N"`) validates the
  answer it reads (R02.2.2 says reads validate; picked yes).
- 07: H-Z-KEYS, H-PRELUDE-MEMBERSHIP.
- 08: H-TABLE-VS-FILL, H-DSU-SAME-MUT, H-DROPPED-RESULT, H-YES-WORDS,
  H-SORTED-KEYS.
- 09: H-PRODUCT-DESTRUCTURE, H-ARRAY-SCALAR-FILL, H-MINEQ-COND.
- 10: H-ARRAY-SCALAR-FILL (over a runtime anonymous range).

## Status

Fixed in the spec (decision in parentheses): H-LENT-IN-VAR (179),
H-BSEARCH-DEBUG-PROBES (180), H-NONE-CASE, H-NOT-PRECEDENCE,
H-ENUM-ONE-LINE (181), H-MISSING-SEQ-STUBS, H-ONE-NAME-MANY-FORMS (182),
H-ARRAY-SCALAR-FILL, H-TABLE-VS-FILL, H-OPT-COMPARE, H-LITERAL-BOUND,
H-DIJKSTRA-W, H-COUNTED-IS-DOMAIN, H-INT-RANGE-CUTS (183), H-LENDS-ARROW,
H-DECORATOR-PUB-ORDER, H-ORDER-NAME, H-MINT, H-EQ-BLOCK, H-IS-PRECEDENCE,
H-KEYWORDS, H-MONOID-ID, H-ARGMAX, H-DERIVE, H-SORTED-KEYS (184). H-RANGE-READ, H-DOMAIN-COUNTED-DECL, H-ENUM-TAG-INT, H-YES-WORDS,
H-ROW-PATTERN, H-PRODUCT-DESTRUCTURE, H-RECORD-MONOID, H-OP-NAMES,
H-NODE-WIDTH-ACT, H-DROPPED-RESULT, H-LOWER-GENERIC-ARGS,
H-INLINE-IF-STMT, H-NEG-RANGE-TYPE, H-MINEQ-COND, H-PRELUDE-MEMBERSHIP,
H-TAKE-DEFAULT (185). H-PREFIX-API, H-STR-FIND-STUB,
H-COLLECT-KEYS, H-REBASE-STUB-FORM, H-DSU-SAME-MUT, H-SEGTREE-DOMAIN-ARG,
H-STD-FUNCTION, H-POSITIONS-AFTER, H-DIAG-CODES (186); H-STALE-OPEN
(the `.enumerate()` item is marked settled).
H-Z-KEYS: `z_function`, `prefix_function` and `prefix_matches` now take
`S: Indexed<K, T>`, so `K` is the input's key type.

