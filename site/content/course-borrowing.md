Aliasing bugs and hidden copies cost time in contests: mutating an array through one alias while iterating through another, or accidentally copying a 200,000-element array inside a helper call. CP Lang v2 prevents both with lexical borrowing and explicit `mut`—with zero lifetime annotations.

## Explicit `mut` at definition and call site

Parameters are read-only borrows by default. To mutate an argument in place, write `mut` in the function signature and at the call site:

```cp CP Lang v2
fn bump(mut a: [V]Int, v: V):
    a[v] += 1

bump(mut a, V.first)
```

Small values (`<= 16` bytes, no heap) are `Copy`. Heap containers (`Array`, `List`, `Map`, `Str`) have strict value semantics: moving on last use, or cloning explicitly with `.copy()`. Plain nested functions (`fn dfs(u: V):`) can mutate captured outer `var` bindings directly without extra annotations.

## Lexical borrows and `lends`

A container can grant temporary access to part of its storage—such as a slice `a[l..=r]` or grid row `g.row(r)`—using `lends` and `lends mut` projections. Those projections desugar to inline scopes and cannot be stored in structs or returned out of a function, so you never write lifetime parameters like `'a`.

## Disjoint aliasing with `law disjoint`

What about swapping two elements of the same array, `swap(mut a[i], mut a[j])`, or swapping two grid rows `swap(mut g.row(r1), mut g.row(r2))`? Both arguments come from the same container. Library authors declare a `law disjoint` check (`i != j`, `r1 != r2`) so the compiler verifies disjointness statically when it can and inserts a debug-mode check otherwise.

## Practice

```exercise borrow-explicit-mut
```

```exercise borrow-inline-callback
```

```exercise borrow-disjoint
```

```exercise borrow-escape
```

```exercise borrow-bounds-output
```
