Almost every range data structure in CP is the same trick: cut a sequence into pieces, summarize each piece, and combine summaries. This chapter names the rules that make the trick correct (associativity, identity, inverse) and shows how CP Lang turns them into ordinary values, so one generic segment tree serves sum, max, and your own custom combine.

> **Status.** Operation values such as `Monoid<T>`, `Sum`, and `Max`, their `law` declarations, and the generic `SegTree` are approved design from [plan/13-abstraction.md](https://github.com/ryanbai1412/cp-lang/blob/main/plan/13-abstraction.md); those blocks are marked *design fragment*. The prototype already runs structs, methods, generic functions over domains such as `fn f<D>(xs: [D]T)`, and the `@invariant(rev)` and `@invariant(shuffle)` property checks; blocks marked *prototype-tested* were compiled and run. Function-typed struct fields, the library `Monoid` type, and `cpl test`/`cpl doc` for libraries are not implemented yet. The [standard library reference](stdlib.html) lists the planned API.

## A fold is a chain of combines

`sum(a)` looks like a builtin, but it is a fold: start from a neutral value and combine every element into it.

```example sum
```

Written out, `sum` over `5 -2 4` is `((0 + 5) + -2) + 4`. Two facts about `+` make this both correct and fast:

- **Identity.** `0 + x == x`, so the fold can start from `0`, and the sum of an empty range is `0` rather than a special case.
- **Associativity.** `(x + y) + z == x + (y + z)`, so the parentheses can move. A segment tree answers `l..=r` by combining about `2 log n` precomputed pieces whose grouping has nothing to do with the left-to-right loop, and it is still correct.

An operation with both properties is a **monoid**. Add an inverse (`-x` for `+`) and it is a **group**. These names matter because each data structure needs a specific one.

```exercise algebra-sum-identity
```

## Associativity is what lets a structure regroup

Take "average of two numbers" as the combine. It looks harmless, but it is not associative:

```text
avg(avg(2, 4), 6) = avg(3, 6) = 4.5
avg(2, avg(4, 6)) = avg(2, 5) = 3.5
```

A segment tree built on `avg` returns different answers depending on how the query range happens to split across nodes. The standard fix is to change what a piece *summarizes*: keep `(sum, count)` per piece. Combining `(s1, c1)` with `(s2, c2)` gives `(s1 + s2, c1 + c2)`, which is associative with identity `(0, 0)`, and the average is `sum / count` at the very end.

That pattern recurs constantly. "Maximum subarray sum in a range" is not a monoid on single numbers, but it is a monoid on `(sum, best_prefix, best_suffix, best)`. When a combine feels wrong, ask what extra fields would make it associative.

```exercise algebra-assoc
```

## Identity and inverse decide which structure fits

The identity is the value of an empty piece, and each operation has its own:

- `Sum<Int>` has identity `0`.
- `Max<Int>` has identity `Int.MIN`, and `Min<Int>` has `Int.MAX`.
- `gcd` has identity `0`, because `gcd(0, x) == x`.

An inverse lets you *remove* a piece, which is what prefix differences do. With `pre` from the [bounds example](course-borrowing.html), the sum of `l..=r` is `pre[r.r] - pre[l.l]`: the prefix up to `r`, with the prefix before `l` taken back out. That only works because `+` has `-`. Maximum has no inverse: knowing the maximum of `1..=r` and of `1..=l-1` tells you nothing about the maximum of `l..=r`.

The approved stdlib turns this into types instead of folklore. `Fenwick` takes a `Group`, `SegTree` takes a `Monoid`, and `SparseTable` takes an idempotent `Semigroup`, so "Fenwick tree for range maximum" is a compile error rather than a wrong answer on test 37.

```exercise algebra-inverse
```

## Order is part of the meaning

Many useful combines are associative but **not commutative**: swapping the operands changes the result. Function composition is the classic example. An affine map `x => mul * x + add` followed by another affine map is again affine, and this runs in the prototype today:

```cp CP Lang · prototype-tested
let n: 1..=9 = read()
domain V = 1..=n

struct Aff:
    mul: Int
    add: Int

// then(f, g) applies f first, then g
fn then(f: Aff, g: Aff) -> Aff:
    Aff(f.mul * g.mul, f.add * g.mul + g.add)

fn compose_all<D>(fs: [D]Aff) -> Aff:
    var acc = Aff(1, 0)               // the identity map x => x
    for d in D:
        acc = then(acc, fs[d])
    acc

let fs: [V]Aff = read()
let all = compose_all(fs)
print(all.mul, all.add)
```

With input `3`, then `2 1`, `3 0`, `1 5`, it prints `6 8`: first `x => 2x + 1`, then times 3, then plus 5, which is `x => 6x + 8`. `compose_all` is generic over the domain `D`, so the same code folds an array indexed by vertices, positions, or queries.

`then` is associative, so a segment tree may regroup it, but it must keep the **left-to-right order** of the pieces. A segment tree written for commutative operations often combines the right-hand accumulator in the wrong order; the approved `SegTree.fold` keeps two accumulators (`a` from the left, `b` from the right) and finishes with `op(a, b)` precisely so that non-commutative monoids work. Path queries on trees go further: `Hld.path` asks for `Monoid.both(m, m.flip)` because one half of a path is walked upward, instead of silently assuming commutativity.

You can ask the prototype whether a function secretly depends on order. `@invariant(rev)` reruns a function on reversed input and reports the first counterexample:

```cp CP Lang · prototype-tested
@invariant(rev)
fn apply_all(xs: [V]-5..=5) -> Int:
    var x = 0
    for v in V:
        x = x * 2 + xs[v]
    x
```

```text
$ python3 proto/cp.py fntest affine.cp
PROPERTY FAILS: apply_all is not invariant under rev
after rev: xs=-5 -4 2 0 -2 1
before: -5
after: -211
```

That is the correct verdict: the binary-number fold is order-dependent. Use `@invariant(rev)` or `@invariant(shuffle)` when you *believe* a combine is commutative and want the tool to argue with you.

```exercise algebra-noncommutative
```

## Operation values instead of one trait per behavior

A **trait** describes what a type canonically supports: `Int` has `Ord` and `Add`. But `Int` has many useful monoids (sum, max, min, xor, gcd), and a trait can only pick one. So the approved design passes behavior as an ordinary **operation value**, a struct of an identity and a function, with its laws written next to it:

```cp Library contract · design fragment
pub struct Monoid<T>(id: T, op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a

const Sum<T> = Group<T>(id: T.zero, op: (a, b) => a + b, inv: a => -a) where T: Add
const Max<T> = Monoid<T>(id: T.MIN, op: max) where T: Ord + Bounded
```

Traits still matter: `Sum<T>` is *built from* `T: Add`, and an `Order<T>` parameter defaults to `T`'s `Ord`. Traits give the canonical operation; operation values name the others.

Using one is just passing a value. Your own monoid is two lines:

```cp CP Lang · design fragment
use std.segtree.*

struct Seg(len: Int, best: Int)
const Up = Monoid<Seg>(id: Seg(0, inf),
    op: (lo, hi) => Seg(lo.len + hi.len, min(lo.best, lo.len + hi.best)))

var best = SegTree<V, Int>(Max<Int>)
var path = SegTree<V, Seg>(Up)
```

Two rules keep this as fast as hand-written C++:

- **Static dispatch is guaranteed.** Every function has its own compile-time identity, so `SegTree(Max<Int>)` is compiled once with `max` inlined, and `SegTree(Sum<Int>)` gets its own copy. Choosing the code at runtime (`if k: Max<Int> else: Min<Int>`) is a compile error whose help says to branch *outside* the structure, unless you opt into an indirect call with `dyn`.
- **Laws cost nothing in the submission.** `cpl fntest` checks every law of every operation value on generated values and shrinks counterexamples; debug runs sample laws on values that really occur; release builds strip them.

```exercise algebra-runtime-op
```

## The standard library is userland code

Nothing above is compiler magic. The approved `SegTree` is a page of ordinary CP Lang that uses generics over an `Index`, an operation value, and the domain size `D.K`:

```cp Library implementation · design fragment
pub struct SegTree<D, T>(m: Monoid<T>):
    t: List<T>
    size: Int

    pub fn new(m: Monoid<T>) -> SegTree<D, T>:
        let size = bit_ceil(D.K)
        SegTree(m, List.filled(2 * size, m.id), size)

    pub fn set(mut self, i: D, x: T):
        var p = i.to_idx + size
        t[p] = x
        while p > 1:
            p /= 2
            t[p] = m.op(t[2 * p], t[2 * p + 1])
```

Note what it does *not* do: it never returns a reference into `t`. Reads return values, and anything that must expose a place inside the tree would use a `lends` projection from the [borrowing chapter](course-borrowing.html). That keeps library code under the same rules as your solution.

Your own library works the same way. The planned layout is a folder configured in `cp.toml`:

```cp CP Lang · design fragment
use std.segtree                  // the standard library
use lib.treap                    // your library folder, from cp.toml
use lib.graph.flow as fl
```

Everything used is inlined into the single submitted C++ file and tree-shaken, so a large personal library does not bloat a submission. The planned `cpl test` and `cpl doc` commands work on standard-library and local-library modules alike, so a library's `@brute` checks and laws run the same way as a solution's.

```exercise algebra-local-library
```

## Common mistakes

- **Using a non-associative combine.** Pairwise averages and `a - b` break regrouping. Carry more fields until the combine is associative.
- **Choosing the wrong identity.** `0` is not the identity of `Max` when values can be negative; use `Int.MIN`, or wrap values in `?T` with `none` as the identity.
- **Assuming commutativity.** Composition, matrix products, and string concatenation care about order. Fold left to right, and use `@invariant(rev)` when unsure.
- **Prefix differences on a monoid.** Subtraction needs an inverse. Range max needs a segment tree or sparse table, not two prefix lookups.

Next, the [verification chapter](course-verification.html) shows how to test recurrences, multi-answer problems, and interactive protocols against slow references.
