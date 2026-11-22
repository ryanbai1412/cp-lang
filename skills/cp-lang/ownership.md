# Ownership, Lending, Moves, and `@cache`

CP Lang prevents hidden O(n) copies and aliasing bugs without lifetime
annotations.

## Plain Values vs Heap Values

- **Plain values** (`Int`, `Float`, `Bool`, `Char`, domain indices, small
  tuples/structs of plain values) copy implicitly.
- **Heap values** (`List`, `[D]T`, `Str`, `Set`, `Map`, `Graph`, `SegTree`,
  structs containing heap fields) **move** on assignment. Using a moved
  variable is a compile error (`E0608`). Use `.clone()` for an explicit copy:

```cp
let a = [1, 2, 3]
let b = a.clone()
print(a.sum(), b.sum())
// output: 6 6
```

## Function Parameters Lend by Default

Passing a heap value to a function borrows (lends) it for the call without
moving or copying:

```cp
fn total(xs: List<Int>) -> Int:
    xs.sum()

let a = [10, 20, 30]
print(total(a), a.len)
// output: 60 3
```

## Mutation Requires `mut` in Both Places

To mutate a parameter, mark it `mut` in the signature **and** at every call
site. Missing `mut` at the call site is `E0402`; assigning to a `let` binding
is `E0401`.

```cp
fn bump(mut xs: List<Int>, delta: Int):
    for i in 0..xs.len:
        xs[i] += delta

var a = [1, 2, 3]
bump(mut a, 10)
print(a)
// output: 11 12 13
```

```cp error(E0401)
let x = 1
x = 2
```

```cp error(E0402)
fn inc(mut xs: List<Int>):
    xs.push(1)

var a = [0]
inc(a)
```

## Avoiding Borrow Conflicts (`E0601`, `E0602`)

Views (`a[l..r]`, `g[u]`) and generators borrow their source until their last
use. Finish reading a view before mutating the container, or copy out the
scalar/cloned value you need first:

```cp
var a = [3, 1, 4]
let n = a.len
a.push(n)
print(a)
// output: 3 1 4 3
```

When you need to mutate a container while iterating over a snapshot of its
current elements, clone or `.collect()` the snapshot first.

## `@cache` on Struct Fields

A method that is logically a read may write a field marked `@cache` through
a read `self`: path compression in `Dsu.find`, lazily grown tables in `Comb`,
tag pushes in a lazy segment tree query. The field must not change any answer:
`==`, hashing and `print` skip it, and `cpl stress` reruns with caches cold
and compares. No lend of the field escapes. This is for library authors; in a
solution you just call `d.find(v)` on a `let` value. `@cache` is not
memoization: for recursive DP use `@memo(array)` or `@memo(map)` (see
`functions-and-generators.md`).
