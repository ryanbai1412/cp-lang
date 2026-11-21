# Spec chapter 6: functions, state and control flow

Status: **draft v2** [proposed]. Fills the gaps found in review round 10 (items 32–34, 38).

## 6.1 Bindings and state

- `let` binds once; `var` is mutable. Every `var` is initialized at its declaration (`var a: [V]Int = 0`
  fills). There is no uninitialized `var` and no `++`.
- A `let` may shadow an earlier binding in the same block, except a name captured by a hoisted function
  (compile error) [r11 N12]. In an un-annotated `var`, a range type infers `Int`; every other type
  (positions, fixed widths, structs) infers itself, so `var total = 1; total += x` never trips a contract; a `let` keeps the computed range [r11 N6].
- **`static let` / `static var`** are the only state shared across tests. Everything else in a
  `@multitest` body is per test. Statics are initialized once, before the first test, and never reset
  [r10 32]. A static can't have a type that mentions a runtime-sized domain or scoped type (chapter 1.4),
  and its initializer can't read input or use `gen` [r11 N17].
- **Heap values copy on assignment** [p1 Q3, proposed default]: `var out = s` copies a `List`, `Str` or `Map`,
  as C++ does. There is no move checker. A lint and `cp explain` flag copies of large values inside loops;
  pass by `let` parameter to avoid them (parameters never copy). A temporary moves instead of copying [r13 P5].

## 6.2 Functions and lambdas

- `fn f(x: T) -> U = expr` or a block body. Functions **hoist** within their block; every capture must be
  initialized before the first call (checked, chapter 1 diagnostics style).
- Nested functions capture by reference to the enclosing activation. This is the default way to split a
  long solution into chunks: each chunk is a nested function with its inputs as parameters. A nested function
  can have a nested `@alt` sibling sharing its captures, compared at real call sites; it moves to the top
  level only to be stressed alone with its own `@gen` [you, r2; p3 proposed].
- Function values (lambdas, nested functions) can't leave their declaring block or go into a static or
  field, by the same rule as domains, so capture by reference is safe [r11 N13].
- `params => e` is the only lambda. Parameters are patterns, so `(u, v) => e` takes two arguments or one
  pair, whichever is expected. `fn` parameters may be patterns too, but stay typed:
  `fn paths((r1, c1): Pt, (r2, c2): Pt)` [r16 R3]. There are no separate argument labels. Arrays and bound methods are functions.
- Named arguments: options are always named, never bare flags. A bare variant name in a named argument
  resolves against that parameter's enum (`order: desc`). Keywords may be argument labels (`first(where:)`).

## 6.3 Control flow

- `if c: … else: …`; one conditional expression `if c: x else: y`.
- `for x in seq where cond:`; `while cond:`; `while let x = opt where cond:`; `loop:` with `break` /
  `continue`. `where` is the one guard keyword. Loops may be labeled (`outer: for …`) and `break outer` /
  `continue outer` exit them [r11 N18].
- `if x min= y:` / `while x max= y:`: a `min=`/`max=` that is the **whole** condition yields whether `x`
  changed (a `none` target always does; a lifted `none` right side and an equal value don't). Nowhere else is
  an assignment an expression [r16 R13].
- `if let a = x, b = y:` binds left to right and stops at the first `none` [r11 N11].
- `return`, `break`, `continue` and `fail` are expressions of type `Never`, which converts to any type, so
  `?? return List.filled(n, 0)` works [r11 N5].
- `repeat n:` runs n times; a negative `n` traps [r10 38].
- `match` is exhaustive. It works on enums, literals, tuples and strings; strings need a catch-all arm
  (`_` or a name, as in `other =>`) [r11 N8].
- `return` from nested functions returns from that function only.
- **Evaluation order** [p1 S4]: operands and arguments evaluate left to right; array `map` and `for` run in
  index order, so a lambda may mutate captures and the effect order is defined.
- **Precedence**, tightest first: postfix (`.f`, `[i]`, `!`, `?`, calls); `**` (right-associative); unary `-` `~`; `* / %`; `+ -`;
  `<< >>`; `&`; `^`; `|`; ranges `.. ..=`; comparisons (non-associative); `not`; `and`; `or`; `??` (right-associative); assignments. So `-x ** 2` is
  `-(x ** 2)` and `not a == b` is `not (a == b)` [r13 P3, P4].
  So `s & (s << 1) == 0` means `(s & (s << 1)) == 0`.
- A dropped non-`Never` result lints, except for stdlib functions marked droppable: `mark`, `pop`, `push`, `union`, `insert` [r13 P6].

- **Grammar** [p2 S3, S12]: a `match` on a range-typed scrutinee is exhaustive when literal arms cover the range,
  and a byte-labelled domain matches on its literals (`'I' =>`). `_` is legal in `let` patterns. `return` in the
  program body ends the program, or the current test under `@multitest`; a lint fires when a `read` is
  reachable after it, naming "read the rest first" [r14 T9]. `assert(cond)` checks an invariant;
  `with checks(asserts: off)` removes it [p2 F9]. Destructuring assignment `(a, b) = (x, y)` works on `var`s.
- **Ordering** [p2 S2]: tuples and structs whose fields are all `Ord` order lexicographically, so
  `best max= (value, who)` is the argmax. A struct widens field-wise when each field widens [p2 S13].

## 6.3b Enums [p1 S5]

- A labeled domain is the plain enum: `domain Side = labels(left, right)` gives `Side.left`, a `[Side]T`
  array, `match` exhaustiveness and named-argument resolution (`order: desc`).
- `enum Shape: kind(Kind), angle` adds payload variants; it is not an index space.

## 6.4 `@memo`

```cp
@memo
fn count(i: 0..=k.len, r: 0..d, tight: Bool) -> Z: ...
```

- **Table choice is explicit** [r7, r10 33]. When every parameter has a bounded type, the cache is a dense
  array over the product of those bounds (`[0..=k.len][0..d][Bool]?Z`); `cp explain` shows its bytes, and
  over the profile's memory it is a compile-time warning. Otherwise it is a compile error naming
  `@memo(hash)`. A dense table never silently becomes a hash map.
- **Lifetime:** the cache lives as long as the block declaring the function: per test in a `@multitest`
  body, per activation inside another function (`upto(k)` gets a fresh cache per call).
- A `@memo` function can't do I/O, mutate captured state, or capture anything but `let` bindings of
  values that can't change while the cache lives. A write to a captured array after the `@memo` declaration
  in the same block is a compile error; writes through other paths hit the debug generation check (3.2)
  [r11 N14, r12 R11].
  It can't outlive the values its bounds mention.
- `cp explain` shows each table's bytes and fill cost per activation [r11 N19].
- Mutual recursion between `@memo` functions is allowed; each has its own table.
- Domains, cuts and bounded ints all count as bounded [p2 S5]. `@memo(hash)` is open addressing over the key's
  storage width. Lowering may run leading non-recursive guards before the probe (unobservable, since `@memo`
  is pure); the dense table is still sized by the types [r14 T18].

## 6.5 Recursion depth

- The compiler infers a depth bound in two cases:
  - **tree recursion:** the recursive argument is bound by `for u in t.children(v)` with `v` the parameter;
    the bound is the domain size [r11 N16];
  - **monotone recursion:** at every recursive call a range-typed parameter `p` is passed as `p + c` (or
    `p - c`), `c` a positive literal or positive range type [r12 R12];
    the bound is its range's size, e.g. Digit Sum's `i: 0..=k.len` gives `k.len + 1` [r11 N15].
  - **sum measure** [p2 S6, r14 T22]: every argument in the measure is `p`, `p + c` or `p - c` with literal `c`,
    and the deltas sum to the same-signed constant at every call; the bound is the sum's range size;
  - **halving** [r14 T3]: the recursive argument comes from `r.halves()` (defined for `len ≥ 2`, traps
    otherwise) or is `p / c` with `c ≥ 2` and `p`'s lower bound ≥ 1; the bound is ⌈log₂⌉ + 1.
  Anywhere else, `@depth(expr)` states it or the compiler warns.
- Over the judge profile's stack, the bundler runs `main` on a big-stack thread where the profile allows it
  and warns where it doesn't [r10 34].
- Arrays never go on the stack (Tourism's accepted C++ segfaulted at n = 10⁶ on 8 MB because of VLAs).

## 6.5b Round-3 rules [p3]

- **Closures.** Nested functions and lambdas capture enclosing `var`s by reference and may mutate them, except
  lambdas passed to lazy adapters (3.6b). A function-typed parameter is written `(A, B) -> R`, and a nullary
  lambda `() => e`. Function parameters don't escape; a call with a statically known function value is
  specialized per value, never through a pointer. `cp explain` lists the clones with their bytes.
- **`@memo` tables.**
  - A dense table is allocated on the first call.
  - A parameter bound may mention earlier parameters (`j: 0..=i`). The table is then jagged: row offsets are
    computed once, and `cp explain` shows the total.
  - The cache entry is `?R` even when `R` is optional, stored with a second niche or a flag byte.
- **Depth inference, more sound shapes:**
  - `@depth` on any function in a mutual-recursion cycle covers the cycle.
  - Domain stepping: the recursive argument is `x.next!`, `x.prev!`, `c.right!.after`, or `t.parent[x]!` of a
    `Tree`/`Forest` (never a raw `[V]?V`, which may cycle). Every call in the group steps the same direction.
    The bound is the domain size.
  - Decreasing position: the argument is bound by `where u < v` (or `filter`) with `v` the parameter, or
    drawn from a range ending before it (`..v`, `D.first .. v`) [r16 R5]. The bound is the domain size.
  - Doubling: `2 * p + c` with `p ≥ 1` and `c ≥ 0`, where `B` is `p`'s range-type upper bound (the contract
    traps past it), gives ⌈log₂ B⌉.

## 6.6 Open questions

1. ~~Reset statics per test?~~ Never; a non-static already gives per-test state [r11].
2. ~~Negative `repeat n` a compile error?~~ No; runtime trap, ranges on the read rule it out [r11].
