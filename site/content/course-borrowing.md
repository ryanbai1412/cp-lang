Aliasing bugs and accidental O(n) copies cost time in contests: mutating an array through one alias while iterating through another, or copying a 200,000-element vector inside a helper call. CP Lang 0.2.2 prevents both with **lexical borrowing** and **`lends` projections**—with zero lifetime annotations.

## Explicit `mut` at definition and call site

Parameters are read-only borrows by default. To mutate an argument in place, write `mut` both in the function signature and at the call site so every mutating call is visible when reading top-down:

```cp CP Lang 0.2.2 · fragment
fn add_all(mut xs: [V]Int, d: Int):
    for v in V:
        xs[v] += d

add_all(mut a, 1)
```

Small non-heap values (`<= 16` bytes) copy implicitly. Heap containers (`Array`, `List`, `Map`, `Str`) have strict value semantics: moving on last use, cloning explicitly with `.copy()`, swapping in O(1) with `swap(mut a, mut b)`, or moving out with `.take()`. Destructuring `let (r, c) = q[head]` copies the values, so a later `q.push` cannot dangle. Nested functions (`fn dfs(u: V):`) can mutate captured outer `var` bindings directly.

## First-principles `lends` and why no lifetimes exist

A container often wants to grant temporary access to one element or sub-view without copying it—such as `c.at(pos)` or a grid row `g.row(r)`. In CP Lang, a method declares `lends T` or `lends mut T` and ends with the projected place:

```example lends
```

Why does `c.at(pos) += delta` need no `'a` lifetime parameter? Because `lends` is **lexical**, not first-class: the compiler desugars `fn at(mut self, i: V) lends mut Int` into an inlined callback `fn at(mut self, i: V, use: (mut Int) -> R) -> R = use(mut self.cnt[i])`. The caller's operation (`+= delta`) runs inside that callback while `at` is still on the stack, and the borrow ends immediately when the statement finishes.

Because a lend cannot be returned from a function, stored in a `struct`, or assigned into a `var`, it can never outlive its owner. Whenever a helper needs to identify a sub-window for later use, return plain data—an index `V` or a `Range<V>`—and re-project at the call site.

## Disjoint aliasing with `@law disjoint`

What about swapping two elements of the same container, `swap(mut a[i], mut a[j])`, or two projected cells `swap(mut b.at(i), mut b.at(j))`? Both arguments borrow from the same owner.

A library author annotates a projection with `@law disjoint(i, j) = i != j` (or `r1 != r2` for grid rows). When the checker sees `if i != j: swap(mut b.at(i), mut b.at(j))`, the law proves the two mutable lends cannot alias. Two different projections on the same container (e.g., `b.at(i)` and `b.other(j)`) are never assumed disjoint.

> **WASM status.** Custom `lends` / `lends mut` projections and explicit `mut` parameters (`lends.cp` above) compile and run in real WebAssembly today. Static `@law disjoint` proof across two simultaneous `mut` lends of the same user projection is still being wired in `cpl-check` (currently rejects with `E0601`).

## Index before place, right before left

In `a[idx].f = rhs`, `a[idx].f ??= rhs`, and compound `a[idx].f op= rhs`, all index expressions and `rhs` finish before the mutable place in `a` is borrowed. Arena code stays safe even when the right-hand side reallocates the same container:

```cp CP Lang 0.2.2 · spec fragment
trie[o].next[s] ??= trie.push_id(Node())
nodes[u].l = merge(nodes[u].l, b)
```

The emitted C++17 evaluates the index and value first, then writes to the slot, so `std::vector` growth cannot leave a dangling reference.

## Practice

```exercise borrow-explicit-mut
```

```exercise borrow-inline-callback
```

```exercise borrow-disjoint
```

```exercise borrow-escape
```

```exercise borrow-lends-output
```

## Write a program

```exercise borrow-write-counter
```
