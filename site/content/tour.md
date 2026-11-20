You already know how to solve the problem. This chapter is about expressing the solution in CP Lang v2.2: indentation instead of braces, `?T` instead of sentinels, structural `[0..n]T` arrays alongside nominal domains, and a file that reads like the flow of your algorithm.

## Values, ranges, and mutation

Use `let` by default, `var` for bindings that change, and `const` for compile-time constants. `Int` is the default 64-bit signed integer. A range type states the problem's constraint right where the value enters the program.

```example accumulation
```

Range-typed storage uses the smallest fitting integer width while arithmetic runs in 64-bit `Int`. `Int` reserves `LLONG_MIN` as the niche for `none`, so `?Int` fits in eight bytes with zero overhead. Use `U64`, `I128`, and `F64` when you specifically need unsigned 64-bit, 128-bit, or floating-point arithmetic.

When you want a plain integer-indexed array without introducing a nominal domain, write the bounds directly and reset with `.fill`:

```example range_array
```

In CP Lang v2.2, `cnt.fill(0)` resets the entire array in place (and `seen.fill(none)` resets a `[D]?T` table between test cases).

## Missing is a value

`?T` means “a `T`, or `none`”. `??` supplies a fallback, `if let` binds a present value, and `opt!` unwraps when you know it exists. A missing answer is not `-1`, and an unreachable distance is not `1e18`.

```example optional
```

`?Int` and `?F64` are tropical: binary `+`, `-`, and unary `-` absorb `none`, while `min`, `max`, `min=`, and `max=` skip it, so `dist[v] min= dist[u] + w` is a complete relaxation step with no `INF`. `min(a, b)` returns a definite `T` as soon as one side is a plain `T`. `*`, `/`, `%`, `+=`, `-=`, and `sum` never lift over `?T`; write `dp[u] = dp[u] + w` when you want absorbing addition. When `none` means failure rather than infinity, test it explicitly instead of relying on `min=` to skip it. Stepping a domain position by an integer (`v + 1`, `v - 1`, `v + k`) also returns `?V`, so stepping off either end can never silently read out of bounds. Neither `?T` nor `Mod<P>` implements `Ord`; when sorting records with optional fields, place `none` explicitly (`by: x => (x.d == none, x.d ?? 0)`).

## Functions, nested helpers, and explicit `mut`

Parameters borrow read-only by default. A function that mutates an argument marks it `mut`, and the caller writes `mut` too. There is no implicit deep copy of an array.

```cp CP Lang v2.2
fn add_to(mut a: [V]Int, v: V, delta: Int):
    a[v] += delta

add_to(mut a, V.first, 3)
let snapshot = a.copy()
```

A plain nested `fn` can read and mutate captured outer `var` bindings directly—ideal for a DFS that updates `vis` and `order` without threading extra parameters:

```cp CP Lang v2.2
var vis: [V]Bool = false
fn dfs(u: V):
    vis[u] = true
    for v in g[u] if not vis[v]:
        dfs(v)
```

Read [ownership & borrowing](borrowing.html) for the lexical rules behind `mut` borrows and `lends` projections.

## Parallel assignment, explicit undo, records, and choices

Swap or step coupled state with parallel tuple assignment—the right-hand side is evaluated completely before any binding is written:

```cp CP Lang v2.2
var (r0, r1) = (a, b)
while r1 != 0:
    let q = r0 / r1
    (r0, r1) = (r1, r0 - q * r1)
```

There is no `defer`. Undo is one explicit line after the work it covers, which also works across sibling branches and early returns:

```cp CP Lang v2.2
let snap = dsu.mark()
solve(left)
dsu.undo_to(snap)

cur.push(x)
dfs(step + 1)
cur.pop()
```

Name record fields instead of remembering `.first.second`, and match enums exhaustively with bare variant names:

```cp CP Lang v2.2
type Edge = (u: V, v: V, w: 1..=1e9)
let edges: [0..m]Edge = read()

enum Move:
    Add(v: V, delta: Int)
    Ask(l: V, r: V)

match move:
    Add(v, delta) => add_to(mut a, v, delta)
    Ask(l, r) => print(range_sum(l, r))
```

## A few C++ habits to leave behind

- Write `and`, `or`, and `not` for logical operators, and `elif`, never `else if`.
- Mutate elements through an indexed loop (`for i in 0..xs.len: xs[i].f()`); there is no `for mut x in xs`.
- `..` excludes the end; `..=` includes it. Open-ended ranges `..i`, `..=i`, and `i..` work in slices and range structures (`st.fold(..i)`).
- Stepping a nominal domain position with `v + k` or `v - k` returns `?V`: handle it with `if let`, `??`, or `!`. `v.cyc(k)` wraps cyclically.
- Use `x min= y` and `x max= y` (any two-argument function works as `x f= y`).
- Integer division `/` and remainder `%` use Euclidean semantics: remainder is always in `0..|b|`, so `-7 / 3` is `-3` and `-7 % 3` is `2`. When operands are proven non-negative, C++ codegen lowers straight to native `/` and `%`. `//` is always a line comment.
- Break out of nested loops with labels: `'outer: for r in Row:` and `break 'outer`.

## Program structure

Top-level statements run in order from top to bottom. A top-level `return` ends the program, or the current test case under `@multitest`:

```cp CP Lang v2.2
@multitest
let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()
print(sum(a))
```

`@multitest` also accepts `@multitest(until_eof)`, and `@multitest fn solve():` when file-top precomputation must run once before the test count is read. For any other shape, write a plain `for _ in 0..tests:` loop; a `@memo fn` nested in it resets on every iteration. Each test case starts with fresh state; shared immutable tables live in `const` or before the case loop. Library modules contain only declarations—importing a helper never reads input or mutates hidden global state.
