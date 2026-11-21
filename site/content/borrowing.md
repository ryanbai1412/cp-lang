Variables own their values. Function parameters borrow them for the call. A projection lets a caller work on part of a value—a row, a slice, or an element—without allocating a copy or keeping a reference for later.

> **Implementation status.** Struct projections with `lends` / `lends mut` and explicit `mut` parameters run directly in `cpl` and the browser WebAssembly playground (try the `lends` example in the [playground](playground.html)). Custom `@law disjoint` proofs and generic library projections are part of the 0.2.2 specification and are rolling out incrementally across the compiler.

## Own it, borrow it, or copy it

```cp CP Lang 0.2.2
domain V = 1..=3
var a: [V]Int = 2

fn total(a: [V]Int) -> Int:
    a.sum()

fn bump(mut a: [V]Int, v: V):
    a[v] += 1

print(total(a))         // read-only borrow, no array copy
bump(mut a, V.first)   // mutable borrow, explicit at both ends
let saved = a.copy()   // an owned copy, explicit cost
```

While a place is borrowed mutably, other code cannot read or write the same place. Multiple read-only borrows are fine. Separate struct fields can be borrowed separately; overlapping container accesses need more evidence.

## A projection lends a place

An ordinary function uses `->` to return an owned value. A projection uses `lends` to give temporary access to a place in its receiver, and the tail expression of its body names that place (`mut place` for a mutable one). `lends` is one of only four library-author keywords; contestants use projections, they rarely write them.

```cp CP Lang 0.2.2
domain V = 1..=3
struct Counter:
    data: [V]Int
    fn at(mut self, i: V) lends mut Int:
        mut self.data[i]

var counter = Counter([V] v => 0)
counter.at(V.first) += 7
print(counter.data)
```

This projection lends one counter without copying it. `lends T` only permits reading. `lends mut T` permits mutation when the caller holds the receiver mutably and the use asks for mutation. It does not turn a read-only receiver into a mutable one.

```cp CP Lang 0.2.2
struct Value:
    data: Int
    fn get(self) lends Int:
        self.data

let owner = Value(7)
let lent = owner.get()
print(lent)
```

The local `lent` is temporary access to `owner`, not an independent owner. A mutating projection use such as `counter.at(i) += 1` holds its mutable lend for that statement. Generic grid-row lends and in-place slice sorting remain specification work; they are not available in the current execution backend.

## Why there are no lifetime annotations

CP Lang does not ask you to name how long references live. It restricts where a borrow can go instead. A lent value may be a local `let`, an argument, a use inside one expression, or a loop's temporary value. It cannot escape into an owning field, a container, a `var`, a stored closure, or an ordinary return value.

The implementation model is an inlined callback (there is no `inline` keyword; `lends`, `yield`, and closure arguments carry that contract): a projection runs the caller's use of the lent place while the owner is still available. Binding a lend to `let` puts the rest of that block inside the callback. The checker can see the whole use; there is no returned reference whose lifetime must be related to some other reference.

That is the tradeoff, not a promise that arbitrary Rust borrowing works without annotations. Structures that must remember identity use plain indices or `List.push_id` handles. Functions that find a useful sub-range return a `Range<V>`, then the caller reopens `a[span]` when needed. If you truly want independent storage, ask for `.copy()`.

```cp CP Lang 0.2.2
domain V = 1..=4
let a: [V]Int = [V] v => v.to_idx + 1
fn find_span(a: [V]Int) -> Range<V>:
    V.first..=V.last

let span: Range<V> = find_span(a)      // plain data: two boundaries
let part = a[span]                     // a lexical lend, used here
print(part.sum())
let saved = part.copy()                // owned data may escape
```

`find_span` above is an ordinary user helper returning a range. It must not return a borrowed slice. Returning the recipe for access is simpler than returning access itself.

## Prove two accesses do not overlap

The array's element projection declares this law:

```cp CP Lang 0.2.2
domain V = 1..=3
struct Counter:
    data: [V]Int
    fn at(mut self, i: V) lends mut Int:
        @law disjoint(i, j) = i != j
        mut self.data[i]

var counter = Counter([V] v => 0)
counter.at(V.last) += 1
print(counter.data)
```

A trivial single-index forwarding projection like this one gets the `i != j` law automatically; it is written out here to show the contract. The law says when two lends through that same projection are separate. It does not assert that every call is safe. The checker must prove the condition at the call site.

```cp CP Lang 0.2.2
domain V = 1..=3
var counts: [V]Int = 1
let (i, j) = (V.first, V.last)
fn merge_into(mut dst: Int, src: Int):
    dst += src

if i != j:
    merge_into(mut counts[i], counts[j])
```

Without the guard or another proof of `i != j`, the two elements might alias, so a mutable access and a read cannot coexist. An explicit `assert i != j` is another way to establish the condition. Index arguments are evaluated before lending begins, so the proof is about fixed argument values, not indices that might change halfway through the call.

For two ranges, the specified disjointness condition is that one ends before the other begins: `s.end <= t.start or t.end <= s.start`. Simultaneous mutable split-slice lends are not yet supported. Use independent owned workspaces today:

```cp CP Lang 0.2.2
let a = [4, 3, 2, 1]
let mid = a.len / 2
var left: List<Int> = []
var right: List<Int> = []
for i in 0..mid: left.push(a[i])
for i in mid..a.len: right.push(a[i])

fn solve(mut left: List<Int>, mut right: List<Int>):
    left.sort()
    right.sort()

solve(mut left, mut right)
print(left, right)
```

The law applies to two calls of the **same projection** on the **same receiver**. Different operations do not automatically become disjoint: a row and a column may cross even when their arguments look different.

## Keep the owner obvious

A lent row cannot be saved as the “best row so far” in a variable outside the loop. In the specified grid API, save its index and reopen the row later, or keep a deliberate copy. Generic row lends are not yet runnable; direct `grid[r, c]` access works today. A generator-lent value lasts for one iteration, not forever. These restrictions keep both mutation and performance visible at the use site.

There are no user-visible pointer arithmetic tricks here. The intended primitive is a checked `Buffer.part`; arrays, slices, grids, and strings build their projections from it. An ordinary library can provide the same borrowing behavior as the standard one.
