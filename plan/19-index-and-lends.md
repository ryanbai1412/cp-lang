# 1.10 One mechanism for `domain`, `Str`, `Grid` and `Perm`: bounded indices and lends

Status: **Decided.** Ryan approved this design on 2026-11-20, with three
changes that are folded in below:

- **`let K = n`, not `const K = n`.** An index type sized by a run-time
  value writes its size as an associated `let`. `const` stays strictly
  compile-time (decision 7.10). See §3.1.
- **`v.l` and `v.r`** are the boundaries on the left and right of a
  position, with `v.l(k)` and `v.r(k)` for a window `k` wide. They are the
  short spellings of `v.before` and `v.after`, which remain as the long
  names. See §3.1 and §4.1.
- **`Char` and `Str` are ASCII only.** A `Char` is one byte, a `Str` is a
  flat array of `Char`, and there is no UTF-8 anywhere in the language.
  See §4.4.

What is built: `proto/cpc/` prototypes the surface rules (§11 says exactly
which, and what it leaves out). The Rust compiler does not implement this
yet. No benchmark was run for this document, so every cost claim is still
a design target to verify (§9), not a measurement.

It replaces the parts of `12-domains-and-sequences.md`, `13-abstraction.md`
and `14-state.md` that describe domains, slices, grids, strings and
permutations as separate compiler features. Where those documents disagree
with this one, this one wins. The plan spells the mutable mode `mut` at
both the parameter and the call site (the prototype also parses `inout`).

---------------------------------------------------------------------------

## 1. The proposal in one page

Today the compiler knows four index-shaped things separately:

- `domain V = 1..=n`, with its bounds, ranges, `Sub<V>` and slices;
- `Str`, an array of `Char` that somehow brings its own domain (1.6);
- grids, `[Row, Col]T` with `g[r, c]`, `g.row(r)` and `p.step(d)`;
- `Perm<V>`, an array the compiler treats as a special kind of `[V]V`
  (the prototype has a `TPerm` type next to `TDom`, `TArr` and `TStr`).

The proposal replaces them with two ideas, each small and general, and
writes all four as ordinary library code on top.

**A bounded index** is any type with a known, finite set of values that map
densely onto `0..K`. That is the existing `Index` trait (M7). A `domain` is
nothing more than a nominal struct that wraps a range type, seals its
constructor, and implements `Index`. The compiler does not know what a
domain is.

**An indexed slice** is what you get when a container lends you part of
itself: one element, a range of elements, a grid row, a string. Lending is
one new construct, a *projection*: a function declared with `lends` instead
of `->`. A projection is defined as sugar for a call with an `inline`
callback (5.4), so it needs no lifetimes and never stores or returns a
borrow. `a[i]`, `a[s]`, `g.row(r)` and `s[r]` are all projections written
in the library.

```
domain V = 1..=n                       // sugar for a sealed struct + impl Index
let a: [V]Int = read()                 // [I]T is Array<I, T>, a library struct
let g: [Row, Col]Char = read()         // Array<(Row, Col), Char>; tuples of Index are Index
let s: Str = read()                    // [domain]Char: an array that brings a fresh index type
let p: Perm<V> = read()                // library struct over [V]V with a sealed constructor

a[v] += 1                              // IndexedMut projection, written in the library
a[l..=r].sort()                        // Slice projection, written in the library
swap(mut g.row(r1), mut g.row(r2))     // allowed when the checker knows r1 != r2
```

What the compiler keeps is listed in §7: range types, sealed nominal
structs, traits with associated `const`s and frozen `let`s, fresh index binders, the
`Indexed` operator traits, projections with the exclusivity checker, and
`Buffer<T>`. None of them mentions domains, strings, grids or
permutations.

The rest of this document is: what is hardwired today (§2), the mechanism
(§3), the four features rewritten in userland (§4), the check against the
borrow rules (§5), checks and cost (§6), what the compiler keeps and drops
(§7), the rows of `decisions.md` that change (§8), how to test it in the
prototype (§9), and open questions (§10).

---------------------------------------------------------------------------

## 2. What is hardwired today

Reading the plan and the prototype, these are the places where the
compiler, not the library, defines behaviour.

**Domains** (12 §1, decisions 2.1 to 2.9). The declaration `domain V =
1..=n`, the 4-byte ordinal, the label, unchecked `[V]T` indexing, `V(x)`,
`x in V`, `.next` / `.prev`, `V.cuts`, `Range<V>`, `Sub<V>` and its
implicit conversion to `V`, product domains and fresh domains returned by
functions. The prototype's `compiler.py` refers to `TDom` 56 times.

**Slices** (12 §3, decision 2.4). `a[s]` is "a borrow of `a` plus the
range". Slices and `a.map(f)` views already live under a rule no user type
can use: "it can be a local `let` or an argument, but it can't be stored
in a struct or outlive `a`" (12 §4). That rule is the one this proposal
makes available to library code.

**Grids** (12 §1.7, 13 G7). `g[r, c]` as "the only indexing form",
`g.row(r)` as a no-copy view, and `step`. Product domains are already
library-shaped (`Index` on tuples, M7), but two-argument indexing and row
views are not.

**Strings** (11, decision 1.6). "`Str` is a byte array with its own
domain." The plan does not yet say where that domain comes from, what its
name is, or what `List<Str>` means when each string has a different
length. §4.4 answers those.

**Permutations** (18 §1). `Perm<V>` appears only as an input type that
knows how to read, validate, generate and shrink itself. The prototype
makes it a subclass of the array type inside the compiler.

**Disjoint element access** (5.3, 14). "Fields are borrowed separately,
indices are not", with `swap` and `a.pair_mut(i, j)` as the two blessed
ways out. `pair_mut` cannot be written in the language today, because
nothing can hand back two places at once.

There is also one soundness gap. 12 §3 says a slice's positions are
`Sub<V>` and that "indexing the slice with a `Sub<V>` needs no check". But
`Sub<V>` is one type for every sub-range of `V`, so a position taken from
`a[s]` could index `a[t]` with no check. §4.2 closes this.

---------------------------------------------------------------------------

## 3. The mechanism

### 3.1 A bounded index is a sealed struct over a range type

Range types already exist (11): `0..n` is an integer with a declared
range, stored in the smallest width that fits, checked where a value
enters the type. A domain is a nominal wrapper around one:

```
// what `domain V = 1..=n` expands to; this is ordinary code
struct V(ord: 0..n):                       // the constructor is private to this body
    pub fn new(x: Int) -> V = V((x - 1).as<0..n>())     // V(x): checked, crashes with x and "V"
    pub fn try(x: Int) -> ?V = if x in 1..=n: some(V(x - 1)) else: none
    pub @get fn label(self) -> Int = self.ord + 1

impl Index for V:
    let K = n                              // evaluated once, here, and frozen
    fn to_idx(self) -> 0..K = self.ord
    fn from_idx(i: 0..K) -> V = V(i)

impl Seq for V                             // ordered positions; everything else is a default method
impl Show for V: fn show(self) = show(self.label)
impl Read for V: fn read() = V.new(read())
```

Three existing features carry all the weight.

- **Nominal structs.** Two domains over the same numbers are different
  structs, so `deg[e]` with `e: E` is the ordinary "expected `V`, got `E`"
  type error. No domain-specific rule.
- **A sealed constructor.** Only the code in the struct's own body can
  build a `V`, and each way in (`new`, `try`, `from_idx`) proves or checks
  the range. That is the whole meaning of "a `V` can only exist if it is in
  range" (12 §1.1).
- **An associated `let` that freezes a value.** `let K = n` inside an
  `impl` is evaluated once, where the impl is declared, and never changes
  afterwards. It may read any immutable `let` in scope, so a domain can be
  sized by an input value. It is not a `const`: `const` means "known at
  compile time" everywhere in the language (7.10), and `n` is not. The two
  spellings keep that line sharp:

  ```
  impl Index for Digit:  const K = 10        // compile-time: arrays of Digit can be sized statically
  impl Index for V:      let K = n           // run-time, frozen: n was read from the input
  impl Index for V:      const K = n         // error: `const` is compile-time only; write `let K = n`
  var m = n
  impl Index for W:      let K = m           // error: K can't be sized by a var; freeze it first: let k = m
  ```

  A trait declares the member once (`K: Int`) and each impl chooses
  `const` or `let`. Code generic over `I: Index` reads `I.K` the same way
  in both cases; when it is a `let`, the value travels as one hidden
  integer, the same `n` a C++ solution keeps global.

`Index` is M7's trait with `K` declared as above. One trait is added for index types whose
values are positions in a line, which is what `.next`, bounds and ranges
need. Enums, tuples and masks are `Index` but not `Seq`.

```
trait Seq: Index + Ord                     // positions in a sequence
    fn next(self) -> ?Self = Self.try_at(self.to_idx + 1)
    fn prev(self) -> ?Self = Self.try_at(self.to_idx - 1)
    fn offset(self, d: Int) -> ?Self = Self.try_at(self.to_idx + d)
    fn cyc(self, k: Int) -> Self = Self.from_idx((self.to_idx + k) % Self.K)
    @get fn l(self) -> Cut<Self> = Bounds.from_idx(self.to_idx)          // the boundary left of v
    @get fn r(self) -> Cut<Self> = Bounds.from_idx(self.to_idx + 1)      // the boundary right of v
    fn l(self, k: Int) -> Cut<Self> = Bounds.at(self.to_idx + 1 - k)     // left end of the k-wide window ending at v
    fn r(self, k: Int) -> Cut<Self> = Bounds.at(self.to_idx + k)         // right end of the k-wide window starting at v
    @get fn before(self) -> Cut<Self> = self.l                           // the long names
    @get fn after(self) -> Cut<Self> = self.r
    type bounds = Cut<Self>             // so `V.cuts` keeps its spelling
```

**Boundaries are written `v.l` and `v.r`.** A position has a boundary on
each side, and prefix sums, ranges and insertion points all live on
boundaries, so these two are written constantly. `v.l(k)` and `v.r(k)`
reach `k` positions out: `v.l(k)` is the left end of the window of `k`
elements that ends at `v`, and `v.r(k)` is the right end of the window of
`k` elements that starts at `v`. So `v.l` is `v.l(1)` and `v.r` is
`v.r(1)`, and the sum of the last `k` elements up to `v` is

```
pre[v.r] - pre[v.l(k)]
```

with no `- 1` or `+ 1` to get wrong. `v.l(k)` and `v.r(k)` are checked
like `v.offset`: leaving the array crashes with the position, `k` and the
domain's name.

`Bounds.from_idx(self.to_idx + 1)` inside `r` type-checks because
`to_idx` returns `0..K` and `Cut<Self>.K` is `K + 1`: the range checker
proves it, with no trusted code.

`v + 1` was an error here (2.9), for the plainest reason: `V` has no
`impl Add`. The ergonomics audit leans the other way (decisions.md §11):
`Seq` provides `impl Add<Int>` whose result is `?V`, the body of
`v.offset(k)`, so a step that leaves the domain is `none` and never a
position outside it.

**The `domain` line stays.** `domain V = 1..=n` is the declaration people
write dozens of times, so it keeps its one-line spelling (G16). It is
defined as sugar for the struct above, in the same way `for` is defined by
the `Iterator` trait: the parser knows the shape, the meaning is library
code. The label can be any ordered scalar, which is how the prototype port
`nccc10s2.cp` already uses it:

```
domain L = 'A'..'A' + k          // label is a Char; ord is 0..k, one byte when k <= 255
```

`if x in V:` is sugar for `if let x = V.try(x):`, a shadowing rebind, so
flow refinement of positions needs no flow typing in the checker.

### 3.2 Fresh index types: `domain D` as a binder

`compress`, `scc`, `freeze` and Euler order already return fresh domains
(2.5), and signatures such as `fn solve(domain V, domain E, ...)` already
appear in 13. The proposal names the general rule once.

`domain D` in a signature or a binding introduces a new index type whose
size is fixed by the value it comes with. Each call site gets its own
type, and the type cannot leave the block that introduced it:

```
pub fn compress(xs: List<Int>) -> (domain X, vals: [X]Int, rank: List<X>):
    let vals = xs.sorted().dedup()
    domain X = 0..vals.len                 // an ordinary domain, declared in the fn body
    (X, vals.freeze_as::<X>(), [X.at(vals.lower_bound(x)) for x in xs])

let (X, vals, rank) = compress(xs)
let (Y, vals2, rank2) = compress(ys)
vals[rank2[0]]                             // error: vals is indexed by X, rank2[0] is a Y

var keep: ?X = none                        // fine: same block as X
```

The escape rule is the standard one for a locally introduced type: a value
whose type mentions `X` can't be stored in a variable declared outside
`X`'s block, returned without re-exporting `X`, or put in a struct that
outlives it. That also makes the loop case sound: a `compress` call inside
a loop makes a new `X` per iteration, and nothing typed by the previous
iteration's `X` can still be alive.

A fresh index type costs nothing new at run time. `K` is one hidden local
next to the binding; a generic fn over `<D>` whose `K` isn't a compile-time
constant receives it as one hidden integer argument, the same `n` a C++
solution passes or keeps global.

This binder is the piece `Str` needs (§4.4).

### 3.3 Indexing is two traits, and `[I]T` is a library struct

`+` is `Add.add` and `<` is `Ord.lt` (M2). Indexing joins them:

```
trait Indexed<I>:                          // a[i]
    type Elem
    fn at(self, i: I) lends Elem           // a projection, §3.4

trait IndexedMut<I>: Indexed<I>            // derived when the impl's `at` says `lends mut`
```

`a[i]` is the projection `a.at(i)` everywhere. In a value position the
lent element is copied out when `Elem: Copy` (`let x = a[i]`, `a[i] + 1`)
and used in place otherwise (`adj[v].len`, `for u in adj[v]`). In a place
position (`a[i] = x`, `a[i] += x`, `f(mut a[i])`, `a[i].push(x)`) the
container must be `IndexedMut`. `a[i, j]` is sugar for `a[(i, j)]`, so
two-argument indexing is not a grid rule: it works for any container
indexed by a tuple.

`Indexed` needs an associated type for `Elem`. Decision 3.8 has waited for
"a real need"; this is it, since without it one container could claim two
element types for the same index.

`[I]T` is sugar for `Array<I, T>`, and `[A, B]T` for `Array<(A, B), T>`:

```
pub struct Array<I, T> where I: Index:
    buf: Buffer<T>                         // exactly I.K initialised slots
    invariant buf.cap == I.K

    pub fn filled(x: T) -> Array<I, T> where T: Copy = ...
    pub fn table(inline f: I -> T) -> Array<I, T> = ...      // [I] i => expr

impl Indexed<I> for Array<I, T>:
    type Elem = T
    fn at(self, i: I) lends mut T:
        law disjoint(i, j) = i != j        // §3.6
        lend mut buf[i.to_idx]
```

Indexing a `[V]T` with a `V` carries no check, and the reason is now a
derivation, not a rule about domains: `i.to_idx` has type `0..I.K`, the
struct invariant says `buf.cap == I.K`, so the value-range checker
(30-compiler) proves `buf[i.to_idx]` in range. `buf` is private and only
assigned in constructors, so the invariant is proven, not assumed
(contracts, 7.9).

A fn that should accept any indexable thing says so with a bound, which
answers open question 4.8 (views as read-only array arguments):

```
fn longest_true<A>(a: A) -> Int where A: Indexed<V, Elem = Bool>
longest_true(alive)                        // an array
longest_true(deg.map(d => d > 0))          // a view; nothing is allocated
```

A fn written with a concrete `a: [V]Bool` keeps accepting only arrays, so
signatures stay concrete unless the author asks for more.

### 3.4 Projections: `lends`

A projection is a fn that gives the caller access to part of its
receiver for a while, without copying and without returning a reference.
It is declared with `lends` where an ordinary fn has `->`, and its body
says `lend place`:

```
impl Array<(A, B), T>:
    pub fn row(self, r: A) lends mut [B]T:
        lend mut Array<B, T>(buf.part(r.to_idx * B.K, B.K))
```

**Meaning.** A projection is exactly a fn with one extra `inline` callback
parameter, called once at the `lend`:

```
// what the compiler checks for `row`
fn row(self, r: A, inline use: ([B]T) -> R) -> R =
    use(Array<B, T>(buf.part(r.to_idx * B.K, B.K)))
```

and a use of the projection passes the rest of the statement, or the rest
of the block for a `let`, as that callback:

```
print(max(g.row(r)))          // g.row(r, row => print(max(row)))

let row = g.row(r)            // g.row(r, row =>
total += sum(row)             //     total += sum(row)
print(row[c])                 //     print(row[c]))
```

So calling a projection needs no new checking rule. 5.4 already says a lambda
passed to an `inline` parameter "is checked as part of the caller's body";
the continuation of a lend is such a lambda. `return`, `break` and `?`
inside it act on the caller, the same as in the body of a `for` over a
generator (12 §5.2). In generator terms a projection is a generator that
yields exactly once, and the "lent until the next `yield`" rule is the
same rule.

**Rules.**

- Every path through a projection's body reaches exactly one `lend`. Code
  after the `lend` runs when the caller is done, which is where a
  projection puts write-back (a `Journaled` array records the old value
  there).
- `lends T` is a read-only lend. `lends mut T` on a projection with plain
  `self` lends as mutably as the caller holds the receiver: read-only
  through a read-only receiver, mutable when the caller holds it mutably
  and the use needs it (an assignment, a `mut` argument, a mutating
  method). One body serves both, because the body may only step through
  fields, indices and other projections of `self`. A projection that must
  change bookkeeping before it lends (a map inserting a default entry)
  declares `mut self`.
- Index arguments are evaluated before any lend begins, left to right.
  That is the existing two-phase rule (5.3), and it is why
  `a[i].push(a[j].len)` and `a[a.len - 1] = x` are fine.
- Left to right holds for every operand, not only indices: in
  `f(next(), next())`, `next() - next()` and `xs.push(next())` the calls
  run in the order they are written. An assignment runs its right side
  first and then the indices of its target, so in `a[next()] = next()`
  the first call made is the value. A program never has to know which
  order C++ would have picked.

**The one intrinsic.** `Buffer<T>` (M6) gains a safe projection,
`buf.part(off, len) lends Buffer<T>`, a sub-block checked once against
`buf.cap` when it is made. It is the only place pointer arithmetic
happens, and nothing in this document needs `@trusted`.

### 3.5 Lent values are second-class

`Array<B, T>(buf.part(...))` in `row` builds a struct around a buffer it
doesn't own. That looks like a struct holding a borrow, which 5.3 forbids.
The rule that makes it safe, and keeps lifetimes out of the language, is:

**A value built from lent parts is lent. A lent value lives only in the
block that made it.**

Concretely, a lent value may be passed as an argument, bound by a `let` in
the same block, iterated, and lent onward by `lend`. It may not be
assigned to a `var`, stored in a field of an owned struct or an element of
a container, captured by a lambda that isn't `inline`, or returned.

Borrowed-ness is a mode of the binding, not a type. This is already true
for parameters: `fn f(a: [V]Int)` and `let a: [V]Int` have the same type,
one borrows and one owns (5.1). The rule extends it one step: `row` lends
an `Array<B, T>` in lent mode, and the callee's `a: [B]T` parameter cannot
tell it from a whole array. No `view` keyword and no second family of
types is needed, and no user-written struct declaration changes: 5.3's "no
struct holds a borrow" stays true for every value that can be stored.

The errors say what to do instead:

```
error: `best_row` would keep a lent row of `g` after this block
  --> d.cp:14:9
14 |         best_row = g.row(r)
   |                    ^^^^^^^^ lent by Array.row; it lives only in this block
help: keep the row's index and re-project later:  best_r = some(r)
      or keep a copy:                             best_row = g.row(r).copy()
```

```
error: a fn can't return a lent slice of `a`
help: return the range and let the caller write a[s]:  -> Range<V>
```

That second help line is the general pattern of the whole design: **the
window is a value, the slice is a lend.** Functions compute and return
ranges, positions and row indices, all plain values with no borrow in
them. The caller applies them to the array where it uses them.

### 3.6 Disjoint lends

5.3 says indices are not borrowed separately, and makes `swap` and
`pair_mut` the ways out. With projections in the library, the compiler
needs a general answer to "may these two lends from the same receiver be
live at once", and the library needs to be able to write `pair_mut`.

A projection may state when two of its lends don't overlap, as a law (M4)
over the two argument lists:

```
fn at(self, i: I) lends mut T:                   // Array, one element
    law disjoint(i, j) = i != j
    ...

fn at(self, s: Range<I>) lends mut Slice<I, T>:  // Array, a range of elements
    law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo
    ...
```

Two live lends of the same projection on the same receiver, at least one
of them `mut`, are accepted when the checker can prove the predicate from
facts it already has at that point. Otherwise it is the exclusivity error,
and the message prints the predicate it needed.

```
if i != j:
    merge_into(mut cnt[i], cnt[j])           // ok: i != j holds here

solve(mut a[l..m], mut a[m..r])              // ok: the first range ends where the second starts

merge_into(mut cnt[i], cnt[j])
// error: cnt[i] and cnt[j] may be the same element
//   Array.at lends disjoint elements only when i != j
// help: guard it:  if i != j: ...   or state it:  assert i != j
```

With that, `pair_mut` and `swap` stop being special:

```
impl Array<I, T>:
    pub fn pair_mut(mut self, i: I, j: I) lends (mut T, mut T):
        assert i != j, "pair_mut({i}, {j}): the two positions must differ"
        lend (mut self[i], mut self[j])

    pub fn swap(mut self, i: I, j: I):
        if i != j: swap(mut self[i], mut self[j])
```

Rules:

- The facts are the flow facts the checker has for other reasons: a
  dominating `if` or `while` condition, `assert`, `requires`, `invariant`,
  a `match` arm, distinct literals, and value ranges (so `i < j` proves
  `i != j`). The index values are immutable copies taken before the lends
  begin, so a fact can't be invalidated while a lend is live; assigning to
  a `var` used in the fact discards the fact for later statements.
- The law is inlined at the use site and must reduce to comparisons the
  range checker can decide. One it can't decide is the error above, never
  a guess.
- `disjoint` relates two lends of the **same** projection. Lends from
  different projections of one receiver (`g.row(r)` and `g.col(c)`) always
  conflict. Different fields are always disjoint, as today.
- A `disjoint` law is a claim about the projection's own body. Like every
  law (M4) it is checked by tooling: debug runs check it at each site where it was
  used, by comparing the address ranges of the two lent places, and
  `cpl fntest` checks it with generated arguments.
- No check is ever inserted silently. When the fact is missing the program
  doesn't compile, and the user writes the `if` or the `assert` that
  states it.

---------------------------------------------------------------------------

## 4. The four features, rewritten in the library

### 4.1 Domains, bounds and ranges

§3.1 is the domain itself. Bounds and ranges are two more library structs,
generic over any `Seq`:

```
pub struct Cut<I>(ord: 0..=I.K) where I: Seq:         // sealed; K + 1 values
    pub @get fn left(self) -> ?I = I.try_at(self.ord - 1)
    pub @get fn right(self) -> ?I = I.try_at(self.ord)

pub struct Range<I>(lo: Cut<I>, hi: Cut<I>) where I: Seq:     // 8 bytes
    pub @get fn len(self) -> Int = max(0, hi.ord - lo.ord)
    pub @get fn first(self) -> ?I = if len > 0: lo.right else: none
    pub fn contains(self, i: I) -> Bool = lo.ord <= i.to_idx and i.to_idx < hi.ord
```

Everything in 12 §1 and §2 reads the same as before:

```
var pre: [V.cuts]Int = 0
for v in V: pre[v.r] = pre[v.l] + a[v]
fn sum(l: V, r: V) -> Int = pre[r.r] - pre[l.l]

pre[v]            // error: pre is indexed by Cut<V>, v is a V
a[v.r]        // error: a is indexed by V, v.r is a Cut<V>
```

Both errors are now plain "wrong struct" errors. `l..=r` on two positions
builds a `Range`, `Range1<I>` is a sealed wrapper that proves `len > 0`,
and `for v in s` iterates a range because `Range` implements the generator
protocol. Enums and tuples keep deriving `Index` (M7, M10), so
`[Dir4]Int` and `[(V, Mask)]Int` work with no extra rule.

### 4.2 Slices

A slice is a library struct holding an array and a window, and it is only
ever made by the projection that lends it:

```
pub struct Slice<I, T>(base: [I]T, win: Range<I>) where I: Seq

impl Indexed<Range<I>> for Array<I, T> where I: Seq:      // a[s]
    type Elem = Slice<I, T>
    fn at(self, s: Range<I>) lends mut Slice<I, T>:
        law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo
        lend mut Slice(self, s)

impl Indexed<I> for Slice<I, T>:                          // part[v]
    type Elem = T
    fn at(self, v: I) lends mut T:
        assert win.contains(v), "{v} is outside the slice {win}"
        lend mut base[v]

impl Slice<I, T>:
    pub @get fn positions(self) -> Range<I> = win
    pub fn sort(mut self) where T: Ord = ...              // runs over the contiguous block
    pub fn copy(self) -> List<T> = ...
```

Use is what 12 §3 promises:

```
a[s].sort()                          // in place, a is held mutably for this statement
let part = a[s]                      // a is held for reading until part's last use
let best = argmax(part)              // a V, usable in every other [V] array
deg[best]                            // fine
for v in part.positions: use(part[v])
```

**One change: a slice is indexed by `I` itself, and `Sub<V>` is removed.**
That closes the gap in §2, and it removes the last implicit conversion in
the language (2.8 named `Sub<V>` to `V` as the only one). The cost is that
`part[v]` with an arbitrary `v: V` carries the window check, in both
builds, unless it is proven (the 7.1 policy for `List` and `raw`). It is
proven for free in the forms slices are used in:

- `for v in part.positions`, `for (v, x) in part.items`, and every
  reduction and whole-slice method never index by an unproven position;
- `if v in part.positions:` and `assert` give the fact directly;
- positions that `argmax(part)`, `part.find(...)` and friends return carry
  the fact as a refinement on the result (the `where` refinements of 18 §1).

The check is a logic check, not a memory check: `v` is a valid position of
`base` whatever the window is, so even with `unchecked:` a slice can never
read outside its array.

A slice of a slice, `a[s][t]`, checks once that `t` lies inside `s` when
it is made. `a.raw` (2.11) is one more library projection, lending the same
storage indexed by `0..K` integers, so `a.raw[i]` is the ordinary checked
integer index.

### 4.3 Grids

A grid is an array indexed by a tuple. Nothing below is known to the
compiler:

```
domain Row = 1..=h
domain Col = 1..=w
type Cell = (Row, Col)                     // Index is derived for tuples, row-major (M7)
let g: [Row, Col]Char = read()             // Array<(Row, Col), Char>, one flat block

g[r, c]                                    // sugar for g[(r, c)]
let p: Cell = (r, c)
g[p]                                       // the same element
```

Rows, columns and steps are library code:

```
impl Array<(A, B), T>:
    pub fn row(self, r: A) lends mut [B]T:                // contiguous: a real [B]T
        law disjoint(r, q) = r != q
        lend mut Array<B, T>(buf.part(r.to_idx * B.K, B.K))

    pub fn col(self, c: B) lends mut Column<A, B, T>:     // strided: its own view struct
        law disjoint(c, d) = c != d
        lend mut Column(self, c)

pub struct Column<A, B, T>(g: [A, B]T, c: B)
impl Indexed<A> for Column<A, B, T>:
    type Elem = T
    fn at(self, r: A) lends mut T = lend mut g[r, c]

impl (A, B) where A: Seq, B: Seq:
    pub fn step(self, d: Dir4) -> ?(A, B):
        let (r, c) = self
        let (dr, dc) = d.delta
        some((r.offset(dr)?, c.offset(dc)?))
```

```
for d in Dir4:
    if let q = p.step(d) and g[q] != '#': ...            // off the grid is none
sort(mut g.row(r))                                       // a row is a [Col]Char to every fn
if r1 != r2: swap(mut g.row(r1), mut g.row(r2))          // two rows at once
let tall = count(c for c in Col if all(x == '#' for x in g.col(c)))
```

Because `row` lends a genuine `[Col]T`, any fn written for one-dimensional
arrays (`runs`, `prefix`, a user's `peel<D>(start: [D]Bool)` from the
`dmopc21c9p4` port) takes a row with no adapter. A column is a different
struct because its elements are `Col.K` apart in memory, so it can't pass
as a contiguous `[Row]T`; generic code over `A: Indexed<Row>` takes either.

Three things fall out that the grid feature never had: three or more
dimensions (`[A, B, C]T` is `Array<(A, B, C), T>`, and `dp[i, j, k]` is the
same sugar), grids of any `Index` pair such as `[V, Mask]Int` or
`[L, L]Bool` (already used in `nccc10s2.cp`), and the same row and column
projections on all of them.

### 4.4 Strings

**ASCII only.** A `Char` is one byte holding an ASCII character. A `Str`
is a flat array of `Char`, so `s[i]` is one load, `s.len` is the number of
characters and the number of bytes, and slicing can never cut a character
in half. There is no UTF-8 in the language: no code points, no
multi-byte `Char`, no normalisation, no "byte index or character index"
question. A non-ASCII character in a `Char` or string literal is a compile
error that says so. Contest input is ASCII; a judge that sends other bytes
is read with `[S]U8`, which is honest about what it holds.

Decision 1.6 says a string is an array of `Char` that brings its own
domain. The binder of §3.2 makes that a general type, not a string rule:

```
[domain]T        // an array packaged with a fresh index type of its own
type Str = [domain]Char
```

A `[domain]T` is stored as a pointer and a length. Binding one to a name
opens it: the name's static type becomes `[S]T` for a fresh index type
`S`, reachable as `s.Idx`, or named in the annotation when two things must
share it.

```
let s: Str = read()                        // s: [s.Idx]Char
let t: Str = read()                        // a different index type

for i in s.Idx: ...
let z = z_function(s)                      // fn z_function<D>(s: [D]Char) -> [D]Int where D: Seq
z[t.Idx.first!]                            // error: z is indexed by s.Idx

for (run, ch) in runs(s): ...              // every sequence view works: s is an array
let mid = s[l..=r]                         // a Slice, as for any array

var lcs: [s.Idx.cuts, t.Idx.cuts]Int = 0
for i in s.Idx:
    for j in t.Idx:
        lcs[i.r, j.r] = if s[i] == t[j]: lcs[i.l, j.l] + 1
                                else: max(lcs[i.l, j.r], lcs[i.r, j.l])
```

The LCS table shows what the domain buys: the `+ 1` shifts and the "is row
0 the empty prefix" question of the C++ version don't exist, and swapping
`i` and `j` is a type error.

When the length is given in the input, there is nothing string-specific at
all, and the read checks the length:

```
let n: 1..=2e5 = read()
domain S = 1..=n
let s: [S]Char = read()
```

Stored strings are packages. A package is an ordinary owned value, so
`List<Str>`, `Set<Str>`, `Map<Str, Int>` and struct fields work, and each
element may have a different length:

```
let words: List<Str> = read(count: n)
var seen = Set<Str>()
for w in words:                            // w is opened here: [w.Idx]Char, lent for one iteration
    if w[w.Idx.first!] == 'a': seen.insert(w.copy())
```

`Eq`, `Ord` and `Hash` are implemented for `[domain]T` by content, and
slices compare by content, so `s[a..=b] == t[c..=d]` needs no shared index
type. Building a string is `List<Char>` and `Str(chars)`, the same
grow-then-freeze shape as `xs.freeze()` in 12 §1.6. Both are flat byte
arrays; neither carries an encoding. `[D]Char` already reads
and prints as text (grids rely on it), so `Str` is a plain alias with no
`Show` or `Read` of its own. Literals and interpolation are unchanged.

The same type serves jagged data in general: CSR adjacency rows and
`List<[domain]Int>` for "n lists, each with its own length" are
`[domain]T` with no new feature.

### 4.5 Permutations

`Perm<I>` is the simplest of the four. It is a sealed struct over `[I]I`
that only offers operations that keep it a permutation:

```
pub struct Perm<I>(to: [I]I) where I: Index:               // sealed
    pub fn id() -> Perm<I> = Perm([I] i => i)
    pub fn from(a: [I]I) -> ?Perm<I>                       // O(K) check; none if a value repeats
    pub fn inv(self) -> Perm<I>:
        var r: [I]I = [I] i => i
        for i in I: r[to[i]] = i
        Perm(r)
    pub fn then(self, q: Perm<I>) -> Perm<I> = Perm([I] i => q[self[i]])
    pub fn swap(mut self, i: I, j: I): to.swap(i, j)       // still a permutation
    pub fn cycles(self) yields List<I>

impl Indexed<I> for Perm<I>:
    type Elem = I
    fn at(self, i: I) lends I = lend to[i]                 // read-only: `lends`, not `lends mut`

impl Read for Perm<I>         // reads K labels; local runs reject a repeat and name the line
impl Gen for Perm<I>          // a random shuffle; small and edge shapes first (identity, reversal, one cycle)
impl Shrink for Perm<I>       // undo one transposition at a time, toward the identity
```

```
let p: Perm<V> = read()
let q = p.inv()
p[v] = u
// error: Perm<V> can't be assigned through an index
//   one write would leave two positions with the same value
// help: p.swap(v, w) exchanges two entries;  Perm.from(arr) checks a whole array
```

That error is the absence of an `IndexedMut` impl, with help text attached
to the struct. The same recipe (a sealed struct, a read-only `Indexed`,
the `Read` + `Gen` + `Shrink` trio of 18 §1) gives `Sorted<[V]T>`,
`Distinct<[V]T>` and `Tree<V>`, so typed input needs no compiler support
per type.

The `nccc10s2.cp` port uses `Perm<L>` for "the letters of `L` listed in
some order", which is a bijection from list positions to `L`, not from `L`
to itself. Proposed: one struct `Bij<I, J>` (a pair of inverse arrays,
`[I]J` and `[J]I`, same size) with `Perm<I>` as `Bij<I, I>`. Euler order
(`r.tin: [V]Pre`, `r.at: [Pre]V`, 13 G5) and `compress`'s rank map are the
same shape and would use it too.

---------------------------------------------------------------------------

## 5. Check against the borrow rules

The borrow rules are 5.3 and 5.4 (14, RECOMMENDATIONS §1): exclusive
mutation, no borrow stored in a struct or returned from a fn, fields
borrowed separately and indices not, two-phase borrows, `inline` callbacks
checked as part of the caller, generator-lent values living one iteration.
This section walks each rule against the mechanism and marks what changes.
I checked these by hand against the written rules; none of it has been run
through a checker.

### 5.1 No stored borrows

Positions, bounds, ranges, cells and permutations are plain owned values:
4 to 8 bytes of integers or an owned array. They hold no borrow, so they
can be stored anywhere, and that is what you store when you want to
remember a part of an array.

Slices, rows, columns and opened strings are lent values (§3.5). Every way
of keeping one is rejected with the same two-line help: keep the window
and re-project, or `.copy()`.

```
var best: ?Slice<V, Int> = none
for s in windows(V, k):
    if sum(a[s]) > top: best = some(a[s])        // error: lent slice assigned to a var
// fix: var best: ?Range1<V> = none ... best = some(s) ... then a[best!]

struct Job(part: Slice<V, Int>)                  // allowed as a declaration
let j = Job(a[s])                                // error: Job would own a lent slice
```

Proposed limit for the first version: lent-mode construction of a struct
is allowed **only inside a projection body**. So user code can write new
view types and new projections, but a block can't assemble an ad-hoc
struct of borrows. That keeps 5.3 word for word ("no struct holds a
borrow" in any value a variable can own) and leaves the wider question
for §10.

### 5.2 No returned borrows

A projection doesn't return. Its desugaring (§3.4) is a call that passes
the continuation down, so the lent place only ever flows into callees and
is gone when the call ends. `fn part(a: [V]Int) -> Slice<V, Int>` is an
error at the signature. The pattern that replaces it is to return the
window:

```
fn best_window(a: [V]Int, k: 1..) -> ?Range1<V> = argmax(sum(a[s]) for s in windows(V, k))

if let s = best_window(a, k): a[s].fill(0)
```

Generators may yield lent values, under the existing one-iteration rule:
`for row in g.rows: ...` lends each row for one loop body, and
`for part in a.chunks_mut(k): part.sort()` lends disjoint mutable
chunks one at a time while `a` is held mutably for the loop.

### 5.3 `inline` callbacks

Since a use of a projection is an `inline` callback, every property of 5.4
carries over, and each is worth stating as a consequence:

- **The continuation is the caller's code.** It may read and write the
  caller's other variables, call `mut fn`s, and use `return`, `break`,
  `continue` and `?`, which act on the caller.
  ```
  let row = g.row(r)
  if row[c] == '#': return none          // leaves the enclosing fn; g is released on the way out
  hits += count(x == '*' for x in row)   // `hits` is the caller's var
  ```
- **The receiver is held for the lend's extent.** A `let` alias holds it
  for reading until the alias's last use (the same "ends at its last call"
  rule as `mut fn`, 14). A `mut` lend holds it exclusively for the
  statement or the call it appears in.
  ```
  let part = a[s]
  a[v] = 0               // error: a is lent to `part` until line 5
  print(part.len)
  ```
- **A `mut` lend is used in place or passed down; it isn't aliased.**
  `sort(mut a[s])`, `a[s].fill(0)`, `a[v] += 1` and `solve(mut a[l..m])`
  cover the uses in 12. A mutable `let` alias is left out of the first
  version because it would hold `a` exclusively for a region the reader
  can't see at the call site.
- **Nesting follows evaluation order.** `g.row(r)[c] = 'x'` is the `row`
  callback containing the `at` callback. Index expressions are evaluated
  first (two-phase), lends begin left to right and end in reverse.
- **No recursion through a projection.** `inline` parameters can't be
  passed to a recursive call of the same fn, so a projection that walks a
  tree to find its target uses a loop. None of the projections in this
  document recurse.
- **`mut fn` and captured state.** A `mut fn` that writes `dist` can't be
  called while `dist[v]` is lent mutably. This is the existing exclusive
  mutation rule, and the error names both the lend and the capture.

### 5.4 Indices and the flow-sensitive `i != j`

This is the one borrow rule the proposal changes. 5.3 says indices are
never borrowed separately. The proposal says: two lends through the same
projection are separate when the projection's `disjoint` law is proven at
that point (§3.6). What is accepted and rejected:

```
a[i] = a[j]                              // ok, no fact needed: a[j] is copied out before a[i] is lent
a[i] += a[i]                             // ok, same reason
f(mut a[i], a[j])                        // error unless i != j is known (when Elem is not Copy)
a.swap(i, j)                             // ok always: the library handles i == j

for i in V:
    for j in V:
        if i == j: continue
        absorb(mut bag[i], bag[j])       // ok: i != j holds after the continue

for (u, v) in pairs(V):                  // pairs yields (u: D, v: D) where u < v
    absorb(mut bag[v], bag[u])           // ok: the refinement gives u != v

var j = i.next!
absorb(mut bag[i], bag[j])               // ok: Seq.next ensures its result is > self
j = pick()
absorb(mut bag[i], bag[j])               // error: the fact about j ended at the assignment

absorb(mut bag[p[i]], bag[p[j]])         // error even when i != j: nothing says p[i] != p[j]
solve(mut a[l..m], mut a[m..r])          // ok: half-open ranges that share an endpoint
absorb(mut g.row(r), g.col(c))           // error: different projections never count as disjoint
absorb(mut s.left[i], s.right[i])        // ok: different fields, as today
```

Why this is safe to add:

- The fact is about immutable index values copied before the lends begin,
  so nothing in the continuation can make it false.
- When the fact is missing, the result is a compile error that prints the
  law, never an inserted check and never undefined behaviour.
- The law is a library author's claim. It is checked in debug runs at the
  sites that relied on it (address overlap of the two lent places) and by
  `cpl fntest`, and the emitted C++ never marks lent places `restrict`, so
  a wrong law in a judge build is a logic bug, not undefined behaviour
  (G12).

Why it is worth adding: without it `pair_mut` stays a compiler intrinsic,
two rows can't be swapped without copying, and divide and conquer on two
halves needs a special `split_at`. With it, all three are four-line
library fns, and the rule a user learns is one sentence: "two elements of
one array can be used at once if the compiler can see they are different
elements".

### 5.5 Generator-lent values

Unchanged, and the "window is a value" split makes the common case easy:

```
for x in a[s]: a[v] = 0                  // error: a is lent to the loop
for v in s: a[v] = 0                     // ok: s is a Range, a plain value; nothing is lent
for (v, x) in a[s].items:
    if x < 0: bad.push(v)                // ok: `bad` is another variable; v is a V and outlives the loop
```

### 5.6 Smallest programs for the red-team suite

Each of these should be a locked test with its message (5.11), and the
first ones a fresh agent tries to break:

1. `let part = a[s]; a[s.first!] = 0; print(part[s.first!])`: rejected,
   message names `part` and the line of its last use.
2. `swap(mut a[i], mut a[i])`: rejected with the `i != j` law.
3. `if i != j: swap(mut a[i], mut a[j])` then the same call after
   `i = j`: first accepted, second rejected.
4. `return a[s]` from a fn: rejected at the signature with "return the
   range".
5. `let (X, vals, _) = compress(xs)` inside a loop, with `var keep: ?X`
   declared outside the loop: rejected, "X belongs to the loop body".
6. `let r = g.row(r1); g[r2, c] = '.'` with `r1 != r2` known: rejected in
   the first version (different projections), with help to write through
   `g.row(r2)`.
7. `p[v] = u` on a `Perm`: rejected with the `swap` help.
8. `words.push(s[l..=r])`: rejected with `.copy()` help; `words.push(s)`
   where `s` is used later: the existing explicit-copy error.
9. `for w in words: longest = w`: rejected (lent for one iteration);
   `longest = w.copy()` accepted.
10. A `disjoint` law that is wrong (`row` declared disjoint from every
    other row but lending overlapping blocks): caught by the debug-run
    overlap check with both call sites.
11. `for x in xs: xs.push(x)`: rejected, the loop lends `xs` for reading
    until it ends; the help offers `for i in 0..xs.len` and
    `for x in xs.copy()`. The same through a fn: `fn grow(): xs.push(7)`
    called in the loop body is rejected and names `grow`.
12. `let part = a[l..=r]; clobber(); print(sum(part))` where `clobber`
    assigns the captured `a`: rejected, "`clobber` can't be called here:
    it changes `a`, and `a` is lent to `part`". One level of indirection
    (`outer` calls `clobber`) is rejected the same way.
13. `fn go(mut x: Int, i: V)` whose body also writes the captured `a`,
    called as `go(mut a[l], l)`: rejected, the callee would reach the
    same storage under two names. This is the recursive local `mut fn`
    case; passing the position (`go(l)`) is the fix.
14. `if i != r: step(); mv(mut a[i], mut a[r])` where `step` assigns the
    captured `i`: rejected, the call drops the fact `i != r`.
15. `a[l] += inc(mut a[l])` and `xs[0] = xs.pop()!`: accepted (decision
    144): the right side runs before the target is lent, so the lends never
    overlap; both are linted.
16. `a[next()] max= 4`: rejected, the target would be evaluated twice.
17. `f(next(), next())` with a `next` that counts: prints in written
    order (3.4), also for methods, operators and `min`/`max`.
18. `let k: Int = max(a[f..f])`: rejected with "expected Int, got ?Int".
    A slice keeps its parent's index type but can be empty, so its
    `min`/`max` are optional even when the array's are not.
19. `domain V = 0..=3000000000`: rejected, a domain holds at most
    2^31 - 1 values. With bounds read at run time the program stops with
    "domain has more than 2^31 - 1 values"; an inverted range (`5..=1`)
    is the empty domain, and the length never wraps, even for
    `Int.min..=Int.max`.
20. `/` and `%` at the ends of `Int`: `a % b` is in `0..|b|` for every
    nonzero `b`, including `b = Int.MIN`; `a % -1` is 0 and
    `Int.MIN / -1` is `Int.MAX`, because `Int.MIN` is `-(2^63 - 1)`
    (decisions.md 1.1); `a / 0` and `a % 0` stop with "division by zero"
    when the checker could not rule the zero out. The pattern
    `-9223372036854775808` is below `Int.MIN` and is not an `Int`; the
    division helpers are still correct for it as a divisor, and stop
    with "Int overflow" when it is divided by `-1`, so a value that got
    there from input or unchecked arithmetic never reaches the undefined
    native division.
21. `a + a`, `a * a`, `-a`, `abs(a)`, `s += a` and `a << k` on values the
    range checker can't bound: a local build stops with "Int overflow"
    and the line; a judge build is the plain operator.
22. `a[z.l..z.r(k)]` with `k` too large, `a[z.l(k)..z.r]` with
    `k = Int.min`, and `V.first + k` past the end: each stops with the
    expression, the domain and its size, the position and `k`, as in
    "`z.r(k)` leaves V.cuts (2 values): from position 0, k = 2". The
    sum never wraps back into the domain.
23. `V.first` and `V.last` when `V` is empty stop with "`V.first`: V is
    empty". A domain with one value has `first == last`, and every window
    of 5.6 case 22 still works on it; `a[z..=f]` written backwards is the
    empty slice.

---------------------------------------------------------------------------

## 6. Checks and cost

All of these are targets. None is measured yet.

- **Indexing `[I]T` by `I`:** no check, by the derivation in §3.3. The
  emitted C++ is `buf[ord]`, the same as today.
- **Creating a position:** `V(x)`, `V.at(i)`, `read()`: checked, as today.
- **A slice indexed by an arbitrary position:** one window check unless
  proven; iteration, reductions and whole-slice methods have none. This
  is new cost in one narrow place, in exchange for removing `Sub<V>`.
- **Projections:** inlined, so `a[i] += 1` is one load and one store, and
  `g.row(r)` is a pointer add. No closure objects exist at run time.
- **Sizes.** A position takes the width its range needs, as every range
  type does (1.2): one byte for `domain L = 'A'..'A' + k`, four for
  `1..=2e5`. Today's plan says four always (2.1), so this is a small
  memory win and a change to note. `?V` stays niche-packed. A range is 8
  bytes, a lent `[I]T` is a pointer, a slice is a pointer plus a range, a
  `Str` package is a pointer plus a length.
- **Code size (G6).** `Array`, `Slice` and the projections are generic
  library code, so more instantiations exist before sharing. M1's "same
  layout shares one copy" rule should absorb it, since most index types
  are 4-byte ordinals. The size report must be checked on the existing
  ports before and after.
- **Compile time (G10).** More of each program is library code, so the
  checked standard library has to be cached, and the debug interpreter
  will want fast paths for `Array.at` and `Slice.at`. A fast path must
  not change behaviour; the differential fuzzing against the emitted C++
  (G12) is what holds that.
- **Errors (G5).** A diagnostic must say "`deg` is indexed by `V`, `e` is
  an `E`", never "expected struct `V(ord: 0..n)`". Names come from the
  `domain` sugar, and every help line quoted in this document is attached
  to a library item, not built into the checker. Each quoted message
  should become a locked test.
- **Trace (7.3).** `3:V`, `V[2..=5]` and grid rendering are `Debug` impls
  on the library structs.

---------------------------------------------------------------------------

## 7. What the compiler keeps, and what it drops

**Keeps (the whole kernel for this area).**

1. Range types and the value-range checker (11, 30).
2. Nominal structs with sealed constructors and proven `invariant`s.
3. Traits with associated members: a `const` (compile-time), a `let`
   frozen from a run-time value where the impl is declared, and
   associated types (3.8).
4. Fresh index binders: `domain D` in signatures and bindings, the
   `[domain]T` package, and the escape rule (§3.2).
5. `Indexed` and `IndexedMut` as the meaning of `a[i]`.
6. Projections (`lends`, `lend`) as sugar for `inline` callbacks, lent
   values as second-class, and `law disjoint` in the exclusivity checker.
7. `Buffer<T>` with `part` (M6).

Plus four pieces of sugar with no semantics of their own: `domain X =
range`, `[I]T` and `[A, B]T`, `a[i, j]`, and `x in V`.

**Drops.** The domain type kind and its indexing rule; `V.cuts`,
`Range`, `Range1` and `Sub<V>` as known types; slices and `a.map(f)` views
as built-in borrowed forms; product-domain indexing and grid row views;
`Str`; `Perm`; `pair_mut` and `swap` as intrinsics; the implicit
`Sub<V>` to `V` conversion; and `a.raw` as a built-in.

By count this is seven general features in place of roughly a dozen
special ones. Of the seven, two are new to the plan: number 5 extends
"operators are trait methods" (M2) to indexing, and number 6 is built
from 5.4. Numbers 3 and 4 are existing leans made precise.

---------------------------------------------------------------------------

## 8. Rows of `decisions.md` this changed

Recorded in `decisions.md` §9 on 2026-11-20. In prose here so the reasons
travel with them:

- **1.6 Strings.** Stays `[domain S]Char`, now defined: `Str` is the alias
  `[domain]Char`, a package opened at binding. No string-only rules.
  `Char` is one ASCII byte and there is no UTF-8.
- **2.1 Domains.** "Positions are 4-byte ordinals" becomes "a domain is a
  sealed struct over a range type; its width follows the range". The
  no-check guarantee is unchanged and now derived.
- **2.2, 2.3 Bounds and ranges.** Same behaviour, library structs over
  `Seq`. Boundaries are spelled `v.l` / `v.r`, with `v.l(k)` / `v.r(k)`.
- **2.4 Slices.** Positions of a slice are `I`, not `Sub<I>`. Slices are
  lent by a library projection.
- **2.5 Fresh domains.** Gains its scoping rule (§3.2).
- **2.8 Conversions.** No implicit conversion remains.
- **2.11 Escape hatch.** `a.raw` is a library projection; spelling and
  check policy unchanged.
- **3.5 Representation traits.** Adds `Seq` next to `Index`.
- **3.8 Associated types.** Settled yes, justified by `Indexed.Elem` and
  `Seq.cuts`. An associated size is `const K` when it is known at
  compile time and `let K = n` when it is frozen from a run-time value.
- **4.8 Views as array arguments.** Settled: a fn opts in with
  `A: Indexed<V, Elem = T>`; concrete `[V]T` parameters stay concrete.
- **5.3 Borrow rules.** "Indices don't borrow separately" becomes "two
  lends of one projection are separate when its `disjoint` law is proven".
  Adds "a value built from lent parts is lent, and lives only in its
  block".
- **5.4 Inline closures.** Unchanged; projections are a second client of
  it after `undo`.
- **5.5 Moves out of arrays.** `swap` and `take` become library fns over
  `at`.
- **New rows.** Projections (`lends`), `law disjoint`, second-class lent
  values and `[domain]T` packages are rows 5.12 to 5.14 and 2.14. `Bij<I, J>`
  is not adopted yet.

---------------------------------------------------------------------------

## 9. How to test it before the Rust checker depends on it

The order follows the cheapest experiment that could kill the idea first.
Step 5 and the checker half of step 6 are started in the prototype (§11).
Steps 1 to 4 are not done: the prototype has no traits or generic structs,
so `Array` can't be written as a `.cp` library file there yet, and nothing
below has been measured.

1. **Hand-desugar in the prototype.** Write `Array`, `Slice`, `row` and
   `Perm` as `.cp` library files using explicit `inline` callbacks, and
   point three existing ports at them: `nccc10s2.cp` (`Perm`, `[L, L]`),
   `dmopc21c9p4.cp` (grid, generic `peel<D>`), and one string problem from
   the corpus. If the emitted C++ differs from today's in the hot loops,
   stop and find out why.
2. **Benchmarks.** Re-run the four kernels of RECOMMENDATIONS §4 (prefix,
   segment tree, knapsack, matmul) with library `Array` against the
   built-in one. The bar is "within noise", since both should emit the
   same loads and stores.
3. **Size and `sizeof`.** Compare the bundled output size of every
   existing port, and `sizeof` of positions, ranges, slices and `Str`.
4. **Corpus pass.** In the 30-problem re-solve, tag every place a port
   wanted to keep a slice, alias a `mut` lend, or use two elements at
   once. Those counts decide the first-version limits in §5.1 and §5.3
   and whether cross-projection disjointness is needed.
5. **Red-team.** Seed the adversarial suite with §5.6 and have a fresh
   agent look for the smallest program where the lend rules surprise.
6. **Then** add the `lends` sugar and the `disjoint` law to the checker.

---------------------------------------------------------------------------

## 10. Open questions

1. **Slices indexed by the parent position, or branded.** The draft picks
   parent-indexed with a window check. The alternative keeps check-free
   indexing by giving each slice a fresh index type
   (`let (domain S, part) = a.rebase(s)`), at the price of a conversion
   back to `V`. The corpus pass should say how often a slice is indexed
   by an arbitrary position at all.
2. **Lent structs outside projection bodies.** Allowing a block to build
   a struct of lent parts would cover the "helper struct over outside
   arrays" case that the research put at about 2% of files, with no
   lifetimes. It also widens what a reader must track. Left out of the
   first version on purpose.
3. **Mutable `let` aliases** of a lend (`let mut row = g.row(r)`).
   The prototype infers it instead: a `let`-bound lend is mutable when the
   receiver is a `var` and the code below writes through it, and then the
   receiver can't even be read until the lend's last use. Whether that
   should need a keyword is open.
4. **Disjointness across projections** (`g.row(r)` against `g[q, c]` with
   `q != r`, or against `g.col(c)`). Needs a law naming two projections.
   Wait for the corpus to ask.
5. **Facts through maps.** `bag[p[i]]` and `bag[p[j]]` are different when
   `p` is a `Perm` and `i != j`. A `Perm` could export that as a law; it
   is the first relational fact that isn't about the index values
   themselves.
6. **Position width.** Follow the range (this draft), or pin positions to
   4 bytes as 2.1 says today.
7. **Spelling.** `lends` / `lend`, `law disjoint`, `v.l` / `v.r` and
   `let K = n` are settled. Still open: `[domain]T`, `s.Idx`, `Seq`, and
   whether `Column` deserves a general `Strided` struct.
8. **How `domain` is defined.** Fixed sugar with a documented expansion
   (this draft), or the first user of the `decorator` mechanism (3.11) so
   a user library could define its own declaration forms.
9. **Dependent domains (2.6)** stay open and untouched: "children of `v`"
   is not expressible by anything here.

---------------------------------------------------------------------------

## 11. What the prototype does today

`proto/cpc/` (the Python compiler, see `agent-context.md`) implements the
rules a user meets, so they can be tried against the corpus before the
Rust checker is written. The checker lives in `proto/cpc/lends.py`; the
tests are `crates/cpl-cli/tests/cases/lend/` (programs that run and print) and
`crates/cpl-cli/tests/cases/bad/` (programs that must be rejected, each with the error
text it must produce on its first line). `proto/run_all.sh` runs both
after the 30 ports.

**Bounded indices.** `v.l`, `v.r`, `v.l(k)`, `v.r(k)`, and the `Index`
names on a domain: `V.K`, `v.to_idx`, `V.from_idx(i)`. A struct becomes an
index type with an `impl`:

```
struct Cell:
    r: Row
    c: Col

impl Index for Cell:
    let K = h * w                                  // frozen from the input here
    fn to_idx(self) -> Int = r.ord * w + c.ord
    fn from_idx(i: Int) -> Cell = Cell(Row.nth(i / w), Col.nth(i % w))

var dist: [Cell]Int = 0
for x in Cell: dist[x] = ...
```

`const K = h * w` and `let K = m` with `m` a `var` are the two errors of
§3.1. A tuple of domains indexes directly: `g[p]` with `p: (Row, Col)` is
`g[r, c]`.

**Projections.** A struct method may be declared `lends T` or
`lends mut T`, with an optional `law disjoint` and a body ending in
`lend place`:

```
struct Bag:
    cnt: [V]Int

    fn at(self, i: V) lends mut Int:
        law disjoint(i, j) = i != j
        lend mut cnt[i]

b.at(v) += 5                              // a place: writes go through the lend
if i != j: move(mut b.at(i), mut b.at(j)) // two lends at once, allowed by the law
```

**Lent views.** `a[l..=r]` and `a[x..y]` (between boundaries) lend a
slice indexed by the same positions; `g.row(r)` lends a row as a `[Col]T`.
They can be passed to any top-level fn that takes the array type (the fn
is specialised for the view), bound with `let`, iterated, reduced, sorted
and reversed in place, and copied with `.copy()`. They can't be put in a
`var`, assigned, pushed, stored in a struct, list or tuple, or returned.

**Lexical exclusivity.** Everything passed by reference to one call is
lent together, so two places from the same root with at least one `mut`
must be provably disjoint: different fields always are; two lends of one
projection are when its law follows from the facts in scope (`if`,
`while`, `assert`, an early `if i == j: return`, distinct literals, value
ranges, and `l..m` next to `m..r`); two different projections never are.
A fact is dropped when a variable in it is assigned, and at the top of a
loop that assigns it. A `let`-bound view keeps its receiver lent until
the last statement that mentions it.

**What the prototype does not do.** These are limits of the prototype,
not of the design:

- `[I]T`, slices, rows and their laws are built into the compiler. They
  behave as the library code of §3.3 and §4, but `Array`, `Slice` and
  `Indexed` are not written in `.cp`, because the prototype has no traits
  or generic structs. Step 1 of §9 is therefore still to do.
- A projection is emitted as a C++ function returning the lent place by
  reference, which is what inlining the callback of §3.4 produces. Code
  after the `lend` (write-back) is emitted as a second function that runs
  when the statement using the place ends, so it sees `self` and the
  parameters but not locals declared before the `lend`, and such a
  projection can be used in an assignment, a `let` or a call statement
  only (`tests/lend/writeback.cp`). The `lend` must be a statement of the
  body itself, and free fns can't be projections.
- A user `Index` type pays one bounds check per index unless `to_idx`'s
  result range proves it; the real checker proves it from `0..K`.
- Indexing a slice by a position always checks the window; the "proven
  for free" forms of §4.2 are not recognised.
- No `[domain]T` packages or `s.Idx`, no fresh-domain escape rule (§3.2),
  no `Column`, no `pair_mut` (a `lends` of a tuple), no views of `Bool`
  arrays, and `swap` of two views works for rows only.
- A wrong `disjoint` law is not caught: the debug-run overlap check of
  §3.6 is not implemented.
- The "last use" of a `let`-bound view is found by name, so a second
  variable with the same name in a nested block extends the lend.
- Calls are followed by name only. A fn's effects are what its body (and
  the bodies of the fns it calls) assign, mutate through a method or pass
  as `mut`; a fn value passed as an argument or stored in a variable is
  not followed, and a method call is followed when the receiver's type is
  known at the call.

**Captured variables and calls.** A fn that changes a `var` it captures
is a use of that `var` wherever it is called (5.10), so the prototype
works out each fn's effects once and checks every call against them:

```
var a: [V]Int = read()
fn clobber():
    a[V(1)] = 99

let part = a[l..=r]
clobber()           // error: `clobber` can't be called here: it changes `a`,
print(sum(part))    //        and `a` is lent to `part` until this line
```

The same analysis drops a fact when a call may assign a variable in it,
rejects `go(mut a[l], l)` when `go` also writes `a` itself, and rejects
`total(xs, grow())` when `grow` changes the `xs` lent to the same call.
Recursion is handled by taking the effects of the whole call graph, so a
recursive local fn that writes a captured array is fine on its own and
an error only while something else holds that array.

**Loops lend what they walk.** `for x in xs` over a `var` list, array,
set, map or view lends it for reading until the loop ends. Assigning
elements of a list by position inside its own loop is still the way to
update in place (`for i in 0..xs.len: xs[i] += 1`); growing, shrinking or
reassigning the thing being walked is the error. `xs.copy()` gives the
loop its own list.

**Evaluation order.** The emitted C++ evaluates operands in the order of
3.4. When two operands have effects, or one changes what another reads,
every operand but the last is evaluated into a temporary first; code with
no such operands is emitted exactly as before.

**The ends of `Int`.** `/` and `%` are Euclidean for every pair of
operands, with the native C++ operators used only when the checker proves
`a >= 0` and `b > 0`. Division by zero stops the program in every
build; `Int.MIN / -1` is `Int.MAX`, since `Int` is symmetric, and only
the pattern `-2^63`, which is not an `Int`, stops when divided by `-1`. Arithmetic the checker could not bound stops a
local build on overflow (`-DLOCAL`, which `cp run` and the test suites
use) and is the plain operator on a judge. A domain's length is computed
in 128 bits and capped at 2^31 - 1. `crates/cpl-cli/tests/cases/redteam/` holds one
program per case of 5.6 that runs, each built under ASan and UBSan.
