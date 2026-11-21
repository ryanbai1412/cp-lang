# 1.8 Syntax and style reference

Status: decided. The user reviewed every item in `archive/style-decisions.md`
(1-33) and said they were aligned. Items marked *proposed* came later and
haven't been reviewed.

This is the complete surface syntax. Later documents use it without
explaining it again.

## Lexical structure

- **Blocks are indentation-based**, introduced by `:`. Indentation is spaces
  only (tabs are an error), and a dedent must return to an enclosing level.
  `{ ... }` exists only for inline expression blocks:
  `sort(xs, by: a => { let k = key(a); (k.x, -k.y) })`. Inside brackets,
  newlines don't end statements.
- **No semicolons**, except inside `{ ... }` inline blocks.
- **Comments** are `// ...` to the end of the line.
- **Naming**: `snake_case` for values and fns, `PascalCase` for types and
  domains. The parser relies on this: in an expression, `Name<` starts
  generic arguments only when `Name` is capitalized and what follows parses
  as type arguments closed by `>` (so `INF < x` is still a comparison;
  *proposed*). Lowercase generic fns get type arguments from the expected
  type; an explicit form for the rare remaining case is open.
- **Numeric literals**: `1e9`, `2e5` and `1e9 + 7` are `Int`s when the value
  is an exact integer; `1_000_000` allows underscores; floats need a
  fraction or a non-integer value (`0.5`, `1.5e-9`).
- **Strings**: every literal interpolates (`"Case #{t}: {ans}"`), `{{` and
  `}}` escape braces, and `r"..."` is raw.

## Bindings and functions

```
let n = read()                  // immutable binding
var best = 0                    // mutable binding
let _ = f()                     // unused

fn dist(a: P, b: P) -> Int:     // block body; the last expression is the value
    abs(a.x - b.x) + abs(a.y - b.y)
fn sq(x: Int) -> Int = x * x    // one-line form
```

- `return` exists only for early exits.
- Parameters can be passed by position or by name, can have defaults, and
  can be marked named-only after `;`. A variadic `..xs: Ts` takes the
  remaining positional arguments as one tuple, and `Ts` is a type parameter
  bound to that tuple's type; `for x in fields(xs)` walks it. There are no
  `*args` or `**kwargs`.
- A mutating parameter is `mut` both where it's declared and where it's
  passed: `fn link(mut g: Dsu, ...)` and `link(mut g, r)`. Method receivers
  don't need the marker at the call site: `st.update(i, x)`.
- Lambdas: `x => x + 1` and `(a, b) => a + b`.

## Types

- **Integers**: `Int` is 64-bit, with `Int.MIN = -(2^63 - 1)` so that `?Int`
  can use the remaining value as its niche. Range types like `0..=1e9` pick
  the storage width. `I32`, `U8`, `U64` and `I128` are for bit-level work.
- **Optionals**: `?T`, with `a ?? b`, postfix `?` to propagate, and
  `if let v = find(): ...`.
- **Arrays indexed by a domain**: `[V]Int`; grids `[Row, Col]Char`, indexed
  `g[r, c]` (never `g[r][c]`).
- **Generics**: `fn peel<D>(s: [D]Bool)`. `<D>` can be left out when the
  parameter is a single capital letter.
- **Records** for one-off shapes: `-> (lo: Int, hi: V)`.
- **Structs** hold fields, defaults and methods in one body, with `self` and
  `mut self`. Constructors are generated (positional or labeled), and
  `e with w: 7` makes an updated copy. Custom constructors are static fns,
  such as `Graph.read()`. There are no constructor side effects and no user
  destructors.
- **Enums** have two forms: `enum Dir = L | R | U | D`, or a block form with
  payloads and methods.
- **Getters** are opt-in: a read-only method with no other parameters,
  marked `@get`, is called without parentheses (`s.len`).

## Expressions

- **Ranges**: `0..n` is half-open and `1..=n` is inclusive. Steps are written
  `0..n by 2`, reversal `V.rev`, and chained comparisons `lo <= x < hi`.
- **Boolean operators**: `and`, `or`, `not`.
- **Arithmetic**: `x ** 2` is power and `a ^ b` is xor (as in C++). `/` is
  floor division and `%` is never negative (decision 24).
- **Inline conditional**: `if a < b: b - a else: a - b`.
- **Match** arms use `=>`:

  ```
  match op:
      Add(l, r) => st.add(l, r)
      Ask(i) =>
          let x = st.get(i)
          print(x)
  ```

- **Tables** (domain-indexed arrays): `[V] u => deg[u] * 2`. A
  comprehension `[deg[u] * 2 for u in V]` builds a `List`, except when the
  declared type is `[V]T`, where it builds the array.
- **Compound assignment** includes `best max= x` and `best min= x`.
- **Chaining**: `xs.sorted().rev`.
- **Compile-time branches**: `if const D.max_len <= 64: ...`.
- **Assertions**: `assert lo <= hi` is kept in submissions unless built with
  `--release`.

## Items

- **Decorators**: `@memo`, `@multitest`, `@brute(f)`, `@get`, `@test`.
- **Domains**: `domain V = 1..=n`, and `domain Node = In + Out` for disjoint
  unions.
- **Modules**: `use std.segtree`; then `SegTree<V, Int>(sum)`.
- **Operation values**: `const best = Monoid<Int>(id: -INF, op: max)`, then
  `var st = SegTree<V, Int>(best)`.

## Proposed and not yet reviewed

- `a.raw[i]`: the domain escape hatch (decided in principle as decision
  2.11; this spelling is proposed).
- `unchecked:` blocks (research/RECOMMENDATIONS.md).
- `requires`, `ensures`, `invariant`, and `where` refinements on input
  records (plan/18-key-features.md).
- The `decorator` declaration form (decision 3.11).
