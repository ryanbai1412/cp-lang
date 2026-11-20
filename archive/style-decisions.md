# Style decisions

Each decision changes only how the language looks, not what it can do. The
current choice is marked **(now)** and my pick is marked **(rec)**. Reply with
something like `1A 2A 3C ...`; anything you skip keeps my pick.

## 1. Blocks — decided: A, plus `{ ... }` for inline expression blocks

```
let order = sorted_by(xs, a => { let k = key(a); (k.x, -k.y) })
```

```
A (now, rec)            B                       C
for u in V:             for u in V {            for u in V do
    if seen[u]:             if seen[u] {            if seen[u] then
        continue                continue                continue
    visit(u)                }                       end
                            visit(u)                visit(u)
                        }                       end
```

## 2. Bindings — decided: `let` / `var` (default pick)

```
A (now, rec)            B                       C
let n = read()          val n = read()          n := read()
var best = 0            var best = 0            mut best := 0
```

## 3. Functions — decided: `fn f(x: T) -> R:` plus `= expr` one-liners (default pick)

```
A (now, rec)                       B
fn dist(a: P, b: P) -> Int:        def dist(a: P, b: P) -> Int:
    abs(a.x - b.x)                     abs(a.x - b.x)

C (one-line form allowed too)
fn dist(a: P, b: P) -> Int = abs(a.x - b.x)
```

With A, C can be added as an extra form for one-liners.

## 4. Array types — decided: `[V]Int` (default pick)

```
A (now, rec)            B                       C
[V]Int                  Int[V]                  Array<V, Int>
[Row, Col]Char          Char[Row, Col]          Array<(Row, Col), Char>
```

A reads left to right, like the access: "indexed by V, gives Int".

## 5. Optionals — decided: prefix `?T` with `??`, `?`, `if let` (default pick)

```
A (now, rec)                     B                         C
fn find() -> ?V                  fn find() -> V?           fn find() -> Option<V>
let v = find() ?? V.first        (same operators)          find().unwrap_or(V.first)
if let v = find(): ...                                     if let Some(v) = find(): ...
```

## 6. Ranges — decided: rec (default pick)

```
A (now, rec)            B (Swift)               C (math)
0..n    half-open       0..<n                   [0, n)
1..=n   inclusive       1...n                   [1, n]
```

## 7. Declaring domains — decided: `domain` (default pick)

```
A (now, rec)                  B                          C
domain V = 1..=n              index V = 1..=n            type V = Index(1..=n)
domain Node = In + Out        index Node = In + Out      type Node = Index(In + Out)
```

## 8. Tables (array comprehensions) — decided: A, plus C when the declared type is `[V]T`

```
A (now, rec)                       B                           C
[V] u => deg[u] * 2                [V] |u| deg[u] * 2          [deg[u] * 2 for u in V]
```

C reuses list-comprehension syntax but hides that the result is indexed by V.

## 9. Generics — decided: A, and `<D>` may be omitted for single capital letters

```
A (now, rec)                    B                            C
fn peel<D>(s: [D]Bool)          fn peel[D](s: [D]Bool)       fn peel(s: [D]Bool)  // D inferred
```

## 10. Boolean operators — decided: `and` / `or` / `not` (default pick)

```
A (now, rec)                     B
if not seen[v] and d < best:     if !seen[v] && d < best:
```

## 11. Decorators — decided: `@memo` (default pick)

```
A (now, rec)            B                       C
@memo                   #[memo]                 memo fn f(...)
fn f(...)               fn f(...)
```

## 12. Match arms — decided: B (`=>`)

```
A (now)                 B (rec)                 C
match op:               match op:               match op:
    Add(l, r): ...          Add(l, r) => ...        case Add(l, r): ...
    Ask(i): ...             Ask(i) => ...           case Ask(i): ...
```

I recommend B because `:` already means "a block follows". With `=>`, a
one-line arm is easy to tell apart from a block.

## 13. Comments — decided: rec (default pick)

```
A (now, rec)            B
// note                 # note
```

## 14. Chaining — decided: rec (default pick)

```
A (now, rec)                     B                              C
xs.sorted().rev()                xs |> sorted |> rev            both
```

## 15. Marking mutation at the call site — decided: `mut` at param and call site; no marker on method receivers

```
A (rec)                 B                       C (prototype now)
link(inout g, r)        link(&mut g, r)         link(g, r)
```

## 16. Multi-dimensional indexing — decided: `g[r, c]` only

```
A (now, rec)            B
grid[r, c]              grid[r][c]
```

## 17. Output — decided: `print` + every string literal interpolates (`r"..."` raw)

```
A (now)                 B                       C (rec)
print(a, b)             print("{a} {b}")        both
```

## 18. Enum declarations — decided: both forms (one-line `A | B`, block with payloads/methods)

```
A (now)                       B                          C (rec)
enum Op:                      enum Op = Add(l, r: P)     both
    Add(l, r: P)                      | Ask(i: P)
    Ask(i: P)
```

## 19. Naming — decided: current naming

```
A (now, rec)                                B
snake_case for values and fns,              camelCase for values and fns,
PascalCase for types and domains            PascalCase for types and domains
```

## 20. Statement terminators — decided: no semicolons

```
A (now, rec)            B
let x = 1               let x = 1;
```

---

# Decisions 21–33 (proposed)

## 21. Lambdas

```
A (now, rec)                     B                         C
sort_by(xs, e => e.w)            sort_by(xs, |e| e.w)      sort_by(xs, fn(e) e.w)
(a, b) => a + b                  |a, b| a + b              fn(a, b) a + b
```

A uses the same `=>` as tables and match arms.

## 22. Inline conditional

```
A (now, rec)                          B                        C
let d = if a < b: b - a else: a - b   let d = a < b ? b - a    let d = (b - a) if a < b
                                              : a - b                  else (a - b)
```

B clashes with `?` for optionals.

## 23. Power and xor

```
A (rec)                 B                       C (now)
x ** 2                  x ^ 2                   pow(x, 2)
a ^ b   // xor          a xor b                 a ^ b   // xor
```

B means `^` reads differently from C++, where it's xor.

## 24. Integer division and remainder — decided: A (floor; `%` never negative)

```
A (rec)                          B (now, C++)
-7 / 3 == -3                     -7 / 3 == -2
-7 % 3 == 2                      -7 % 3 == -1
```

A removes the negative-remainder bug. When both operands are provably
non-negative from their ranges, A compiles to plain `/` and `%`.

## 25. Numeric literals

```
A (now, rec)                     B
let MOD = 1e9 + 7                let MOD = 1000000007
let n: 1..=2e5 = read()          let n: 1..=200000 = read()
let big = 1_000_000              let big = 1000000
```

In A, `1e9` is an `Int` when the value is an exact integer.

## 26. Range steps

```
A (rec)                 B                       C
for i in 0..n by 2      for i in range(0, n, 2) for i in (0..n).step(2)
for v in V.rev          for v in reversed(V)    for v in V.rev()
```

## 27. Chained comparisons

```
A (rec)                          B
if lo <= x < hi:                 if lo <= x and x < hi:
```

## 28. Return values

```
A (now, rec)                     B
fn sq(x: Int) -> Int:            fn sq(x: Int) -> Int:
    x * x                            return x * x
```

`return` stays for early exit in both.

## 29. Operation structures (monoids, actions)

```
A (rec)                                     B
const best = Monoid<Int>(id: -INF, op: max) struct MaxI(v: Int):
var st = SegTree<V, Int>(best)                  impl Monoid:
                                                    fn id() -> MaxI = MaxI(-INF)
                                                    fn op(a: MaxI, b: MaxI) -> MaxI = ...
                                            var st = SegTree<V, MaxI>()
```

A treats the operation as a value, so one type can have many monoids (max,
min, sum) with no wrapper types. Laws are checked per `const` value, and the
generated C++ inlines `op`. Traits stay for things a type has exactly one of:
`Ord`, `Eq`, `Hash`, `Show`, `Read`, `Add`.

## 30. Compile-time branches

```
A (rec)                          B                         C
if const D.max_len <= 64:        static if D.max_len <= 64: when D.max_len <= 64:
    ... u64 mask ...                 ... u64 mask ...           ... u64 mask ...
```

## 31. Unused bindings

```
A (now, rec)                     B
for (_, v) in edges: ...         for (_e, v) in edges: ...
```

## 32. Assertions

```
A (rec)                                  B
assert lo <= hi   // kept in submissions assert lo <= hi   // removed in submissions
                  // unless --release
```

## 33. Modules

```
A (rec)                          B
use std.segtree               (whole stdlib always in scope)
let st = SegTree<V, Int>(sum)    let st = SegTree<V, Int>(sum)
```

## Also decided in discussion

- Anonymous records stay for one-off shapes: `-> (lo: Int, hi: V)`.
- Structs: fields, defaults and methods in one body; `self` / `mut self`;
  auto constructors (positional or labeled); `e with w: 7`.
- Copies: big values (arrays, lists, maps, sets, records holding them) need an
  explicit `.copy()`; moves (last use) stay implicit and free.
- Constructors take args by value: moved in on last use, else `.copy()`.
  Custom constructors are static fns (`Graph.read()`); `e with w: 7` moves or
  copies `e` by the same rule. No constructor side effects, no user destructors.
- Integers: `Int` is 64-bit; range types (`0..=1e9`, `0..=255`) pick the
  storage width. `I32`/`U8`/`I128` only for bit-level work. Types stay
  capitalized.
- Getters: opt-in. A read-only method marked `@get` (no other params) is used
  without parens (`s.len`); every other method keeps its parens.
- Params: positional or named args, defaults, named-only after `;`, typed
  variadics `..xs: T`, no `*args` / `**kwargs`.
- Strings: `"Case #{t}: {ans}"` interpolates; `{{` escapes; `r"..."` raw.
- Ownership: variables own, params borrow, no pointers; big copies need
  `.copy()` (pending confirmation).

