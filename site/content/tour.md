You already know how to solve the problem. This chapter is about expressing the solution: indentation instead of braces, values instead of sentinels, and a file that reads like the main flow of your algorithm.

## Values and mutation

Use `let` by default, and `var` for a binding that changes. `Int` is the default 64-bit integer. A range type states a constraint, rather than asking you to remember which width is enough.

```example accumulation
```

Range-typed storage is intended to use the smallest fitting width, while arithmetic uses `Int`. `Int` reserves the most-negative 64-bit bit pattern for `none`, so `?Int` fits in the same eight bytes. Use the explicit fixed-width types for bit-level work.

## Missing is a value

`?T` means “a `T`, or `none`”. `??` supplies a fallback; `if let` uses the value only when it exists. A missing predecessor is not `-1`, and an unreachable distance is not a suspiciously large integer.

```example optional
```

`v.prev` returns `none` at the first position. You cannot silently step outside the domain. The same pattern handles `v.next`, an empty minimum, and a lookup that finds nothing.

## Functions and explicit changes

A parameter borrows by default. A function that changes its argument marks it `mut`, and its caller agrees by writing `mut` too. There is no implicit deep copy of an array.

```cp CP Lang · design
fn add_to(mut a: [V]Int, v: V, delta: Int):
    a[v] += delta

add_to(mut a, V.first, 3)
let snapshot = a.copy()
```

An ordinary helper can see immutable values in the surrounding program. A nested `mut fn` can mutate captured variables but cannot escape as a stored function value. Read [ownership & borrowing](borrowing.html) before writing helpers that keep views into containers.

## Records and choices

Name the pieces of an edge instead of remembering what `first.second` means. Use an enum when the input describes one of several shapes, and `match` to handle each shape.

```cp CP Lang · design
let edges: [E](u: V, v: V, w: 1..=1e9) = read()

enum Move:
    Add(v: V, delta: Int)
    Ask(l: V, r: V)

match move:
    Add(v, delta) => add_to(mut a, v, delta)
    Ask(l, r) => print(range_sum(l, r))
```

This is a fragment: it assumes domains `V` and `E`, an array `a`, and the two helpers are already defined. The [Modern Art corpus port](https://github.com/ryanbai1412/cp-lang/blob/main/proto/ports/ccc21s2.cp) uses this pattern for row and column strokes.

## A few C++ habits to leave behind

Write `and`, `or`, and `not` for logical operators. `..` excludes the end; `..=` includes it. A position plus an integer is deliberately ambiguous: choose `.next`, `.offset(k)`, or `.cyc(k)` depending on the edge behavior you want.

In the approved design, integer division is `/` and remainder is `%`, with Euclidean semantics. The remainder is always non-negative and smaller than the absolute divisor: `-7 / 3` is `-3`, `-7 % 3` is `2`, and `7 / -3` is `-2` with remainder `1`. Division by zero is an error. When operands are proven non-negative, the backend can use native C++ operations. `//` starts a line comment, not a division expression.

## Program structure

Use top-level code for the solution's flow and functions for parts you want to test alone. A top-level `return` ends the program, or the current case under `@multitest`. The decorator is library-level contest behavior, not a special kind of program body.

```cp CP Lang · design
@multitest(sum: n <= 2e5)
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
print(sum(a))
```

Each case gets fresh ordinary state. Expensive immutable precomputation can use `static let`. Library modules contain declarations rather than hidden top-level actions; importing a helper should not secretly read input or change state.
