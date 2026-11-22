# Reading Diagnostics and Fixing Top Codes

Diagnostics have a prefix indicating the phase and severity:

- `E00xx` Lexing (tabs, `#` comments, bad tokens)
- `E01xx` Parsing (precedence, syntax)
- `E02xx` Names (unresolved symbol/field/method, casing)
- `E03xx` Types and domains (wrong domain, type mismatch, optional ordering)
- `E04xx` Mutation (`let` assignment, missing `mut`)
- `E05xx` Removed or unsupported syntax
- `E06xx` Ownership, lending, `Print` trait (`Bool` printing)
- `E07xx` Traits and generics (ambiguous method)
- `E08xx` Contexts, inference, fresh types
- `E09xx` Numbers and layout (`U8` / width rules)
- `Wxxxx` Warnings (unused/dropped values)
- `Lxxxx` Lints (complexity or style traps, e.g. `L0608`)
- `Rxxxx` Runtime traps in checked builds (`cpl run`, `cpl test`, `cpl stress`)

## Top Compile-Time Errors and Fixes

### `E0001` / `E0004` — Tabs or `#` Comments

Use 4 spaces for indentation (never tabs) and `//` for comments (never `#`).

### `E0110` — Unparenthesized `??`

`??` cannot be mixed with `+`, `-`, `*`, `<`, etc. without parentheses:

```cp
let x: ?Int = None
let y = (x ?? 0) + 1
print(y)
// output: 1
```

### `E0208` — Naming Convention

Variables and functions must start lowercase (`snake_case`); types, domains,
and enum variants must start uppercase (`PascalCase`).

### `E0301` / `E0302` — Wrong Domain or Type Mismatch

`[V]T` only accepts `V`, not `Int` or another domain `W`. Convert explicitly
with `V(i)` or `V.at(i)` (returning `?V`), or declare the loop over `for v in V:`.

### `E0307` — Non-`Bool` Condition

`if` and `while` require a `Bool`: write `if n != 0:` instead of `if n:`, and
`if let Some(v) = opt:` instead of `if opt:`.

```cp error(E0307)
let n = 3
if n:
    print(n)
```

### `E0308` — Ordering on `?T`

Arithmetic (`+`, `-`, `*`) lifts through `?T`, and `min`/`max`/`min=`/`max=`
ignore `None`, but `<`, `<=`, `>`, `>=` do not accept `?T`. Unwrap first:

```cp
let a: ?Int = Some(3)
let b: ?Int = None
var best = a
best min= b
print(best!)
// output: 3
```

### `E0401` / `E0402` — Mutability

- `E0401`: binding was declared `let`; change to `var`.
- `E0402`: function parameter is `mut`; add `mut` at the call site (`f(mut x)`).

### `E0601` / `E0602` / `E0608` — Lending and Moves

- `E0601` / `E0602`: a view or generator is still alive while mutating its
  container. Finish reading the view (or `.collect()` / copy out scalars) before
  mutating.
- `E0608`: a heap value was moved and used again. Pass it by lend (function
  parameter) or call `.clone()`.

### `E0606` — `Bool` Does Not Implement `Print`

Wrap booleans with `yes(b)` (`YES`/`NO`), `yes(b, ("Yes", "No"))`, or `Int(b)`.

## Top Runtime Traps (`cpl run`)

- `R0101` / `R0102`: out-of-bounds `List` index or `V(x)` outside domain `V`.
- `R0201` / `R0202`: value outside declared range type or integer overflow.
- `R0301`: `!` unwrap on `None` — replace with `?? default` or `if let Some(v) = ...`.
- `R0401`: `assert` failed.
- `R0501` / `R0502`: stdin token parse failure or input range/domain violation.
