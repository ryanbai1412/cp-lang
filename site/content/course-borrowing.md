This chapter builds borrowing from one question: who may touch this value right now? CP Lang answers it with plain rules about blocks and calls, not with lifetime annotations. By the end you will be able to lend part of a structure, lend two parts at once, and keep a result that outlives a borrow without fighting the checker.

> **Status.** The rules here are approved design. The Python prototype implements struct projections with `lends`, `lend`, and `law disjoint`, plus lexical checks for slices and rows; blocks marked *prototype-tested* were compiled and run with `proto/cp.py`, and the error messages are copied from it. Blocks marked *design fragment* show the approved shape that the prototype does not run yet. The generic `Array`/`Slice` library code and dynamic checking of wrong `disjoint` laws are still missing; the [implementation notes](https://github.com/ryanbai1412/cp-lang/blob/main/plan/19-index-and-lends.md#11-what-the-prototype-does-today) list the limits. The [borrowing reference](borrowing.html) is the short version of this chapter.

## Start with ownership

Every value has exactly one owner: a `let` or `var` binding, a struct field, or a slot in a container. When a function needs a value, it does not get a copy and it does not become the owner. It **borrows** the value for the duration of the call.

```cp CP Lang · prototype-tested
let n: 1..=9 = read()
domain V = 1..=n
var a: [V]Int = read()

fn total(xs: [V]Int) -> Int:
    sum(xs)

fn bump(mut xs: [V]Int, by: Int):
    for v in xs.positions: xs[v] += by

print(total(a))        // read-only borrow: no copy of the array
bump(mut a, 10)        // mutable borrow, written at both ends
print(a)
var keep = a.copy()    // an owned copy: the cost is written out
```

Three things are worth noticing.

- **A read-only borrow is the default.** `total(a)` passes the whole array without copying it, and `total` cannot change it. Many read-only borrows of the same value may exist at once.
- **A mutable borrow is spelled twice.** The parameter says `mut xs`, and the call says `bump(mut a, 10)`. When you skim a solution at 2 a.m., every line that can change `a` contains `mut a`. There is no hidden reference that might be aliased somewhere else.
- **Copies are explicit.** `.copy()` is the only way to get a second owned array, so an accidental O(n) copy inside a loop is visible in the source.

While a value is borrowed mutably, nobody else may read or write it. That single rule is what lets the C++ backend use plain references and pointers with no runtime bookkeeping.

```exercise borrow-explicit-mut
```

## A borrow lives in a block

A slice such as `a[l..=r]` does not copy anything. It **lends** a window of `a`. You can bind it with `let`, pass it, loop over it, and lend it onward, but only inside the block that made it. The borrow ends at its last use.

```cp CP Lang · prototype-tested
let l = V(2)
let r = V(4)
a[l..=r].sort()                 // sort one window in place

let part = a[l..=r]             // `a` is lent to `part` from here...
var best = l
for v in part.positions:
    if part[v] > part[best]: best = v
print(best, a[best], sum(part)) // ...to its last use here
a[best] = 0                     // fine: `part` is no longer used
```

Positions inside a slice are ordinary `V` values, so `best` is a real index into `a` and stays valid after `part` is gone. If you write to `a` while `part` is still going to be used, the prototype stops you and says exactly which borrow is in the way:

```text
error: line 8: `a[l]` can't be changed here: `a` is lent to `part` (line 7)
  until its last use on line 9
  help: finish with `part` first, go through `part`, or keep the position and
  re-project later; `.copy()` keeps an owned copy
```

The fix is almost always to reorder: finish reading `part`, then write. The checker does not need lifetimes because the borrow cannot leave the block; it only needs to know where the block's last use is.

## Projections are inline callbacks

A struct can expose a place inside itself with a **projection**. The return type says `lends` (or `lends mut`), and the body ends with `lend` instead of returning a value.

```cp CP Lang · prototype-tested
let n: 2..=9 = read()
domain V = 1..=n

struct Bag:
    cnt: [V]Int

    fn at(self, i: V) lends mut Int:
        law disjoint(i, j) = i != j
        lend mut cnt[i]

fn move(mut from: Int, mut to: Int):
    to += from
    from = 0

let zero: [V]Int = 0
var b = Bag(zero)
for v in V: b.at(v) = v.label     // write through the projection
let i = V.first
let j = V.last
if i != j:
    move(mut b.at(i), mut b.at(j))
print(b.cnt)
```

With input `4` this prints `0 2 3 5`. The bag starts from the labels `1 2 3 4`, and `move` shifts the first count onto the last.

The key idea is what a projection *means*. It is not a function that returns a reference. It is a function that takes the rest of your expression as an **inline callback** and runs it while the place is still inside its own frame:

```cp CP Lang · design fragment
// what `fn at(self, i: V) lends mut Int` means
fn at(mut self, i: V, inline use: (mut Int) -> R) -> R =
    use(mut cnt[i])

// so `b.at(v) += 5` is
b.at(v, x => x += 5)
```

Because the callback runs *inside* `at`, there is nothing to return and nothing that could dangle. The callback is `inline`, so the C++ is the same as writing `b.cnt[v] += 5` by hand: no closure object, no indirect call. This one desugaring is why CP Lang needs no lifetime parameters: a borrow is always shorter than the call that made it, by construction.

```exercise borrow-inline-callback
```

## Two mutable lends at once: `law disjoint`

`move(mut b.at(i), mut b.at(j))` lends two cells of the same bag mutably at the same moment. That is only safe when they are different cells. The projection states when two of its lends do not overlap:

```cp
law disjoint(i, j) = i != j
```

At each call that lends the same projection twice, the checker must *prove* the law from facts it can see: an enclosing `if i != j`, an `assert i != j`, or range facts such as `r < l2`. Delete the `if` from the program above and the prototype says:

```text
error: line 21: `b.at(i)` and `b.at(j)` may be the same storage, and both are
  lent `mut` to this call: Bag.at lends disjoint parts only when `(i != j)`,
  and that is not known here
  help: guard it (`if i != j: ...`), state it (`assert i != j`), or copy the
  read-only one first
```

Arrays come with the same laws built in. `swap(mut a[l], mut a[r])` needs `l != r` to be known, and two slices may be lent mutably together when their ranges cannot overlap:

```cp CP Lang · prototype-tested
fn mix(mut xs: [V]Int, mut ys: [V]Int):
    for v in xs.positions: xs[v] += 1
    for v in ys.positions: ys[v] += 2

let m = V(3)
mix(mut a[l.l..m.l], mut a[m.l..r.r])   // [l, m) and [m, r] cannot overlap
```

Two things the law does **not** give you:

- **Different projections are never assumed disjoint.** If `Bag` also had `fn other(self, i: V) lends mut Int` that lends `cnt[i]`, then `move(mut b.at(i), mut b.other(j))` is rejected even when `i != j`, because nothing relates one projection's storage to the other's. Separate *fields* are separate storage; separate *projections* are not.
- **A law is a promise you make.** If you write `law disjoint(i, j) = true` on a projection that can lend the same cell twice, the checker trusts you. The prototype does not yet check wrong laws at runtime; the approved design checks them in debug builds.

```exercise borrow-disjoint
```

## Nothing escapes: return positions, not borrows

A lent value is second-class. It cannot be:

- returned from a function,
- assigned to a `var`,
- stored in a struct field or pushed into a `List`,
- captured by a closure that is not `inline`.

Each of these would let the borrow outlive its block, and the prototype names the exact move that would do it:

```text
error: line 7: `var keep` would keep the lent slice `a[l..=r]` after this block;
  a lent value lives only in the block that made it
  help: keep the range or the row index (a plain value) and re-project where it
  is used, or keep a copy: `a[l..=r].copy()`
```

The help text is the whole technique. A window is a **value**; a slice is a **borrow**. Functions compute and return windows, and the caller applies them to the array it owns:

```cp CP Lang · prototype-tested
fn widest(lo: V, hi: V) -> Range<V>:
    lo..=hi

let w = widest(V.first, V(3))   // a plain value: store it, return it, push it
a[w].rev()                      // re-project at the point of use
```

This is also how CP code usually wants to be written. A function that "finds the best segment" returns the segment's bounds; the caller decides whether to sort it, reverse it, or sum it. If you genuinely need the data after the array changes, take `.copy()` and pay for it in plain sight.

```exercise borrow-escape
```

## Boundaries are values too

The boundary shorthands `v.l` and `v.r` from the [indices chapter](indices.html) fit the same picture. `v.l` is the boundary just before `v` and `v.r` the one just after it; they are values of `V.bounds`, not borrows. That is why a prefix-sum table indexed by boundaries needs no `+ 1` arithmetic and no borrowing at all:

```example bounds
```

`pre[v.r] = pre[v.l] + a[v]` reads as "the prefix ending after `v` is the prefix ending before `v`, plus `a[v]`". The last boundary is `V.last.r`, so the final print is the total.

```exercise borrow-bounds-output
```

## Common mistakes

- **Keeping a slice in a `var`.** Keep the `Range<V>` (or the row index) instead, or call `.copy()`. A `let` slice is fine because it ends at its last use.
- **Forgetting `mut` at the call.** `bump(a, 1)` is rejected when the parameter is `mut`. The duplication is deliberate: the call site must show the mutation.
- **Writing while a `let` slice is still used.** Move the write after the slice's last use, write *through* the slice, or remember the position and re-project.
- **Expecting two projections to be disjoint.** Only the same projection, with a provable `law disjoint`, may be lent mutably twice. Otherwise borrow separate fields, or copy the read-only side first.
- **Proving the law too late.** `assert i != j` must come *before* the call; the checker reads facts in order, like a reader would.

Next, the [algebra chapter](course-algebra.html) uses these borrowing rules to build generic data structures that never hand out references.
