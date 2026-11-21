This reference covers every primitive type, numeric rule, operator, optional lifting rule, user-defined type, core trait, control-flow construct, and reserved keyword in CP Lang 0.2.2.

## Bindings and constants

Every value is declared with one of three bindings:

```cp CP Lang
let n = 10                      // immutable value (cannot be reassigned)
var ans = 0, ways = 1           // multi-binding on one line
const MOD = 1_000_000_007       // compile-time constant, inlined everywhere
```

Blocks are defined by indentation (no braces). Semicolons `;` separate multiple inline statements on one line (`if ok: a += 1; b -= 1` keeps both statements inside the `if` branch; never write a trailing semicolon at the end of a line). Expressions continue across lines automatically after any binary operator, after `=`, or inside open `()`, `[]`, or `{}`. Comments start with `//` (line comments only).

## Numeric types and literals

CP Lang has four numeric types:

- `Int` — signed 64-bit integer covering `-(2^63 - 1)..=(2^63 - 1)` (the single value `-2^63` is reserved as the niche sentinel for `?Int`, giving zero-overhead optionals).
- `U64` — 64-bit unsigned integer with wrapping arithmetic, intended for bit tricks and polynomial rolling hashes.
- `I128` — signed 128-bit integer for intermediate products like `(I128(a) * I128(b)) % m`.
- `F64` — 64-bit IEEE 754 floating-point number.

Integer literals accept underscores (`1_000_000_007`), scientific notation without a decimal point (`1e9`, `2e5`, `1e18` are all `Int`), hex (`0xFF`), and binary (`0b1010`). A decimal point marks an `F64` literal (`0.0`, `1e-9`).

Range types such as `0..=1_000_000_000` or `-100..=100` check bounds on entry (in `read()` or explicit casts) and pack into 1, 2, or 4 bytes in arrays and structs automatically.

### Floating-point (`F64`) rules

- Integer literals unify directly with `F64` (`let half: F64 = 1 / 2.0`), while `Int` variables require an explicit conversion `F64(x)`.
- `Int(x)` on an `F64` is a compile error — choose the rounding direction explicitly with `floor(x)`, `ceil(x)`, `round(x)`, or `trunc(x)`.
- `NaN` and `Inf` trap locally as soon as they are produced, which makes `<` a strict total order on `F64`.
- Bare `print(x)` on an `F64` is a compile error — specify precision explicitly with `print("{x:.6f}")` (negative zero `-0.0` is normalized to `0.0`).
- Compare floats with `abs(a - b) < eps`; `==` and `!=` on `F64` compare exactly (exact `== 0.0` is fine).
- Built-in math intrinsics: `sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `abs`, and `PI`.

### Scalars and lossless conversions

`Bool` (`true`, `false`), `Char` (a single ASCII byte, e.g. `'a'`, `'\n'`), and `Str` (an ASCII byte string). Lossless conversions are written as type calls: `Int(ch)` (`0..=127`), `Char(i)`, and `Int(b)` (`0` or `1`).

## Operators and compound updates

- **Arithmetic:** `+`, `-`, unary `+`/`-`, `*`, `/` (Euclidean floor division), `%` (Euclidean modulo, always in `0..|b|`), `**` (exponentiation). When the range checker proves `a >= 0` and `b > 0`, `/` and `%` emit native C++ `/` and `%` with zero overhead.
- **Bitwise:** `&`, `|`, `^` (XOR), `~` (bitwise NOT), `<<`, `>>`, plus `&=`, `|=`, `^=`, `<<=`, `>>=`, and helpers `popcount(x)`, `ctz(x)`, `clz(x)`, `bit_len(x)`.
- **Ranges:** `..` (half-open) and `..=` (inclusive) bind looser than arithmetic and tighter than comparisons and `in`, so `i in 0..n - 1` parses as `i in (0..(n - 1))`.
- **Logic and comparison:** `and`, `or`, `not`, `in`, `not in`, `==`, `!=`, `<`, `<=`, `>`, `>=`.
- **General compound updates:** In addition to `+=`, `-=`, `*=`, `/=`, `%=`, any two-argument function `f` can be written as `x f= y` — most commonly `best min= cand`, `best max= cand`, and `g gcd= x`.

## Tropical Optionals (`?T`)

Optional types replace sentinel constants like `1e18` or `-1` across the entire language and standard library. `?Int`, `?F64`, `?V` (domain indices), `?Bool`, `?Char`, and `?Enum` are niche-packed with zero extra bytes.

```cp CP Lang 0.2.2
domain V = 1..=3
let (src, v) = (V.first, V.last)
var dist: [V]?Int = none
dist[src] = 0                       // bare T lifts implicitly into ?T
let via = dist[src] + 7
if dist[v] min= via: print("improved") // update and test whether the value changed
let d = dist[v] ?? -1               // fallback; use parens when mixing ?? with + - * / %
fn maybe() -> ?Int: 7
fn next_value() -> ?Int:
    let opt = maybe()
    let u = opt ?? return none      // diverging RHS (return / break / continue) allowed
    let x = opt!                    // unwrap (traps on none; elided when flow-proven)
    let y = maybe()?                // early-returns none from a fn returning ?U
    u + x + y
print(next_value())
```

### Lifting rules and guardrails

- Arithmetic (`+`, `-`, `*`, `/`, `%`, `**`, unary `-`/`+`) and bitwise (`&`, `|`, `^`, `<<`, `>>`, `~`) operators lift over `?Int` and `?F64`, absorbing `none` (`none + w` and `none * 2` are `none`; division by zero still traps when both sides are present).
- `min(a, b)` and `max(a, b)` skip `none`: `(T, ?T) -> T` and `(?T, ?T) -> ?T`. For three or more values, pass a list (`min([a, b, c])`).
- `x min= cand` and `x max= cand` skip when `cand` is `none` — even when `x` is a plain `Int`!
- Both updates return `Bool`: `true` exactly when the destination changes. Use `if best[v] min= via:` to enqueue a graph relaxation or update a DP witness only on improvement. Ties and `none` candidates return `false`; a present candidate improves an empty destination.
- `==` and `!=` lift `T` to `?T` automatically (`if dq.last != x:`), while `<`, `<=`, `>`, `>=` never lift over `?T`.
- `p ??= init` initializes `p` lazily when `none` and lends it mutably: `(adj[k] ??= []).push(x)`.
- `?V + k`, `sum(gen)`, and compound `+=`/`-=` **never** lift over `?T`.
- Neither `?T` nor `Mod<P>` implements `Ord`. To sort optionals, pass an explicit key such as `by: x => (x.d == none, x.d ?? 0)`.
- When `none` means *invalid/impossible* rather than *+inf/-inf*, check explicitly instead of relying on `min=` skipping it:

```cp CP Lang 0.2.2
fn solve(child: Int) -> ?Int:
    if child >= 0: child else: none

var best: ?Int = none
for child in [3, -1, 4]:
    if let hit = solve(child):
        best min= hit
    else:
        best = none
        break
print(best, none: "-1")
```

## Records, structs, enums, and traits

Define lightweight named tuples with `type`, stateful types with methods using `struct`, and algebraic sum types with `enum`:

```cp CP Lang 0.2.2
type Point = (x: Int, y: Int)

struct Pt: x: Int, y: Int       // one-line colon struct (commas separate fields)

struct Seg:
    l: Int
    r: Int
    fn width(self) -> Int: self.r - self.l

enum Event:
    Open(x: Int)
    Close(x: Int)
    Query(id: Int, x: Int)

enum Dir: N, E, S, W            // one-line plain enum (commas or `|`)
```

- **Records (`type`)** are structural and support both field names (`p.x`) and positional indexing (`p.0`).
- **Structs and enums** are nominal. Seven core traits derive automatically whenever all fields support them: `Index`, `Seq`, `Eq`, `Ord`, `Hash`, `Show`, and `Read`. An enum orders by variant, in declaration order, then by the variant's fields, so `events.sort()` puts every `Open` before every `Close`.
- Automatic tagged-enum input with `@tags("...")` is specified but not yet implemented. Use explicit character matching as shown in [I/O & formatting](io-and-formatting.html).
- **`for f in fields(x)`** runs its body once per field of a struct, record or tuple, in declaration order, with `f` that field; `for (a, b) in fields(x, y)` walks two values of one type in step. The loop is unrolled when the program is compiled, so each `f` has its own type, the body has no `break` or `continue`, and a generic fn can use it on whatever it is given:

  ```cp CP Lang 0.3
  fn same<T>(a: T, b: T) -> Int:
      var n = 0
      for (x, y) in fields(a, b):
          if x == y:
              n += 1
      n
  ```

  On an enum value, `fields(e)` walks the payload of the variant `e` is (nothing for a variant without one); `fields(a, b)` runs only when both are the same variant.
- **`P(e for f in fields(x))`** builds a struct field by field: `e` runs once per field, in order, expected to be that field's type, and gives it. `P(read() for _ in fields(P))` reads one (`fields(P)` names the type, so there is nothing to bind); `P(x + y for (x, y) in fields(a, b))` adds two. A struct with a `new` still goes through `new`.
- Generic functions and types use angle brackets (`fn swap<T>(mut a: T, mut b: T)`), with explicit type arguments at call sites when they cannot be inferred: `read<(Int, Int)>()`, or `f::<T>()`.

## Control flow, pattern matching, and functions

```cp CP Lang 0.2.2
let x: Int = read()
fn neg(): print("negative")
fn zero(): print("zero")
fn pos(): print("positive")

if x < 0:
    neg()
elif x == 0:
    zero()
else:
    pos()

let sign = if x >= 0: 1 else: -1
```

### Pattern matching (`match`) and `if let` / `while let`

`match` is exhaustive and works as both a statement and an expression. Chain multiple optional bindings and boolean guards in `if let` and `while let`:

```cp CP Lang
fn first() -> ?Int: 2
fn second(a: Int) -> ?Int: a + 1
if let a = first(), b = second(a) and a + b > 0:
    print(a + b)

enum Event:
    Open(x: Int)
    Close(x: Int)
    Query(id: Int, x: Int)
let ev = Query(0, 7)
var active = 2
var ans: [0..1]Int = 0
match ev:
    Event.Open(x) => active += 1
    Event.Close(x) => active -= 1
    Event.Query(id, x) if active > 0 => ans[id] = active
    Event.Query(id, _) => ans[id] = 0
```

### Lazy parameters

A parameter marked `lazy` has a nullary function type, and the caller writes the
expression itself; the fn runs it by calling the parameter, zero or more times:

```cp
fn or_else<T>(a: ?T, lazy d: () -> T) -> T:
    if let v = a:
        return v
    d()

fn solve(k: Int) -> Int = k * k

let cached: ?Int = none
print(or_else(cached, solve(4)))   // `solve(4)` runs only because `cached` is none
```

A lambda `() => e` is accepted too.

### Variadic parameters

A last positional parameter written `..xs: Ts` takes every remaining positional argument as one tuple; `Ts` is a type parameter, bound per call to the tuple's type, so the arguments can differ in type. Walk them with `fields`:

```cp
fn show_line<Ts>(..xs: Ts):
    for x in fields(xs):
        print(x)

show_line(3, "abc", true)   // Ts = (Int, Str, Bool)
```

For many arguments of one type, take a `List<T>` and pass `[a, b, c]`.

### Loops, parallel assignment, and closures

- **Counted and infinite loops:** `repeat n:` runs a body `n` times (sugar for `for _ in 0..n:`); `loop:` runs until `break` (sugar for `while true:`).
- **Filtered loops & comprehensions:** `for v in V if ok(v):` and `[f(v) for v in V if ok(v)]` (always use `if` for filters). Comprehensions and generator expressions (`sum(...)`, `[...]`) take a **single** `for` clause — for 2D loops, use nested `for r in Row:` / `for c in Col:` statements.
- **Free math functions:** `abs(x)`, `min(a, b)`, `max(a, b)`, `gcd(a, b)`, `lcm(a, b)` are free functions, not methods on `Int` (write `abs(a - b)`, not `(a - b).abs()`). Power is `**` (`a ** b`, never `.pow`).
- **Conditional expressions, `match` arms, & lambdas:** inline conditionals use `if cond: a else: b` (with colons — never `then` or Python's `a if cond else b`). Never leave `=`, `=>`, `and`, `or`, or binary operators at the end of a line (`E0102`): wrap multi-line expressions in parentheses `(...)`. Both `match` arms (`pat => expr`) and arrow lambdas (`x => expr`, `[V] v => expr`) take a **single value expression on the same line** (no `continue`/`break`/`return` after `=>`). For multi-statement bodies or loop control (`continue`/`break`), define a local `fn f(x: T) -> U:` or use `if`/`elif`/`else`.
- **`mut` parameters vs local copies:** `fn f(mut x: T)` borrows `x` mutably from the caller and requires `f(mut arg)` at every call site (`E0402`). If you only want to reassign a local copy of a scalar parameter inside `f`, keep the parameter immutable (`fn f(x: Int)`) and bind `var cur = x` inside the body.
- **Tuples & reserved keywords:** positional tuples `(Int, Int)` use `.0`, `.1`; named tuples `(a: Int, b: Int)` use `.a`, `.b` (not `.0`). Avoid reserved keywords as variable names — especially `use`, `type`, `in`, `match`, `domain`, `fn`, `let`, `var`.
- **Index-value pairs:** For a list or string, iterate `i in 0..a.len` and access `a[i]` when you need both.
- **Domains and containers:** `for v in V:` iterates a domain; `for v in V by k:` and `for v in V.rev by k:` step along a domain; `for x in a:` visits container elements. For a two-dimensional grid, use nested row/column loops.
- **Modifiers:** `.rev` and `.step(k)` work on any range (`for i in (0..n).step(2).rev:`).
- **Labeled exits:** `'outer: for r in Row: ... break 'outer` (or `continue 'outer`).
- **Parallel assignment:** `(a, b) = (b, a % b)` evaluates the full right-hand side before assigning to the left.
- **Closures & nested functions:** Short expressions use arrow lambdas (`x => x * 2`, `(a, b) => a + b`). Multi-line local helpers that mutate enclosing variables use plain nested `fn` declarations:

```cp CP Lang
var total = 0, step = 1
repeat 4:
    total += step
    step *= 2
loop:
    if total >= 15: break
    total += 1
let evens = [x for x in 0..8 if x % 2 == 0]
print(total, evens)
```

## Reserved keywords and decorators

CP Lang keeps its surface small enough to remember during a contest — **34 keywords total**:

- **Everyday keywords (30, including 3 literals):** `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `and`, `or`, `not`, `assert`, `mut`, `self`, `unchecked`, `true`, `false`, `none`.
- **Library-author keywords (4):** `trait`, `impl`, `lends`, `yield`.
- **Contest decorators (7) + file directive (1):** `@memo`, `@brute`, `@tags`, `@gen`, `@checker`, `@interactor`, `@grader`, plus `@!multitest`.
- **Library-author decorators (4):** `@get`, `@test`, `@law`, `@on_unimplemented`. A trait's `@on_unimplemented("`{T}` can't be hashed; ...")` is the help a call shows when a type argument `T` has no `impl` of it.
