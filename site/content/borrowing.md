Variables own their values. Function parameters borrow them for the call. A projection lets a caller work on part of a value—a row, a slice, or an element—without allocating a copy or keeping a reference for later.

> **Approved design, partially prototyped.** The Python prototype implements struct projections with `lends` and a disjointness law (spelled `lend` and `law disjoint` there; v2.2 drops the `lend` keyword and writes the law as the `@law` decorator), plus lexical checks for slices and rows. Generic library implementations and the full checker remain work in progress. The fragments below explain the design; [runnable projection tests](https://github.com/ryanbai1412/cp-lang/blob/main/proto/tests/lend/projection.cp) show the supported subset and the [implementation notes](https://github.com/ryanbai1412/cp-lang/blob/main/plan/19-index-and-lends.md#11-what-the-prototype-does-today) state its limits.

## Own it, borrow it, or copy it

```cp CP Lang v2.2
fn total(a: [V]Int) -> Int:
    sum(a)

fn bump(mut a: [V]Int, v: V):
    a[v] += 1

print(total(a))         // read-only borrow, no array copy
bump(mut a, V.first)   // mutable borrow, explicit at both ends
let saved = a.copy()   // an owned copy, explicit cost
```

While a place is borrowed mutably, other code cannot read or write the same place. Multiple read-only borrows are fine. Separate struct fields can be borrowed separately; overlapping container accesses need more evidence.

## A projection lends a place

An ordinary function uses `->` to return an owned value. A projection uses `lends` to give temporary access to a place in its receiver, and the tail expression of its body names that place (`mut place` for a mutable one). `lends` is one of only four library-author keywords; contestants use projections, they rarely write them.

```cp Library implementation · CP Lang v2.2
impl Array<(A, B), T>:
    pub fn row(self, r: A) lends mut [B]T:
        @law disjoint(r, q) = r != q
        mut Array<B, T>(buf.part(r.to_idx * B.K, B.K))
```

This library fragment assumes the array's private buffer `buf` and index types `A` and `B`. It lends a contiguous row; it does not copy the row's elements. `lends T` only permits reading. `lends mut T` permits mutation when the caller holds the receiver mutably and the use asks for mutation. It does not turn a read-only receiver into a mutable one.

```cp CP Lang v2.2
let row = grid.row(r)
print(sum(row))
```

The local `row` is temporary access to `grid`, not an owner. A mutating use such as `sort(mut grid.row(r))` holds the corresponding mutable lend for that statement.

## Why there are no lifetime annotations

CP Lang does not ask you to name how long references live. It restricts where a borrow can go instead. A lent value may be a local `let`, an argument, a use inside one expression, or a loop's temporary value. It cannot escape into an owning field, a container, a `var`, a stored closure, or an ordinary return value.

The implementation model is an inlined callback (there is no `inline` keyword; `lends`, `yield`, and closure arguments carry that contract): a projection runs the caller's use of the lent place while the owner is still available. Binding a lend to `let` puts the rest of that block inside the callback. The checker can see the whole use; there is no returned reference whose lifetime must be related to some other reference.

That is the tradeoff, not a promise that arbitrary Rust borrowing works without annotations. Structures that must remember identity use plain indices or `List.push_id` handles. Functions that find a useful sub-range return a `Range<V>`, then the caller reopens `a[span]` when needed. If you truly want independent storage, ask for `.copy()`.

```cp CP Lang v2.2
let span: Range<V> = find_span(a)      // plain data: two boundaries
let part = a[span]                     // a lexical lend, used here
print(sum(part))
let saved = part.copy()                // owned data may escape
```

`find_span` above is an ordinary user helper returning a range. It must not return a borrowed slice. Returning the recipe for access is simpler than returning access itself.

## Prove two accesses do not overlap

The array's element projection declares this law:

```cp Library contract · CP Lang v2.2
fn at(self, i: I) lends mut T:
    @law disjoint(i, j) = i != j
    mut self.data[i.to_idx]
```

A trivial single-index forwarding projection like this one gets the `i != j` law automatically; it is written out here to show the contract. The law says when two lends through that same projection are separate. It does not assert that every call is safe. The checker must prove the condition at the call site.

```cp CP Lang v2.2
if i != j:
    merge_into(mut counts[i], counts[j])
```

Without the guard or another proof of `i != j`, the two elements might alias, so a mutable access and a read cannot coexist. An explicit `assert i != j` is another way to establish the condition. Index arguments are evaluated before lending begins, so the proof is about fixed argument values, not indices that might change halfway through the call.

For two ranges, the law is that one ends before the other begins: `s.hi <= t.lo or t.hi <= s.lo`. Adjacent halves can therefore be lent separately:

```cp CP Lang v2.2
solve(mut a[l.l..m.l], mut a[m.l..r.r])
```

The law applies to two calls of the **same projection** on the **same receiver**. Different operations do not automatically become disjoint: a row and a column may cross even when their arguments look different.

## Keep the owner obvious

A lent row cannot be saved as the “best row so far” in a variable outside the loop. Save its index, then call `grid.row(best_r)` later, or keep a deliberate copy. A generator-lent value lasts for one iteration, not forever. These restrictions keep both mutation and performance visible at the use site.

There are no user-visible pointer arithmetic tricks here. The intended primitive is a checked `Buffer.part`; arrays, slices, grids, and strings build their projections from it. An ordinary library can provide the same borrowing behavior as the standard one.
