# Ownership, mutation and lending

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/05-ownership.md` |
| Describes | `spec/core/05-ownership.md`, `spec/conformance/core-05/`, `crates/cpl-check` (the lend checker) |

CP Lang has no pointers, no reference types and no lifetimes. Every value
is either **owned** or **lent**, the compiler infers which, and twelve
rules (R1 to R12 in `spec/core/05-ownership.md`) decide what each may do.
This doc explains the model, why it is shaped this way, and how it
compares with Rust, C++ and Swift. Terms follow `spec/GLOSSARY.md`: a
*domain* is a finite index space, a *position* is one of its elements, a
*place* is anything you can write to (`x`, `s.f`, `a[i]`, `a[l..r]`).

## The problem

Contest code mutates constantly: arrays filled by a DP, a DSU compressed
while it is read, a vector pushed to while it is walked. In C++ the
mistakes that matter are aliasing mistakes, and none of them is caught:

```cpp
vector<int> v = {1, 2, 3};
for (int x : v) v.push_back(x);   // iterator invalidation: undefined behaviour
int &first = v[0];
v.push_back(4);                   // `first` may now dangle
sort(v.begin(), v.end(), [&](int a, int b) { return v.size() - a < b; });
                                  // the comparator reads the array being sorted
vector<int> best = rows[i];       // silent O(n) copy hidden in an `=`
```

Python removes undefined behaviour but adds the opposite bug, sharing:

```python
dp = [[0] * m] * n     # n names for one row; dp[0][0] = 1 sets every row
b = a                  # b.append(x) also changes a
```

The two languages fail in opposite directions. C++ copies silently and
aliases through references; Python never copies and aliases everything.
Rust fixes both, but at the price of `&`, `&mut`, lifetimes and
`split_at_mut` in a program that is one page long and written in twenty
minutes. CP Lang's goal is Rust's guarantees with Python's surface: no
undefined behaviour and no aliasing bug can be written, and a typical
contest program never mentions ownership at all.

## The model

Read the examples first; the rules after them say the same thing.

```cp
var xs = [1, 2]
let ys = xs  // xs moves into ys: O(1), no copy
print(ys.len)

let p = (x: 3, y: 4)
let q = p  // plain record: a copy
print(p.x + q.y)

fn grow(mut xs: List<Int>): xs.push(1)
var a = List<Int>()
grow(mut a)  // `mut` at both ends: you see who writes a
print(a.len)
```

```cp-fragment
domain V = 0..8
var a: [V]Int = 0
let w = a[V(2)..V(5)]        // w is a view: a lend of a
a[V(2)] = 5                  // error E0602: a is lent to w until the next line
print(w.sum())
```

Swap the last two lines and it compiles: a lend lasts until its last use,
not to the end of the block.

What a contestant holds in their head is six sentences (chapter 5 §1):

1. A variable owns its value; `let` can't change it, `var` can.
2. Small values with no heap part copy; heap values (`List`, `Map`,
   `Str`, `[D]T`, structs holding them) move, and copying one is
   `.clone()`.
3. A parameter is a lend of the caller's value: the callee can use it
   but never keep it, and `mut` (written at both ends) lets it change it.
4. A value built from a lend is itself lent: slices, views, generator
   values, loop items, projection results, closures over local state.
5. While a lent value is alive (until its last use) its sources are
   frozen for writes, or untouchable if it is a `mut` lend; and it can
   never be kept anywhere that outlives them.
6. Each call is checked as one access to everything it touches: its
   arguments and whatever the called fn or closure captures.

And one picture: every live lent value holds an arrow to the variables it
borrows. A variable with an arrow at it is frozen (no writes under a read
lend, no access at all under a `mut` lend). The arrow disappears at the
value's last use, and it may never outlive the variable it points at.
That is the whole checker.

The question the checker asks is always the same: **does anyone hold a
live view that this write would break or change underneath them?**

## Key decisions

### 1. Values, not objects

**Chose:** no two variables ever name the same mutable value. `let b = a`
moves or copies, never aliases (decisions 5.1, 128). A graph of objects
that point at each other is written as arrays over a domain, with
positions as links.

**Lost:**

| Option | What `let b = a` does | Why it lost |
|---|---|---|
| C++ value semantics | deep copy | an \(O(n)\) step hides in `=`; violates "costs are visible" |
| Python / Java references | shares | `b.push(x)` changes `a`: the aliasing bug class we exist to remove |
| Rust ownership + references | move, plus `&`/`&mut` types | precise, but lifetimes and borrow types cost annotations in one-page programs |
| CP Lang | move at last use, else an error naming `.clone()` | |

**Why:** values give *locality*. If no other name can reach `x`, then
"nothing on this screen touched `x`" is a fact the reader and the
optimizer can both rely on, without whole-program knowledge. Mutation is
always to a name you can see. Almost every aliasing bug needs two names
for one thing; values remove most of them before any rule about lending
is needed, so the lend rules only have to cover the short-lived views
that remain.

### 2. Plain copies, heap moves, `.clone()` is the only copy

**Chose:** a value is *plain* when no part of it lives on the heap
(numbers, `Bool`, `Char`, positions, cuts, ranges, optionals, tuples,
records and enums of plain values). Plain values copy. Heap values move
when bound, passed to a constructor, stored or returned; a move is a few
words, never a deep copy (R1, R2; decision 128). Using a variable after
it moved on any path is a compile error (E0608), and the fix the error
names is `.clone()`.

```cp
var xs = [1, 2]
let ys = xs.clone()  // the O(n) step is spelled out
print(xs.len + ys.len)
```

**Lost:** "copies when at most 16 bytes" (an earlier draft): a record
gaining one field would silently change from copy to move, so size must
not change meaning; *no heap part* is the rule instead (chapter 5 §13).
Rust's `derive(Copy)`: an annotation for something the compiler can
decide. `.copy()` as the name: `std.seq` and decision 141 already said
`.clone()`.

**Why:** moving is what a contestant already means when a value has no
later use. Returning a container is \(O(1)\) because returns move, so
`fn make() -> List<Int>` is free. The one place a move isn't what you
meant (using `a` again) is an error at that line, never a silent copy.

Moving out of a place that stays alive, such as an element, is
`xs[i].take()`, which exists only where the type has an empty value to
leave behind; otherwise `xs[i].replace(x)` returns the old value.

```cp
var win = [[1], [2], [3], [4]]
let k = 3
let l = 1
if k > l:
    win[k] = win[l].take()  // `k > l` proves the two places apart
print(win[k].len, win[l].len)
```

### 3. Parameters are lends; callees never take ownership

**Chose:** `fn f(xs: List<Int>)` reads the caller's list with no copy and
no move. `fn f(mut xs: List<Int>)` may also write it, and the call must
say `f(mut a)` (R3, R4; decision 128). A receiver works the same way with
no marker at the call: `st.add(x)` needs `st` to be a mutable place.

**Lost:** callee-takes-ownership parameters (Rust's `fn f(xs: Vec<i32>)`,
Swift's `consuming`). A callee that wants to drain an argument takes
`mut xs` and calls `xs.take()`; the caller keeps the (now empty) variable.
Implicit `mut` at the call site (C++ `T&`, Swift without `&`): the reader
of `f(a)` could not tell that `a` changes.

**Why:** with parameters always lent, a call can never end a variable's
life, so a contestant never has to think about who owns what across a
call. `mut` at both ends makes every write visible where it happens.

**Optional `mut` parameters** (R05.4.5; decision 447). `dfs(g, from: s,
seen: mut seen)` shares one visited table across walks, and `dfs(g, from:
s)` makes its own, so `seen` is a `?mut [V]Bool = None`: a `mut x` or
nothing. The body reads it with the `if let s = seen:` every optional
already uses, and `s` is the caller's place, so `s[v] = true` writes `x`.
Nothing new to learn at the call: it is still `mut` where the write is
visible. The alternative, two overloads or a separate `dfs_with`, is the
`_by` variant the stdlib rules out. Lowering makes one instance per call
shape, so the C++ is a plain reference parameter with no null check.

### 4. Lending is a mode, not a type

**Chose:** whether a value is owned or lent is not part of its type
(R6; decision 141). A slice `a[l..r]` has the same type as the array it
came from; `xs.filter(p)` is lent and `Filter(xs, p)` is owned, and they
are the same struct. The only place the word appears is a fn signature:
`-> lends T` when a result borrows from the parameters, `-> lends(x) T`
to say which.

```cp
domain V = 1..=4
fn window(xs: [V]Int, l: V) -> lends [V]Int = xs[l..]
let a = [V] v => Int(v)
print(window(a, V(3)).sum())
```

**Lost:**

- *Borrow types and lifetimes* (Rust `&'a [T]`): every view type,
  struct holding a view, and fn returning one carries a lifetime
  parameter. Rejected in decision 141: five rules instead of a borrow
  checker's vocabulary.
- *A second kind of struct for views* (a `lends S` marker on view
  structs, an earlier draft): one `Filter` that owns or borrows by what
  built it is simpler, and library authors write each adapter once.
- *Lent values only in `let`s* (decision 5.14, then rule 3 of 141):
  relaxed by decision 144 so a lent value can sit in a `var`, field or
  container that dies no later than its sources. The case that forced it:

```cp-fragment
domain V = 1..=4
let ws: [V]Int = [V] v => 5 - Int(v)
var st = SegTree((x, y) => if ws[x] <= ws[y]: x else: y, [V] v => v)
st[V.first!] = V.first!
```

`st` holds a closure that reads `ws`, so `st` is lent from `ws`. It is
declared after `ws` in the same block, so it dies first, and R8 allows it.

**Why:** a mode the compiler infers costs the contestant nothing to
write. Hover shows which mode a binding has, and the errors name the
source and the line of last use.

### 5. Second-class lends instead of lifetimes

**Chose:** a lent value can never outlive the caller's block (R8). A
`lends` fn is checked as if the rest of the caller's block were passed to
it as a callback: `let w = window(a, l); use(w)` means
`window(a, l, w => use(w))` (R05.6.2). Because the lend can't escape that
callback, no lifetime is ever needed. `return` and `break` in the rest of
the block still act on the caller, because it is the caller's code.

```cp
domain V = 1..=4
fn window(xs: [V]Int, l: V) -> lends [V]Int = xs[l..]
fn f(a: [V]Int) -> Int:
    let w = window(a, V(2))
    if w.sum() > 5: return 1
    0
print(f([V] v => Int(v)))
```

What is not allowed is keeping a lent value anywhere that outlives a
source: returning it without `lends`, storing it through a parameter, or
storing it in a top-level `var` from a fn (R05.7.8 to R05.7.10).

```cp-fragment
var rows = [[1, 2], [3]]
var best = List<Int>()
for r in rows:
    if r.len > best.len: best = r      // error E0603: r is lent for one iteration; write r.clone()
```

**Lost:** first-class references with inferred lifetimes. They would let
a fn return a view into a struct field and a struct store a view of
another struct, which contest code almost never needs, and the cost is
the whole lifetime vocabulary in every error message.

**Why:** this is the *second-class values* idea (Osvald et al., 2016):
values that may be passed down but never returned up or stored need no
lifetime analysis, only scopes. The callback reading of `lends` also
gives projections a superpower: code after the tail expression of a
`lends mut` fn runs when the caller is done, so `SegTree`'s `st[i] = x`
can repair the ancestors after the write (R05.6.3).

```cp
fn head(mut xs: List<Int>) -> lends mut Int = xs[0]
var a = [1, 2]
head(mut a) = 7  // a writable place, returned by a fn
a[1] = 5
print(a[0], a[1])
```

### 6. Lends last until last use

**Chose:** a lent value is alive from where it was made to its last use,
not to the end of its block (R7). Rust calls this non-lexical lifetimes.

```cp
domain V = 0..8
var a: [V]Int = 0
let w = a[V(2)..V(5)]
print(w.sum())  // last use of w
a[V(2)] = 5     // fine: w is dead
print(a[V(2)])
```

**Lost:** lexical lends (until end of block). Simpler to state, but they
reject the code above, which every contestant expects to work.

**Why:** it matches intuition. The sharp edge is that adding a later use
extends the lend, so an added debugging `print(w)` can break a write in
between (chapter 5 §13a, "Sharp edges", item 4); the error names both lines.

### 7. Mutation XOR access, checked only on lends

**Chose:** while a lent value is alive, a **read lend** lets its sources
be read but not written, resized or moved, and a **`mut` lend** lets no
one touch them except through it (R7). Two live lends of overlapping
places, one of them `mut`, are an error unless the compiler proves them
apart (R9).

**Why this rule:** every aliasing bug needs a writer and another live
view of the same data at the same moment, and this rule forbids exactly
that pair. It removes four bug classes (chapter 5 §13a):

| Class | Example |
|---|---|
| Invalidation | `for x in v: v.push(x)`; holding `v[0]` across a push |
| Stale reads | a comparator reading the array being sorted; a slice summed while it changes |
| Self-aliasing | `merge(mut a, mut b)` that clears `b` when `a` is `b` |
| Lost locality | "nothing here touched `x`" needing whole-program knowledge |

Two `mut` lends of one place are banned too, because every `mut` lend
also reads.

**Lost:** an earlier rule that a read lend only forbids length changes
(so `a[i] = x` under a live view of `a` was allowed). It permits the
stale-read class, and plan red-team case 1 found the program it breaks;
"no writes at all" replaced it (chapter 5 §13).

### 8. Disjointness from facts, not from APIs

**Chose:** the checker proves two places apart from the facts in scope:
different fields never overlap; two elements are apart when
`if i != j`, `assert`, `i < j`, value ranges or distinct literals prove
it; two slices are apart when their ranges are (R9, R05.8.1 to R05.8.2;
decision 5.13).

```cp
struct S:
    var left: List<Int>
    var right: List<Int>
fn mv(mut a: List<Int>, mut b: List<Int>): a.push(b.pop()!)
var s = S(left: [1], right: [2])
mv(mut s.left, mut s.right)  // different fields
print(s.left.len, s.right.len)
```

```cp
fn f(mut a: List<Int>, i: Int, j: Int):
    if i != j: swap(mut a[i], mut a[j])  // `i != j` is the proof
var a = [1, 2, 3]
f(mut a, 0, 2)
print(a[0], a[2])
```

```cp-fragment
domain V = 1..=4
fn paint(mut x: [V]Int, mut y: [V]Int):
    for k in x.keys: x[k] = 1
    for k in y.keys: y[k] = 2
var a: [V]Int = 0
let c = V(3).l
paint(mut a[..c], mut a[c..])          // the ranges meet at c and can't overlap
print(a[V(2)], a[V(3)])
```

**Lost:** Rust's `split_at_mut`, `swap` by index and `get_many_mut`: one
API per shape of proof, which a contestant must know exists. A runtime
distinctness check is listed in decision 128 as a possible later
relaxation, not adopted.

**Why:** the proof a contestant would write anyway (`if i != j`) is the
proof the checker reads. A library projection states when two of its
results are apart with `@law disjoint(i, j) = i != j`; single-index
forwarding derives it automatically (decision 5.13).

### 9. A call is one access; closures borrow per call

**Chose:** a call touches its arguments, the variables the called fn
captures, and the captures of every closure argument, all for the
duration of the call, and those accesses must not conflict (R10). A
closure's captures are borrowed *at each call*, not for the closure's
whole life (decision 144).

```cp
var cnt = 0
let inc = () => cnt += 1
inc()
print(cnt)  // fine: inc borrows cnt only while it runs
inc()
print(cnt)
```

```cp-fragment
var xs = [3, 1, 2]
xs.sort(by: x => xs.len - x)
// error E0602: sort holds xs `mut` for the call, and the closure reads xs
```

The effect of a call is computed through any depth of calls in one
linear pass over the call graph, so a helper that writes `a` conflicts
with a live view of `a` even when called through another helper.

**Lost:**

- *Captures held for the closure's life* (Rust's model for a closure
  that captures `&mut cnt`): rejects the counter above, which reads
  `cnt` between calls.
- *`mut fn`* for nested fns that write outer state (decision 5.10): any
  nested fn declared after a `var` may write it, and the checker tracks
  what it touches.
- *Inline closures reusing the callee's `mut` borrow* (decision 5.4):
  withdrawn by decision 144; its one client was deleted and it was the
  model's one unsound corner.

**Why:** "fns and closures are code, not views". They look at state only
while running, which is how a contestant thinks of a recursive `dfs`
nested in `main` that fills `depth`:

```cp
domain V = 1..=4
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3)]
let g = Graph.undirected(es)
var depth: [V]Int = 0
fn visit(v: V, from: ?V):
    for (to: u, ..) in g[v]:
        if u != from:
            depth[u] = depth[v] + 1
            visit(u, v)
visit(V.first!, None)
print(depth.max()!)
```

A closure that captures local state is lent (R05.10.2), so it can't be
returned past that state:

```cp-fragment
fn counter() -> (() -> Int):
    var c = 0
    () => c                  // error E0603: the lambda captures local c
```

A closure passed to a fn is a parameter there, so the callee can call it
but not keep it, which is why every closure argument is inlined and
costs nothing (R05.10.4). Only constructors keep closures.

### 10. Push generators, so loops keep their control flow

**Chose:** library loops (`dfs`, `divisors`, submasks) are generators that
push items into the loop body (decision 126). A generator value is a
recipe with its arguments filled in; it runs nothing until walked, and
every walk reruns it (R12). It holds its arguments like a struct holds
fields, so `divisors(12)` is owned and `dfs(g, from: [s])` is lent from
`g`.

```cp
fn divisors(n: Int) -> yields Int:
    for d in 1..=n.isqrt():
        if n % d == 0:
            yield d
            if d * d != n: yield n / d
let gs = List(divisors(12), divisors(30))  // owned generator values in a list
for d in gs[0]: print(d)
```

`break`, `continue` and `return` in the body act on the loop and the
enclosing fn, even through a recursive generator:

```cp
fn walk(n: Int) -> yields Int:
    yield n
    if n > 0:
        for x in walk(n - 1): yield x
fn f() -> Int:
    for x in walk(10):
        if x == 7: return x
    0
print(f())
```

**Lost:** pull-style stored iterators (`Iterator<T>` with `next()`).
They need a paused generator's frame to live somewhere, which is where
Rust's "lending iterator" problem comes from, and they make `return` from
inside a traversal awkward. Lockstep over two generators is `List(g)`.

**Why:** push generators keep every lend inside one loop, so the loop
item is lent for exactly one iteration (R05.9.2), and the generator's own
lends are live for exactly the walk (R05.9.1).

### 11. `@cache` fields: interior mutability with a promise

**Chose:** a method that is logically a read may write a field marked
`@cache` through a read `self`: path compression in `Dsu.find`, lazy
tables in `Comb` (decision 145; chapter 5 §10a).

```cp
struct Dsu:
    @cache parent: List<Int>
    pub fn find(self, x: Int) -> Int:
        let p = self.parent[x]
        if p == x: return x
        let r = self.find(p)
        self.parent[x] = r
        r

let d = Dsu(parent: [0, 0, 1])
print(d.find(2), d.parent[2])
```

The compiler checks what keeps it memory-safe: no lend of the field
escapes (C1), only the struct's own methods write it (C2), a cache write
is a `mut` access inside the struct (C3), no unknown calls while the
field is lent (C4), and derived `==`, `hash`, `Ord` and `print` skip it
(C5). The author promises what keeps it correct: cache writes change no
answer (P1). Stress runs check P1 by undoing cache writes at method exit
and comparing output.

**Lost:** `Comb` alone having interior mutability (a stdlib privilege,
against "no privileged stdlib"); Rust's `Cell`/`RefCell` types, which
put the mode in the type; making `find` take `mut self`, which would
forbid `find` on a `let` DSU and inside any read-only walk.

### 12. Three tiers of trust, all greppable

| Tier | Spellings | If broken | Surfaced by |
|---|---|---|---|
| Proved | moves, lends, R1 to R12 | can't be | the compiler |
| Trap | `!`, indexing, `assert`, preconditions such as sortedness | stops with a message | checked builds check every one |
| Promise | `@cache`, `@law` | wrong answers, never undefined behaviour | `cpl explain` lists each; stress runs check them |
| Unsafe | `@trusted:` blocks, `@cpp("...")` bodies | anything | `cpl explain` lists each |

Anything the compiler trusts without proof is an `@` attribute from this
closed list, so `grep -E '@(cache|law|trusted|cpp)'` finds all of it.
Codegen never optimizes on a promise (no `__restrict` from a `disjoint`
law), which is what keeps a broken promise from becoming undefined
behaviour.

## What the compiler proves and what each build checks

| Property | When | Cost on the judge |
|---|---|---|
| No use after move, no write through `let` | compile time (R2, R3) | none |
| No write under a live view, no access under a `mut` lend | compile time (R7, R10) | none |
| No lend outlives its source | compile time (R5, R8) | none |
| Two `mut` places are apart | compile time from facts (R9) | none |
| Index in range, `x!` not `None`, `assert` | checked builds trap (`cpl run`, stress, `-DLOCAL`) | unchecked in release by default (decision 148) |
| `@cache` answers unchanged, `@law`s hold | stress runs | none |

The ownership rules are entirely static: they generate no runtime code.
Lowering is direct: an owned field is a C++ value, a lent field a
reference or span, and each mix of owned and lent fields a program
actually builds is one template instantiation (chapter 5 §13a). Because
the C++ never aliases two `mut` places and never keeps a reference past
its source, the emitted code has no undefined behaviour from aliasing
even when a trap check is compiled out.

## Comparisons

| | Rust | C++ | Swift | CP Lang |
|---|---|---|---|---|
| `let b = a` on a vector | move | deep copy | copy-on-write | move at last use, else error |
| References in types | `&T`, `&mut T`, lifetimes | `T&`, `T*` | none (value types) | none; `lends` on fn signatures only |
| Parameter default | by value (moves) | by value (copies) | borrowed, `inout` for writes | lent, `mut` for writes |
| Write marked at the call | `&mut a` | no | `&a` | `mut a` |
| Aliasing rule | `&mut` XOR `&` | none | runtime exclusivity checks | mutation XOR access, static |
| Two `mut` elements | `split_at_mut` | allowed | `swapAt` | `if i != j` proves it |
| Closure capture | for the closure's life | by `[&]` or `[=]` | escaping vs non-escaping | borrowed per call |
| Interior mutability | `Cell`, `RefCell` | `mutable` | class references | `@cache` field |
| Iterators | pull; lending iterator is hard | pull, invalidation is UB | pull | push generators |

**Rust.** The goals are the same: no undefined behaviour and no aliasing
bugs. CP Lang's lends are second-class (never in a type, never past the
caller's block), so it drops `&`, `&mut`, lifetimes, `derive(Copy)`,
`split_at_mut` and the closure kinds `Fn`/`FnMut`/`FnOnce`. What it gives
up is the same list chapter 5 §13a calls "not expressible, on purpose":
closures that outlive their captures, two owners of one mutable value
(`Rc<RefCell<T>>`), paused generators, callees that take ownership, and
pointers between nodes (use positions in a domain).

**C++.** C++ code translates directly, but its two defaults flip: `=` on
a container moves instead of copying, and a reference that would dangle
or alias a write is a compile error instead of undefined behaviour.
`vector<int>&` parameters become plain parameters, non-const references
become `mut`.

**Swift.** The closest relative, with Hylo. Swift has value semantics
for collections, `inout` parameters marked `&` at the call, non-escaping
closure parameters, and the law of exclusivity. Swift checks
exclusivity partly at runtime and keeps copy-on-write buffers so that
`let b = a` is cheap; CP Lang checks everything statically and makes
`let b = a` a move, so no refcount or uniqueness check is ever emitted.
Swift's `_modify` accessors are CP Lang's `lends mut` projections, with
write-back after the tail expression.

**Hylo and Mojo.** Hylo's mutable value semantics, `inout` and yield
subscripts, and Mojo's `read`/`mut`/`owned` argument conventions, are the
same family; CP Lang adds inference of the mode for local bindings and
views, so the convention appears only on parameters (`mut`) and
signatures (`lends`).

## Sharp edges

These are the places the model surprises, so errors and hovers must
explain them (chapter 5 §13a):

1. `let` copies a plain element, lends a heap element and moves a
   variable. `let row = rows[i]` is a read lend of `rows` (R05.3.5).
2. The mode is not in the type: `xs.filter(p)` is lent, `Filter(xs, p)`
   owned; hover shows which.
3. Passing `mut seen` to a generator freezes `seen` for the value's life;
   capturing it in a nested generator freezes it only per walk.
4. A later use extends a lend, so an added `print` can break a write in
   between.
5. Declaration order matters for R8: a lent value must be declared after
   its sources.
6. A record gaining a heap field stops copying and starts moving.
7. Re-walking a `mut`-lent generator reruns it against changed state
   (legal, linted).

## Worked examples

**Evaluation order makes the common updates legal** (R11): the right
side runs first, then the target's indices, then the write, so each lend
ends before the next starts.

```cp
var a = [1, 2]
var cnt = [0, 0, 0, 0]
a.push(a.len)
cnt[a[0]] += 1
print(a.len, cnt[1])
```

**A named mutable place** (R05.3.4):

```cp
var a = [1, 2, 3]
let p = mut a[1]
p += 10
print(a[1])
```

**Walking keys instead of the list** lets the body push, because `.keys`
is a plain range fixed before the loop:

```cp
var rows = [[1, 2], [3]]
for k in rows[0].keys: rows[0].push(rows[0][k])
print(rows[0].len)
```

**Exercises.** Chapter 5 §14 has 35 snippets with answers; each one is a
conformance case under `spec/conformance/core-05/exercise-*.cp`. Working
through them, using the seven-step procedure in §12, is the fastest way
to learn the model.

# Reference

## Where the precise rules are

| Topic | Spec | Rule ids |
|---|---|---|
| Six-sentence model, tiers, the picture | `spec/core/05-ownership.md` §1 | |
| Plain and heap values | §2 | R05.2.1, R05.2.2 |
| Bindings and places | §3 | R05.3.1 to R05.3.5 |
| Parameters, constructors | §4 | R4 |
| Returns, owned vs lent results | §5 | R5 |
| `lends` fns and projections | §6 | R05.6.1 to R05.6.4 |
| What is lent, liveness, keeping | §7 | R05.7.1 to R05.7.10 |
| Overlap, calls, evaluation order | §8 | R05.8.1 to R05.8.4, R9 to R11 |
| Generators and views | §9 | R05.9.1 to R05.9.4 |
| Closures and nested fns | §10 | R05.10.1 to R05.10.8 |
| `@cache` fields | §10a | R05.10.9 to R05.10.14 |
| Top-level state | §11 | |
| How to evaluate a snippet | §12 | |
| Comparison with Rust, limits, sharp edges | §13a | |
| Value kinds | `spec/core/02-values-types.md` §2.1, §2.12 | R02.1.1 to R02.1.4 |
| Build modes and checks | `spec/core/09-contexts-checks.md` | |

Decisions: 5.1 to 5.14, 126, 128, 141, 144, 145, 148 in `decisions.md`.

## Changes

- 2026-11-23: first version.
