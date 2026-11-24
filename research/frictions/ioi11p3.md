# ioi11p3: Rice Hub

## Dream

```cp
let x: sorted [Field]1..=len = read()
print(Field.longest_window((l, r) => cost(l, r) <= budget))
```

## The sliding window is two hand-moved positions
- **Where:** `var l` and the `while cost(l, r) > budget` loop; a monotone
  window (longest range whose predicate holds) is a std-shaped algorithm.
- **Severity:** low.

## A sorted input needs its own generator, and `x <= len` is not a type
- **Where:** `sorted_fields` exists only so `read` makes sorted values; a
  sorted-input type and a bound naming `len` would replace it.
- **Severity:** low.

## The grader reads B as `int`
- **Where:** submission 3732163's own `main` reads `b` as `int`; `orig.cpp`
  reads it as `ll` so B up to 2e15 compares.
- **Severity:** low.
