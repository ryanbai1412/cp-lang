# Ownership, mutation and lending

This page is the whole model. After reading it you should be able to look
at any snippet and say whether it compiles, and which rule decides it. It
supersedes the shorter statements in `core/12-powers.md` W4/W8, `plan/14-state.md`
and `plan/19-index-and-lends.md` §3.4 to §5 where they differ; §13 lists
every difference and why.

There are no lifetimes, no reference types and no pointers. Every value is
**owned** or **lent**, the checker infers which, and twelve rules (R1 to
R12) say what each may do.

## 1. The model in six sentences

1. A variable owns its value; `let` can't change it, `var` can.
2. Small values with no heap part copy; heap values (`List`, `Map`, `Str`,
   `[D]T`, structs holding them) move, and copying one is `.clone()`.
3. A parameter is a lend of the caller's value: the callee can use it but
   never keep it, and `mut` (written at both ends) lets it change it.
4. A value built from a lend is itself lent: slices, views, generator
   values, loop items, projection results, closures over local state.
5. While a lent value is alive (until its last use) its sources are frozen
   for writes, or untouchable if it is a `mut` lend; and it can never be
   kept anywhere that outlives them.
6. Each call is checked as one access to everything it touches: its
   arguments and whatever the called fn or closure captures.

### Six models

1. **Values, not objects.** No two variables refer to the same list:
   `let b = a` moves or copies, never aliases. Mutation is always to a
   name you can see.
2. **A lend is "I'm looking at it, so it holds still".** Read: many
   lookers, no writers. `mut`: one looker, who may write.
3. **Looking lasts until your last glance,** and never past the block it
   started in.
4. **A call looks at everything it touches** for its duration: arguments
   and whatever its fn and closure arguments capture.
5. **Fns and closures are code, not views.** They look only while running.
6. **"Holds still" means the answers, not the bytes.** A `@cache` field
   (§10a) may change how answers are stored, never what they are.

The danger the rules prevent is a reader whose view a write breaks or
changes underneath it (iterator invalidation, a slice summed while it
changes, two names for one element). So the question is always: does
anyone hold a live view this write would break?

### Proved, promised, unsafe

| Tier | Spellings | If broken | Surfaced by |
|---|---|---|---|
| Proved | moves, lends, R1 to R12 | can't be | the compiler |
| Trap | `!`, indexing, `assert`, preconditions such as sortedness | stops with a message | debug builds check every one |
| Promise | `@cache`, `@law` | wrong answers, never undefined behaviour | `cpl explain` lists each; stress runs check them |
| Unsafe | `@trusted:` blocks, `@cpp("...")` bodies | anything | `cpl explain` lists each; debug shadow checks where possible |

Anything the compiler trusts without proof is an `@` attribute from this
closed list, so `grep -E '@(cache|law|trusted|cpp)'` finds all of it, and
adding to the list takes a decision entry. Codegen never optimizes on a
promise (no `__restrict` from a `disjoint` law), which is what keeps a
broken promise from becoming undefined behaviour.

### The picture

Every live lent value points at the variables it borrows. A variable with
a pointer at it is frozen: no writes under a read lend, no access at all
under a `mut` lend. The pointer goes away at the value's last use, and it
may never outlive the variable (R8). That is the whole checker.

- **Values built from arguments hold pointers:** slices, views, projection
  results, structs with a lent field, generator values. They lend from
  when they are made to their last use.
- **Fns and closures hold no lends.** A call acts as if everything they
  capture were passed as extra arguments at that moment. They only hold
  the captured variable's name: it can't move while they live, and they
  can't outlive it.

```python
let walk = dfs(g, s, mut seen)   // seen lent `mut` from here...
print(seen[x])                   // error: seen is lent to walk
let n = walk.count()             // ...to here

fn walk2() -> yields V: ...      // same DFS, using seen as a capture
print(seen[x])                   // fine: walk2 touches seen only while it runs
let n2 = walk2().count()
```

## 2. Plain and heap values (R1, R2)

**R1. Plain values copy.** A value is plain when no part of it lives on the
heap: `Int`, `Bool`, `Char`, `F64`, `Mod<P>`, positions, cuts, ranges,
optionals, tuples, records and enums of plain values, fn values that
capture only plain values. Using a plain value never affects the original.

**R2. Heap values move.** `List`, `Map`, `Set`, `Str`, `[D]T`, `Graph`,
`SegTree` and any struct, tuple or enum holding one move when bound,
passed to a constructor, put in a container or returned. Using a variable
after it moved is a compile error; `x.clone()` is the explicit O(n) copy.
A move is O(1) (a few words), never a deep copy.

```python
var xs = List(1, 2)
let ys = xs                 // xs moves into ys
print(xs.len)               // error: xs moved on the line above; write xs.clone() there
```

```python
let k = 5
let j = k                   // plain: a copy
print(k + j)                // fine
```

`let b = a` where `a` is used again is the error above, never a silent
copy (decision 128). Moving out of a place that stays alive, such as an
element, is `xs[i].take()`, which exists only where the type has an empty
value to leave behind: `None` in a `?T`, the empty `List`, `Str`, map or
set. For anything else, `xs[i].replace(x)` returns the old value.

## 3. Bindings and places

- `let x = e` binds `x` to `e`; `x` can't be reassigned or mutated. (R05.3.1)
- `var x = e` binds a mutable variable: `x = e2`, `x.push(..)`, `x[i] = ..`. (R05.3.2)
- A **place** is a variable, a field (`s.f`), an element (`a[i]`), a slice
  (`a[r]`) or any projection result (§6). Places are what `mut` lends. (R05.3.3)
- `let p = mut a[i]` names a place: `p += 1` writes `a[i]`. `p` is a `mut`
  lend of `a` (R7). (R05.3.4)
- Reading an element or field binds a lend of it, not a copy, when the
  value is heap (`let row = rows[i]`: §6, `at` is a projection). A plain
  element copies (`let x = a[i]`). (R05.3.5)

**R3. Mutation needs a mutable place.** Writing, pushing, calling a
`mut self` method, or passing `mut x` requires `x` to be a `var`, a `mut`
parameter, or a `mut` lend (`let p = mut ...`). `let` bindings and plain
parameters are read-only all the way down.

## 4. Parameters and calls (R4)

**R4. Parameters are lent.** `fn f(xs: List<Int>)` reads the caller's list,
with no copy and no move. The callee can read it, pass it on, lend from it,
but never keep it (R8). `fn f(mut xs: List<Int>)` may also write it, and
the call says so: `f(mut a)`. A method receiver is the same, with no marker
at the call: `fn add(mut self, x: Int)` is called as `st.add(x)` and needs
`st` to be a mutable place.

```python
fn grow(mut xs: List<Int>): xs.push(1)
var a = List<Int>()
grow(mut a)                 // fine
grow(a)                     // error: grow takes `mut xs`; write grow(mut a)
let b = List<Int>()
grow(mut b)                 // error: b is a let; declare it var
```

A `mut` parameter is a place: the callee may assign it whole
(`xs = List()`), which replaces the caller's value.

**Generated constructors move.** `S(a, b)` and struct literals take their
arguments by value (R2): an owned argument moves in, a lent argument makes
the struct lent (R6). That is how a library struct holds a closure or a
source: `Filter(self, keep)`. A custom constructor (`SegTree(max, a)`) is a
static fn and follows R4/R5 below (chapter 2.6 has the one rule).

```python
struct Holder: xs: List<Int>
var a = List(1, 2)
let h = Holder(a)          // a moves into h
print(a.len)               // error: a was moved into h on line 3; write Holder(a.clone())
```

## 5. Returns (R5)

**R5. A fn returns an owned value, unless it is declared `lends`.**
Returning moves, and is O(1). Returning a parameter, part of one, or
anything built from one is returning a lent value, which needs `lends` in
the signature (§6), or `.clone()`.

```python
fn make(n: Int) -> List<Int>:
    var xs = List<Int>()
    for i in 0..n: xs.push(i)
    xs                                  // moves out: O(1)

fn same(xs: List<Int>) -> List<Int> = xs
// error: returns the lent parameter xs; return xs.clone(), or declare `lends List<Int>`
```

**Custom constructors follow R5 too.** A static-fn constructor is a fn, so its parameters
are lent for the call and its result is owned unless it says `lends`. An
owned result never keeps its source busy: the source is free again when
the call returns.

```python
let t = Rooted(g, root: r)        // reads g, owns its arrays: g is free
let fast = LcaTable(t)            // reads t, owns its table: t is free
let ev = xs.filter(p)             // filter is `lends`: xs is lent while ev lives
```

The stdlib picks with one rule: **a result owns its data when copying
what it needs fits within the cost the fn already documents; it lends only
when copying would cost more than building**, which is the case for O(1)
views and adapters (`a[l..r]`, `.filter`, `g.weighted(f)`). So every
O(n)-or-more build (`Graph.directed`, `Rooted`, `LcaTable`, `Lift`,
`Centroids`, `t.virtual`, `SegTree`) returns an owned value, and `lends`
in a signature always means "O(1), and borrows". Sharing a domain (`V`,
`t.Pre`) is not a lend: domains are types, so a table indexed by `t.Pre`
holds nothing of `t`.

## 6. `lends` fns and projections

A fn whose result borrows from its arguments declares it:

```python
fn window(xs: [V]Int, l: V) -> lends [V]Int = xs[l..]       // read lend
pub fn at(mut self, i: D) -> lends mut T                    // a writable place
fn pick(a: List<Int>, b: List<Int>) lends(a) List<Int>   // borrows only a
```

- **Meaning.** The result is a lent value whose sources are every
  non-plain parameter, or the ones named in `lends(...)`. The body is
  checked against that: lending from `b` in `pick` is an error in `pick`. (R05.6.1)
- **Desugaring.** A call passes the rest of the caller's block as a
  callback: `let w = window(a, l); use(w)` is `window(a, l, w => use(w))`.
  That is why no lifetime is needed: the lent value never outlives the
  caller's block. `return` and `break` in that rest-of-block act on the
  caller, because it is the caller's code. (R05.6.2)
- **`lends mut T`** is a place: the caller can assign through it,
  `a[i] = 5` is `a.at(i) = 5`. Code after the tail expression in the body
  runs when the caller is done (write-back: a `SegTree` repairs ancestors). (R05.6.3)
- **Indexing, slicing, `.rev`, `.map`, `.filter` are all `lends` fns.** So
  `let row = rows[i]` and `let w = a[l..r]` are lent values by this rule,
  with nothing special about `[]`. (R05.6.4)

## 7. Lent values (R6 to R8)

**R6. What is lent.** A value is lent when any part of it is lent:
- a parameter (R4), and anything a `lends` fn returns (§6); (R05.7.1)
- a struct, tuple or enum with a lent field (`Filter(self, keep)`); (R05.7.2)
- a generator value with a lent argument (`dfs(g, s)`, `dfs(mut seen)`); (R05.7.3)
- a loop item, for the iteration (§9); (R05.7.4)
- a closure or nested fn that captures a local `var`, a local heap value or
  a parameter (§10). (R05.7.5)

It is a `mut` lend when any part is a `mut` lend. Its **sources** are the
variables its parts borrow, transitively. Everything else is owned.

**R7. While a lent value is alive, its sources are lent.** Alive means from
where it was made to its last use (not to the end of the block).
- **Read lend:** the sources can be read, but not written, resized or moved. (R05.7.6)
- **`mut` lend:** the sources can't be touched at all except through it. (R05.7.7)

```python
var a: [V]Int = 0
let w = a[l..r]
a[l] = 5                    // error: a is lent to w until line 4
print(w.sum())
```
Swap lines 3 and 4 and it compiles: `w` is dead before the write.

**Binding a lent value.** `let ys = xs` with `xs` lent re-lends it (same
sources, no copy). `var ys = xs` with a lent heap value is an error
suggesting `xs.clone()`: a `var` owns what it holds, and taking ownership
would move out of the caller's value. Plain values copy either way.

**R8. Where a lent value may be kept.** A lent value may be bound by `let`, put in a field, or pushed into a container, **if that storage dies
no later than every source**: it is declared in the same block as the
sources or a block nested inside it, and after them. It may never be:
- returned, except from a fn declared `lends` (§6); (R05.7.8)
- stored into anything reached through a parameter (a `mut` parameter's
  storage belongs to the caller); (R05.7.9)
- stored in a top-level `var`. (R05.7.10)

```python
let ws = read<[V]Int>()
var st = SegTree((i, j) => if ws[i] <= ws[j]: i else: j, [V] v => v)   // fine: st dies before ws
```
```python
var best = List<Int>()
for r in rows:
    if r.len > best.len: best = r      // error: r is lent for one iteration; write r.clone()
```
```python
fn keep(mut out: List<Str>, s: Str): out.push(s)
// error: out belongs to the caller; push s.clone()
```

## 8. Overlap and order (R9 to R11)

**R9. Two live lends of overlapping places, one of them `mut`, are an
error unless the compiler proves them apart.**
- Different fields never overlap: `f(mut s.left, mut s.right)` is fine. (R05.8.1)
- Two elements need the projection's `disjoint` law proven from facts at
  that point (`if i != j`, `assert`, `i < j`, value ranges, distinct
  literals). Two slices need the ranges proven apart:
  `solve(mut a[l..m], mut a[m..r])` is fine. (R05.8.2)
- Different projections of one receiver always conflict
  (`g.row(r)` and `g.col(c)`). (R05.8.3)
- A container's length can't change while any place in it is lent:
  `let x = mut xs[i]` then `xs.push(..)` is an error. (R05.8.4)

```python
swap(mut a[i], mut a[j])                // error: a[i] and a[j] may be the same element
if i != j: swap(mut a[i], mut a[j])     // fine
```

**R10. A call is one access to everything it touches:** its arguments, the
variables the called fn or nested fn captures, and the captures of every
closure argument, all for the duration of the call. Those accesses must not
conflict with each other (R9) or with live lends (R7). A capture is borrowed
at each call, not for the closure's whole life, so a closure that writes
`cnt` can be called, then `cnt` read, then the closure called again.

```python
xs.sort(by: x => xs.len - x)
// error: sort holds xs `mut` for the call, and the closure reads xs
```
```python
var a: [V]Int = 0
fn clobber(): a[V.first!] = 0
let part = a[l..r]
clobber()                   // error: clobber writes a, and a is lent to part until line 5
print(part.sum())
```
The effect is computed through any depth of calls (`outer` calling
`clobber` is the same error), in one linear pass over the call graph.

**R11. Evaluation order.** Operands run left to right. An assignment or
compound assignment runs its right side first, then the target's index
expressions, then lends the target for the write, so each lend ends
before the next starts. `a.push(a.len)`, `a[i] = a[j] + 1` and
`cnt[a[i]] += 1` are fine. `xs[0] = xs.pop()!` and `a[l] += inc(mut a[l])`
also compile, with a lint: a right side that writes the target's container
is legal but easy to misread (`inc` runs first, then the new `a[l]` gets
its result added).

## 9. Generators and views (R12)

A **view** is a lent value that is a sequence (`a[l..r]`, `a.filter(p)`,
`a.map(f)`): an ordinary struct whose `src` field holds a lent source, so
R6 to R8 cover it with nothing extra. The same struct built from an owned
value owns it (`Filter(xs, p)` moves `xs`).

A **generator value** is a generator fn with its arguments filled in:
`let g = dfs(graph, s)` runs nothing. It holds its arguments like a struct
holds fields, so `divisors(12)` is owned (it can be stored or returned),
`dfs(graph, s)` is lent from `graph`, and `dfs(graph, s, mut seen)` is a
`mut` lend of `seen`.

**R12. Walking.** `for x in g: B` calls the generator with `B` as its loop
body; every walk reruns it from the top, with fresh locals and the current
state of its arguments.
- The generator's lends are live for the walk, so `B` may read a
  read-lent source but not write it, and may not touch a `mut`-lent one. (R05.9.1)
- The item `x` is the body's parameter: plain items copy, heap items are
  lent for one iteration (R8: keep one with `x.clone()`). (R05.9.2)
- `break`, `continue` and `return` in `B` act on the loop and the
  enclosing fn, through recursive generators too. (`x?` never transfers
  control: it propagates `None` within its expression, chapter 2.) (R05.9.3)
- Walking a `mut`-lent generator value twice is legal and linted: the
  second walk sees what the first changed. (R05.9.4)

```python
var seen: [V]Bool = false
for v in dfs(g, s, mut seen):
    seen[v] = true          // error: seen is lent `mut` to the dfs for the whole loop
```
```python
for x in xs: xs.push(x)          // error: xs is read-lent to the loop
for i in xs.keys: xs.push(xs[i]) // fine: xs.keys is a plain range, fixed before the loop
```

## 10. Closures and nested fns

A lambda (`x => ...`, block lambdas included) and a nested `fn` are the
same thing: a fn value with captures. There is no `mut fn` (decision 5.10):
a nested fn declared after a `var` may read and write it.

- **What is captured.** Plain `let`s are copied in. Local `var`s, local
  heap values and parameters are captured by reference. Top-level names
  are not captures; any fn may use them (a fn that writes a top-level
  `var` has that effect at every call, R10). (R05.10.1)
- **Mode.** A fn value that captures by reference is lent (R6), with the
  captured variables as sources, so R8 limits where it can be kept: it can
  be bound, passed, stored in a local `var` or struct, but not returned or
  stored past those variables. A captured variable can't be moved while
  the fn value is alive. (R05.10.2)
- **Calls.** Captures are borrowed per call (R10), and `mut` captures are
  exclusive for that call: passing `mut a[l]` to a fn that also writes the
  captured `a` is an error. (R05.10.3)
- **Arguments.** A closure passed to a fn is a parameter there (R4): the
  callee can call it but can't keep it, so every closure argument is
  inlined and costs nothing. Only constructors keep closures (§4). (R05.10.4)
- **Recursion.** Nested fns and `let`-bound lambdas may call themselves. (R05.10.5)
- **Spelling differences only.** A nested `fn` can take explicit type
  parameters (`fn go<T>(..)`), attributes (`@memo`) and sibling fns in
  the same block, so mutual recursion works. A lambda infers its
  parameter types from where it is used, and its `let` name is visible
  only from its own body on. Both can be generators (`-> yields T`). (R05.10.6)
- **`return`** inside a lambda returns from the lambda. (Non-local exit
  from closures is deferred, decision 126; use a `for` over a generator.) (R05.10.7)
- **`@memo`** fns may not capture a `var` they could see change. (R05.10.8)

```python
var cnt = 0
let inc = () => cnt += 1
inc()
print(cnt)                  // fine: inc borrows cnt only during each call
inc()
```
```python
fn counter() -> (() -> Int):
    var c = 0
    () => c                 // error: the lambda captures local c, so it can't be returned
```

## 10a. Caches: `@cache` fields

A method that is logically a read can write a field marked `@cache`
through a read `self`: path compression in `Dsu.find`, lazy tables in
`Comb`, tag pushes in a lazy segment tree query, splay-tree reads.

```python
pub struct Dsu<V>:
    @cache parent: [V]V
    size: [V]Int

    pub fn find(self, x: V) -> V:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r
```

Checked by the compiler (these make it memory-safe):

- **C1. No lend escapes.** No `lends` result, projection or stored value
  borrows the field, so no outside view can see it change (as with Rust's
  `Cell`). (R05.10.9)
- **C2. Only the struct's own methods write it.** Outside code can't, even
  through `mut`. (R05.10.10)
- **C3. Inside the struct, a cache write is a `mut` access to the field.**
  A method walking `self.parent` can't call `self.find` partway through.
  Only callers are relaxed. (R05.10.11)
- **C4. No unknown calls while the field is lent.** A method holding a
  lend into the field may not call a closure or fn parameter: it could
  capture the struct and call `find`, writing under the method's view. (R05.10.12)
- **C5. Derived `==`, `hash`, `Ord` and `print` skip `@cache` fields.**
  Otherwise `d1 == d2` could flip after a `find`; a struct that needs them
  compared writes the impl by hand. (R05.10.13)

Promised by the author (this makes it correct):

- **P1. Cache writes change no answer.** Every sequence of public calls
  gives the same results as if each cache write through a read `self` were
  undone at method exit. Equivalently, the abstract value (the partition,
  the factorials) never changes. (R05.10.14)

Stress runs check P1 with a cold mode that undoes those writes at method
exit and compares output with a normal run; P1 is exactly what makes the
two comparable, so no struct needs its own checker.
- Shared ownership (two owners of one mutable value) stays out: pass
  `mut x` to each call, or capture `x`.

```python
let d = build_dsu(edges)            // a let
for v in V: print(d.find(v))        // fine: find takes read self
```

## 11. Top-level state

The file body is a fn (decision 6.1). Top-level `let`s are visible to
every fn; a top-level `var` is visible to fns declared after it, which may
write it. R10 treats such a fn's use of a top-level `var` as part of every
call to it. Nothing lent may be stored in a top-level `var` (R8).

## 12. How to evaluate a snippet

1. **Mark each binding** owned or lent (R6). For lent ones, note the
   sources and the line of last use.
2. **Moves:** any use of a heap variable after it moved (R2)?
3. **Mutation:** is every write, `mut` argument and `mut self` call on a
   mutable place (R3)?
4. **Live lends:** between each lent value's creation and last use, does
   anything write, resize or move a source, or touch a `mut`-lent source
   (R7)? Include writes done by called fns and closures (R10).
5. **Keeping:** is a lent value returned without `lends`, stored through a
   parameter or globally, or stored somewhere that outlives a source (R8)?
6. **Overlap:** does any call or statement hold two overlapping places with
   one `mut`, without a proof they differ (R9)?
7. **Order:** evaluate right side, then target indices, then the write
   (R11); only lends alive at the same moment can conflict.

If every step passes, it compiles.

## 13. What this page settles

Earlier documents disagree with each other in places. This page picks:

| Topic | Earlier text | Now | Why |
|---|---|---|---|
| Copy spelling | `.copy()` (5.2, plan/14) | `.clone()` | 128, 141 and `std.seq` already say `clone` |
| Mutating nested fns | `mut fn` (plan/14, plan/15) | any nested fn after a `var` | 5.10 deleted `mut fn` |
| Stored pull iterators | `Iterator<T>`, pull for `zip` (4.2, 4.4) | push only; lockstep is `List(g)` | 126, 141 |
| Plain vs heap | "≤ 16 bytes copies" (3.5) | "no heap part" copies | size must not change meaning |
| Read-lent sources | "none changes length" (W4) | no writes at all | plan/14's exclusive-mutation rule; red-team case 1 |
| Lent values in `var`s, fields, containers | never (5.14, 141 rule 3) | allowed when the storage dies no later than its sources | `var st = SegTree(V, op: closure over ws)` must work |
| Lent struct construction | only inside projection bodies (plan/19 §5.1) | anywhere | 141 makes view structs plain |
| Closure captures | held for the closure's life (implied by 141 rule 4) | borrowed per call | calling a counter closure, reading the counter, calling it again must work |
| Inline closures reusing the callee's `mut` borrow | allowed (5.4) | withdrawn | its client (`undo`/`with`) was deleted (5.8); it was the one unsound corner |
| Mutable `let` aliases | left out of the first version (plan/19 §5.3) | `let p = mut a[i]` allowed | implemented and used by ports |
| `let row = rows[i]` on a heap element | unclear (move? copy?) | a read lend | `at` is a `lends` fn; no special case for `[]` |
| Interior mutability | `Comb` alone had it (19.10) | `@cache` fields, for any struct | no stdlib privilege; `Dsu.find` needs it too |

## 13a. Why, compared with Rust, and limits

**Why mutation XOR access.** Every aliasing bug needs a writer and another
live view of the same data at once; the rule forbids exactly that pair.
It removes four classes:

1. **Invalidation:** a view depends on length or storage and a write
   moves it (`for x in v: v.push(x)`, holding `v[0]` across a push).
2. **Stale reads:** a reader assumes the data holds still (a comparator
   reading the array being sorted, a slice summed while a callee writes).
3. **Self-aliasing:** two names for one thing where code assumed two
   (`merge(mut a, mut b)` that clears `b` when `a` is `b`).
4. **Lost locality:** without it, "nothing here touched `x`" needs
   whole-program knowledge, for the reader and the optimizer.

Two `mut` lends are banned because every `mut` lend also reads. Values,
not objects, already removes most aliasing; XOR is checked only on lends.

**Compared with Rust.** Our lends are second-class: never in a type, never
past the caller's block.

- No `&T`, `&mut T` or lifetimes; a fn says `lends` / `lends(x)` instead.
- Parameters are lent or `mut`; a callee never takes ownership.
- Plain values copy and heap values move by rule (no `derive(Copy)`).
- One closure kind; captures are borrowed per call, not for the closure's
  life, so `let inc = () => cnt += 1; inc(); print(cnt); inc()` compiles.
- Projections are inline callbacks, so they can repair after a write.
- `if i != j` proves disjoint `mut` places; no `split_at_mut`.
- Push generators, so no lending-iterator problem.

Closest relatives: Hylo (mutable value semantics, `inout`, yield
subscripts), Swift (non-escaping closures, exclusivity, `_modify`), Mojo
(`read`/`mut`/`owned`), and second-class values (Osvald et al., 2016).

**Not expressible, on purpose:** closures that outlive their captures
(returned counters, stored callbacks); two owners of one mutable value
(pass `mut x` per call); a paused generator (use `List(g)` for lockstep);
a callee that takes ownership (`mut xs` plus `xs.take()`); pointers
between nodes (domain indices). Closures of different types never share
a container, since each closure has its own type and nothing lowers to
`std::function` (see R06.2.5): use an enum of the cases.

**Sharp edges** (errors and hovers must carry these):

1. `let` copies a plain element, lends a heap element, moves a variable.
2. The mode is not in the type: `xs.filter(p)` is lent, `Filter(xs, p)`
   owned; hover shows which.
3. Passing `mut seen` freezes it for the value's life; capturing it
   freezes it per walk.
4. A later use extends a lend, so an added `print` can break a write
   between; a nested fn that writes `a` conflicts at its calls.
5. Declaration order matters for R8.
6. A record gaining a heap field stops copying and starts moving; a lint
   or hover badge for plain-ness is open.
7. Re-walking a `mut`-lent generator reruns it against changed state
   (linted).

**Lowering, in one line** (the companion doc is paused): a struct's code
is written once; in C++ each mix of owned and lent fields the program
actually builds is one template instantiation (owned by value, lent by
reference or span), so usually one or two per struct.

## 14. Exercises

Does each snippet compile? `V`, `a: [V]Int`, `g: Graph<V, E>`, `rows:
List<List<Int>>` and `i, j, l, r: V` are in scope unless the snippet
declares them; `a` and `rows` are `var`s. Answers follow each one.

**1.**
```python
var xs = List(1, 2)
let ys = xs
print(ys.len)
```
<details><summary>Answer</summary>Yes. <code>xs</code> moves into <code>ys</code> (R2); <code>xs</code> is never used again.</details>

**2.**
```python
var xs = List(1, 2)
let ys = xs
print(xs.len + ys.len)
```
<details><summary>Answer</summary>No. <code>xs</code> is used after it moved (R2). Fix: <code>let ys = xs.clone()</code>.</details>

**3.**
```python
let p = (x: 3, y: 4)
let q = p
print(p.x + q.y)
```
<details><summary>Answer</summary>Yes. A record of <code>Int</code>s is plain and copies (R1).</details>

**4.**
```python
fn twice(xs: List<Int>) -> List<Int>:
    var ys = xs
    ys.extend(xs)
    ys
```
<details><summary>Answer</summary>No. <code>xs</code> is a lent parameter; <code>var ys = xs</code> would take ownership of a lent heap value ("Binding a lent value", before R8). Fix: <code>var ys = xs.clone()</code>.</details>

**5.**
```python
fn grow(mut xs: List<Int>): xs.push(0)
let b = List<Int>()
grow(mut b)
```
<details><summary>Answer</summary>No. <code>b</code> is a <code>let</code>, not a mutable place (R3).</details>

**6.**
```python
let w = a[l..r]
print(w.sum())
a[l] = 5
```
<details><summary>Answer</summary>Yes. <code>w</code>'s last use is before the write (R7).</details>

**7.**
```python
let w = a[l..r]
a[l] = 5
print(w.sum())
```
<details><summary>Answer</summary>No. <code>a</code> is read-lent to <code>w</code> until line 3 (R7).</details>

**8.**
```python
let row = rows[i]
rows.push(List())
print(row.len)
```
<details><summary>Answer</summary>No. <code>row</code> is a lend of <code>rows</code> (§6), and pushing resizes a source (R7). Fix: <code>rows[i].clone()</code>, or print first.</details>

**9.**
```python
var longest = List<Int>()
for row in rows:
    if row.len > longest.len: longest = row
```
<details><summary>Answer</summary>No. <code>row</code> is lent for one iteration and <code>longest</code> outlives it (R8, R12). Fix: <code>longest = row.clone()</code>, or keep the index with <code>rows.argmax(by: r => r.len)</code>.</details>

**10.**
```python
for x in rows[i]: rows[i].push(x)
```
<details><summary>Answer</summary>No. <code>rows[i]</code> is read-lent to the loop (R12).</details>

**11.**
```python
for k in rows[i].keys: rows[i].push(rows[i][k])
```
<details><summary>Answer</summary>Yes. <code>.keys</code> is a plain range taken before the loop; each statement's lends end with it (R11).</details>

**12.**
```python
swap(mut a[i], mut a[j])
```
<details><summary>Answer</summary>No. Nothing proves <code>i != j</code> (R9). Fix: <code>if i != j:</code> or <code>a.swap_at(i, j)</code>.</details>

**13.**
```python
fn solve(mut lo: [V]Int, mut hi: [V]Int) = todo
let m = l.next(5)!
solve(mut a[l..m], mut a[m..r])
```
<details><summary>Answer</summary>Yes. The ranges meet at <code>m</code> and can't overlap (R9).</details>

**14.**
```python
let p = mut a[i]
p += 1
a[j] = 0
p += 1
```
<details><summary>Answer</summary>No. <code>a</code> is <code>mut</code>-lent to <code>p</code> until line 4 (R7). Without line 4 it compiles.</details>

**15.**
```python
a.push(a.len)
cnt[a[i]] += 1
```
<details><summary>Answer</summary>Yes. Arguments and indices are evaluated before the target is lent (R11). (Assume <code>a</code> is a <code>List&lt;Int&gt;</code> here and <code>cnt</code> a <code>var</code> map.)</details>

**16.**
```python
fn inc(mut x: Int) -> Int:
    x += 1
    x
a[l] += inc(mut a[l])
```
<details><summary>Answer</summary>Yes, with a lint. The right side runs first, so <code>a[l]</code> is incremented, then <code>inc</code>'s result is added to the new value (R11).</details>

**17.**
```python
var cnt = 0
let bump = () => cnt += 1
bump()
print(cnt)
bump()
```
<details><summary>Answer</summary>Yes. Captures are borrowed per call (R10).</details>

**18.**
```python
fn counter() -> (() -> Int):
    var c = 0
    () => c
```
<details><summary>Answer</summary>No. The lambda captures local <code>c</code>, so it is lent and can't be returned (R5, R8).</details>

**19.**
```python
fn adder(k: Int) -> (Int -> Int) = x => x + k
```
<details><summary>Answer</summary>Yes. <code>k</code> is plain, so it is copied in; the lambda is owned (§10).</details>

**20.**
```python
let ws = read<[V]Int>()
var st = SegTree((x, y) => if ws[x] <= ws[y]: x else: y, [V] v => v)
st[i] = i
```
<details><summary>Answer</summary>Yes. <code>st</code> is lent from <code>ws</code> and declared after it in the same block (R8). (<code>SegTree</code>'s element type here is <code>V</code>.)</details>

**21.**
```python
var xs = List(3, 1, 2)
xs.sort(by: x => xs.len - x)
```
<details><summary>Answer</summary>No. <code>sort</code> holds <code>xs</code> <code>mut</code> for the call, and the closure argument reads <code>xs</code> (R10).</details>

**22.**
```python
fn clobber(): a[l] = 0
let part = a[l..r]
clobber()
print(part.sum())
```
<details><summary>Answer</summary>No. <code>clobber</code> writes <code>a</code> while it is lent to <code>part</code> (R7, R10).</details>

**23.**
```python
var seen: [V]Bool = false
for v in dfs(g, s, mut seen):
    if seen[v]: print(v)
```
<details><summary>Answer</summary>No. <code>seen</code> is <code>mut</code>-lent to the generator for the whole walk, so even reading it is excluded (R12). Fix: let the generator report it, or materialize the walk with <code>List(...)</code> first.</details>

**24.**
```python
var seen: [V]Bool = false
let walk = dfs(g, s, mut seen)
let n1 = walk.count()
let n2 = walk.count()
```
<details><summary>Answer</summary>Yes, with a lint: the second walk reruns the DFS against the <code>seen</code> the first one filled, so <code>n2</code> is 0 (R12).</details>

**25.**
```python
let gs = List(divisors(12), divisors(30))
for d in gs[0]: print(d)
```
<details><summary>Answer</summary>Yes. <code>divisors(12)</code> captures only an <code>Int</code>, so it is owned and can live in a list (§9).</details>

**26.**
```python
fn tail(xs: [V]Int, l: V) -> [V]Int = xs[l..]
```
<details><summary>Answer</summary>No. It returns a lent slice without <code>lends</code> (R5). Fix: <code>lends [V]Int</code>, or return the range <code>l..</code> and slice at the caller.</details>

**27.**
```python
fn pick(x: List<Int>, y: List<Int>) lends(x) List<Int> = y[..]
```
<details><summary>Answer</summary>No. The body lends from <code>y</code> but the signature says only <code>x</code> (§6).</details>

**28.**
```python
fn keep(mut out: List<Str>, s: Str): out.push(s)
```
<details><summary>Answer</summary>No. <code>s</code> is lent, and <code>out</code> belongs to the caller (R8). Fix: <code>out.push(s.clone())</code>.</details>

**29.**
```python
struct Pair: left: List<Int>, right: List<Int>
fn both(mut x: List<Int>, mut y: List<Int>) = todo
var p = Pair(List(), List())
both(mut p.left, mut p.right)
both(mut p.left, mut p.left)
```
<details><summary>Answer</summary>Line 4 compiles (different fields, R9); line 5 doesn't (the same place twice).</details>

**30.**
```python
var depth: [V]Int = 0
fn visit(v: V, from: ?V):
    for (to: u, ..) in g[v]:
        if u != from:
            depth[u] = depth[v] + 1
            visit(u, v)
visit(V.first!, None)
print(depth.max())
visit(V.first!, None)
```
<details><summary>Answer</summary>Yes. A nested fn after a <code>var</code> may write it; the capture is borrowed only during each call (§10, R10).</details>

**31.**
```python
var xs = List(1, 2)
let f = () => xs.len
let ys = xs
print(f())
```
<details><summary>Answer</summary>No. <code>xs</code> is captured by <code>f</code>, which is alive until line 4, so it can't move (§10).</details>

**32.**
```python
let evens = a.filter(x => x % 2 == 0)
let total = evens.sum()
let best = evens.argmax()
a[l] = 0
```
<details><summary>Answer</summary>Yes. Two walks of a read-lent view are fine; <code>evens</code> is dead before the write (R7).</details>

**33.**
```python
fn evens(xs: [V]Int) -> lends Filter<[V]Int, Int -> Bool> = xs.filter(x => x % 2 == 0)
for x in evens(a): a[l] += x
```
<details><summary>Answer</summary>No. The result borrows <code>a</code> for the loop, and the body writes <code>a</code> (R12).</details>

**34.**
```python
var a2 = a.clone()
for x in a.filter(x => x > 0): a2[l] += x
```
<details><summary>Answer</summary>Yes. The loop lends <code>a</code>; the body writes the separate <code>a2</code>.</details>

**35.**
```python
var best: ?[V]Int = None
for (l2, r2) in queries:
    best = a[l2..r2]
print(best!.sum())
```
<details><summary>Answer</summary>Yes: <code>best</code> is declared after <code>a</code>'s declaration in an enclosing block, so it dies first (R8), and <code>a</code> stays read-lent until line 4. The idiomatic version keeps the range (<code>var best: ?Range&lt;V&gt;</code>) so <code>a</code> isn't frozen meanwhile.</details>
