# Rebuild round 2: holes

I worked from `spec/` at 239c87c3. I did not open `spec/delta/`, and I read
no other part of the repo. A hole is any place where I had to guess, even
when the intent was clear.

## Summary

There are 32 holes:

- silent (the spec says nothing): 12
- contradiction (two places disagree): 15
- missing-stdlib (a name is used or needed but has no stub in `spec/std`): 5

These are the ten a contestant is most likely to hit, most likely first:

1. H-CORE-CONTAINERS: `List`, `Str`, `Map` and `Set` have no stubs, and
   `Map` and `Set` are not in the prelude.
2. H-LET-ELSE-OPTIONAL: it is unclear whether `let x = opt else:` unwraps a
   `?T`.
3. H-RUNTIME-RANGE-TYPE: it is unclear whether a range type can have
   runtime ends, as in `read<1..=n>()`.
4. H-FIND: `find` means two different things, and the spec has no
   overloading.
5. H-INFER-LATER: `Window(min)` needs a later statement to infer its type.
6. H-GRAPH-NEXT: `Graph` does not implement `Next`, but `topo(g)` and
   `Rooted(g)` need it.
7. H-SORT-DESC: chapter 7 says descending is a key, but the `sort` stubs
   take a `desc:` flag.
8. H-RECORD-WIDEN: it is unclear whether widening reaches inside records,
   tuples and sequences.
9. H-NOT-IN: there is no `not in` operator.
10. H-ABS: `abs` is used in the spec but has no stub.

## Silent

### H-RUNTIME-RANGE-TYPE
Question: is a range type with a runtime end legal, as in
`let k = read<1..=n>()` with `n` read from input?
Readings: (a) yes, it is a contract checked on read, with `Int` storage;
(b) no, range types need static ends, so you would read `Int` and
`assert`.
Rules: R02.2 (range types), R03.1 (the width table says "static ends"),
R03.1.3 (fields need static bounds), R03.1 (`0..=V.count` is allowed).
Picked: (a). Used in 02 and 09.

### H-NOT-IN
Question: is `x not in s` an operator?
Readings: (a) yes, as in Python; (b) no, write `not (x in s)`.
Rules: R01.5 (the precedence list has `in` only), R01.10 grammar.
Picked: (b). Used in 03.

### H-LET-ELSE-OPTIONAL
Question: does `let x = opt else: continue` bind the `T` inside a `?T`, or
do you need `let Some(x) = opt else:`?
Readings: (a) a bare name unwraps, as `if let v = m.get(k)` does in R01.7;
(b) a lowercase name always binds (R01.3.3), so the pattern is irrefutable,
`x` is a `?T`, and the `else` can never run.
Rules: R01.6.2, R01.7.1, R02.4.2, R01.3.3.
Picked: (a). Used in 03 and 10.

### H-PARTIAL-TYPE-ARGS
Question: can you give some type arguments and leave others to inference,
as in `Window<Int, _>(max)`, or `Window<Int>(max)` when `M` has no
default?
Readings: (a) `_` is a type hole (`std/tree.cp` returns `Rooted<V, _>`);
(b) only trailing defaults may be omitted.
Rules: R08.2.5, R01.10 grammar (it has no `_` type).
Picked: avoided it by writing `Window(max)`. That runs into H-INFER-LATER.

### H-RECORD-WIDEN
Question: does conversion 2 (a range type to a wider one) apply inside a
tuple, a record, an array or a sequence? Examples: pushing
`(x: s.first!, d: 1)` into a `List<(x: Int, d: Int)>`, passing a
`[V]0..=9` where a `[V]Int` is expected, and
`chain(Seq<0..=1e18>, Seq<Int>)`.
Readings: (a) it applies fieldwise and elementwise; (b) only to a bare
scalar.
Rules: R04.4.22, R04.4.24, R02.10.1.
Picked: (b), to be safe. Solution 07 wraps values in `Int(...)` so that
`chain` gets two sequences of one element type.

### H-SIBLING-RANGE-UNIFY
Question: what type do sibling range types unify to? Example:
`if c: 1 else: 5`, or `[read<0..=9>(), 100]`.
Readings: (a) the hull, `1..=5`; (b) `Int`.
Rules: R02.10.1 says siblings are unified but not how ranges join;
R02.10.2.
Picked: (a).

### H-MOD-PRINT
Question: how does a `Mod<P>` print? `std/mod.cp` has no `Print` impl, and
the conformance case `mod-closed` expects `1`.
Readings: (a) it derives structurally from its one field `v`, giving the
residue (R08.1.6, R10.3.5); (b) it is core-special, like reading
(R10.2.3).
Picked: (a). Used in 08.

### H-FREE-SELF-FN
Question: a free fn whose first parameter is `self: S`, such as
`pub fn sum<S: Seq>(self: S)` or `find_all(self: S, ...)`, is called as
`xs.sum()`. Is `sum(xs)` legal too? The prelude imports `sum`, `count`,
`all` and `any` as free names. Does `self: T` type the receiver?
Readings: (a) both call forms work; (b) only the method form works.
Rules: R01.10 grammar (`param := 'mut'? (pattern | 'self') (':' type)?`),
R11.3.3 ("the folds are methods"), R12.4.1.
Picked: (b).

### H-RANGE-READ-ANON
Question: is `read<[n]Range<0..=1e18>>()` valid? An integer range is used
as `D`, and `s.first!` would be typed `0..=1e18`.
Readings: (a) yes, by R04.4.28 together with R10.2.3; (b) `Range<T>` reads
only over declared domains.
Picked: (a). Used in 07.

### H-OPT-FOLD
Question: is `.min()` over a `Seq<?Int>` legal? `?T` is not `Ord`
(R02.4.5), but `min` and `max` treat `None` as the identity (R02.4.6).
Readings: (a) it is an error; (b) the fold skips `None`, giving a `?Int`.
Picked: avoided it with a `var tour: ?Int` and `min=` in 03.

### H-MEMO-SUBSET
Question: can `@memo(array)` take a `Subset<B>` parameter?
R06.4.1 lists "domain, cut, range, `Bool`, enum". `lib/search-dp.md`
says `B.subsets` (written `Subset<B>`) "is the domain".
Readings: (a) yes, since it is a domain; (b) no, since it is a struct over
`U64`.
Picked: (a). The bottom-up form in 03 avoids the question.

### H-DUP-SPELLINGS
Question: which spelling is canonical when two do one job?
- `a..b by k` (R01.8.4) and `r.step(k)` (R04.4.13, R07.7.11).
- `a.partition_point(ok)` (R07.12.5), `a.keys.bsearch(ok)` (R04.4.20) and
  `first_gt` or `last_lt` (both in the prelude).
Picked: `by` and `bsearch`.

## Contradiction

### H-TURBOFISH
R01.3.4 says "A lowercase generic function takes its type arguments from
the expected type, or explicitly as `read<[n]Int>()`", and
`DREAM-BRIEF.md` says "Generic calls are `f<T>(...)`".
`std/graph.cp` (topo doc) says "pass the domain: `topo::<V>(u => ...)`".
Picked: `f<T>()`.

### H-RANGE-START-END
`DREAM-BRIEF.md` says "ranges have `.start`, `.end` (exclusive), `.len`,
`.rev`".
04-indices.md says "Rejected: `.start`/`.end`/`.before`/`.after`", and its
tables use `.l`, `.r`, `.first` and `.last`.
Picked: chapter 4.

### H-AFTER
R04.2.19 says "`v.after`, `v.before` | the pieces `v.r..` and `..v.l`",
and 04-indices.md Open says "(The positions after `v` are `v.after`,
decision 186.)"
The same file, at line 316, still says "Rejected:
`.start`/`.end`/`.before`/`.after` (four words for one idea)".
Picked: `v.after` exists.

### H-FIND
R07.9.5 says "`s.find(p)`, `s.rfind(p)` | `?T`, linear scan", which is a
predicate that returns an element.
04b-keys.md and `std/strings.cp` say "`find` ... `-> ?K` on any
`Indexed<K, Char>`", which is a pattern that returns a key. The prelude
imports `find` from `std.strings`, and there is no overloading (R06.1.6).
04b's own example also calls `s[l..].search(',')`, and `search` has no
stub.
Picked: `a.keys.find(i => ...)` for predicates. I did not use `find`.

### H-SORT-DESC
07-sequences.md says "Descending is a key, not a flag:
`by: x => desc(x.score)`".
`std/seq.cp` has `sort(..., desc: Bool = false)` and
`sorted(..., desc: Bool = false)`.
Picked: `desc(...)` as a key.

### H-GRAPH-READ
R10.2.5 says "there is no `Graph.read`".
12-powers.md W11 has "`read<Graph<V, E>>(directed: false, weighted:
true)` is ordinary library code", and R02.6.2 uses `Graph.read()` as its
example of a custom constructor.
Picked: read the edge records, then call `Graph.directed` or
`Graph.undirected`.

### H-STRUCT-PAREN-HEADER
The grammar (R01.10) is `struct_decl := 'pub'? 'struct' TYPE_NAME
generics? (':' fields NL | ...)`, with no parenthesized header and no
`let` in a body. The stubs use both:
- `pub struct Mod<P>(v: 0..P):` with `let MOD = P`
- `pub struct Wire():`
- `pub struct Window<T, M>(op: M):`
- `pub struct Subset<B>(bits: U64):`
- `pub struct Mapped<S, U>():`
Picked: read them as private fields and constants.

### H-RUNTIME-MOD-SPELLING
There are four spellings of a runtime modulus:
- 02-values-types.md §2.9: `type Z = Mod(read<Prime>())`
- R03.5.1: "`Mod<p>` with `p` read once"
- diagnostics E0801: "use `type Z = Mod(p)`"
- `std/mod.cp`: "`let P: Int = read()`, then `type Z = Mod<P>`"

The last one also breaks R01.3.2 (E0208, a capitalized value).
Picked: none of my solutions needed it.

### H-INFER-LATER
R02.10.1 says "Inference never looks at later statements: `var x = None`
followed by `x = 3` is an error".
`lib/ops-ranges.md` writes `var w = Window(min)`, and the element type `T`
appears only in the later `w.push(a[i])`.
Picked: the ops-ranges form. Used in 02.

### H-EMPTY-SUBSET-BRACES
`std/subset.cp` and `lib/search-dp.md` say "`{}` is the empty subset"
(`best[{}] = 0`).
R01.2.5 and the grammar make `'{' block_expr '}'` an inline block, so `{}`
would be an empty block.
Picked: `Subset<C>.empty`, and `single` and `full`.

### H-LIST-CTOR
05-ownership.md R2 has `var xs = List(1, 2)`, two elements.
07-sequences.md has `List(s)` as "conversions of anything that isn't a
chain: `List(0..n)`". So it is unclear whether `List(5)` is `[5]` or an
error, and there is no overloading.
Picked: list literals, plus `List(seq)` for conversions only.

### H-DROP-SEQ
R07.7.3 says "`s.take(k)`, `s.drop(k)` | an O(1) view on an `Indexed`; a
`Seq` otherwise".
`std/seq.cp` declares only `pub fn drop<S: Indexed>(self: S, k: Int = 1)`.
Picked: `drop` only on indexed sequences. Used in 09.

### H-PREFIX-FN-LIB
`lib/strings.md:162` has `fn prefix_function<S: Indexed>(s: S) ->
List<Int>`.
`std/strings.cp` (fixed in 239c87c3) and R04b.5.3 have `-> [K]Int`, keyed
by `s.keys`.
Picked: the stub. Used in 06.

### H-MEMO-EXAMPLE
`lib/search-dp.md`'s `@memo` example declares `-> Mod`, with no modulus,
and returns `Mod(sum % k == 0)`, which builds a `Mod` from a `Bool`.
R02.11.1 says "No implicit numeric conversions", and `Mod.new` takes an
`Int`.
Picked: treat it as an example bug.

### H-DIAG-CODES
R01.4.6 says "`++` is a lex error". The conformance case `plusplus`
expects E0004, but the catalogue describes E0004 only as "`#` is not a
comment". Three more codes are unclear:
- `'ab'` (R01.4.3) has no listed code. E0007's example is `'é'`.
- An `unchecked:` block used as a value (R01.3.7) has no listed code.
- A spare value in input (R03.3.4) could be R0501 or R0502.
Picked: see Q5, Q11 and Q23.

## Missing-stdlib

### H-CORE-CONTAINERS
`List`, `Str`, `Map` and `Set` are used throughout the spec, for example
`List<T>()`, `xs.push`, `m.get(k)`, `m[k] += 1` (DREAM-BRIEF),
`Set(s)`, `s.push(c)`, `xs.reserve`, `.take()` and `.replace(x)`. None of
them has a stub in `spec/std`; `grep "struct List\|struct Map"` finds
nothing. R11.3 says the compiler has no built-in name list. Even so,
`Map` and `Set` are not in `std/prelude.cp`.
Picked: assumed they are core and always in scope.

### H-GRAPH-NEXT
`topo<V>(g: Next<V>)` and `Rooted.new<N>(next: N, root: V)` take a
`Next`. `Graph` declares `fn next(self, u: V) -> yields V`, but
`std/graph.cp` has no `impl Next<V> for Graph`. R02.6.5 says named
structs satisfy traits only through `impl`.
Picked: assumed `Graph` implements `Next`. Used in 01 and 05.

### H-ABS
06-control-functions.md §6.1 writes `abs(a.x - b.x)`, and `std/math.cp`
uses `abs(d)` in docs. No free `abs` is stubbed; there are only
`Ratio.abs` and `BigInt.abs`.
Picked: assumed a prelude `abs`. My solutions do not use it.

### H-POPCOUNT
Bit counting on `Int` or `U64` has no stub: popcount, trailing zeros and
lowest bit. Only `Subset.len`, `Subset.first` and `bit_len` exist.
Picked: used `Subset` in 03.

### H-SUBSET-CONTAINS
`b in s` on a `Subset` (lib/search-dp.md) needs `Contains`, as Q37 and
R12.4.7 show. `std/subset.cp` has a `contains` method but no
`impl Contains`.
Picked: assumed `in` works. Used in 03.
