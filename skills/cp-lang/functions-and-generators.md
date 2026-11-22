# Functions, Generators, and Memoization

## Functions and Closures

Functions use `fn`, indentation blocks, and implicit return of the last
expression (or explicit `return`). Single-expression functions can use `=`:

```cp
fn sq(x: Int) -> Int = x * x

fn clamp(x: Int, lo: Int, hi: Int) -> Int:
    if x < lo: return lo
    if x > hi: return hi
    x

print(sq(5), clamp(12, 0, 10))
// output: 25 10
```

Closures use `=>`: `x => x * 2`, `(a, b) => a + b`.

## Generators (`-> yields T`)

A generator pushes values with `yield` and composes lazily with `.map(...)`,
`.filter(...)`, `.sum()`, `.min()`, `.max()`, `.first`, and `for x in ...`
without allocating an intermediate `List`:

```cp
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0:
            yield i

print(evens(8).sum(), evens(8).collect())
// output: 12 0 2 4 6
```

Use `.collect()` when you actually need a materialized `List<T>`.

## O(1) Getters vs Work Methods

Parentheses signal work:

- **No `()`** (`@get`): O(1) field-like properties — `a.len`, `a.is_empty`,
  `a.first`, `a.last`, `st.all`, `d.components`.
- **With `()`**: O(n) scans, allocations, or mutating calls — `a.sum()`,
  `a.sorted()`, `a.collect()`, `a.clone()`, `h.pop()`.

Calling a getter with `()` or omitting `()` on a method triggers `L0608`.

## Memoization (`@memo`)

### `@memo(array)` for Bounded Keys

When every parameter is a domain or bounded range, `@memo(array)` backs the
cache with a flat array (zero hashing overhead):

```cp
domain I = 0..=10

@memo(array)
fn fib(i: I) -> Int:
    let k = Int(i)
    if k <= 1: return k
    fib(I(k - 1)) + fib(I(k - 2))

print(fib(I(10)))
// output: 55
```

### `@memo(map)` for Sparse or Unbounded Keys

When the key space is large or sparse (e.g., `Int` states reached by division),
use `@memo(map)`:

```cp
@memo(map)
fn ways(n: Int) -> Int:
    if n <= 1: return 1
    ways(n / 2) + ways(n / 3)

print(ways(12))
// output: 7
```
