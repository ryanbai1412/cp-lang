---
name: cp-lang
description: Write idiomatic, correct CP Lang v0.3 competitive programming solutions, use the cpl CLI workflow, and navigate the standard library.
---

# CP Lang v0.3

CP Lang compiles to one C++17 file. Write top-level statements (no `main`).
Good CP Lang says the editorial's idea in the editorial's words, states every
fact once, and its obvious reading is the correct and fast one. Do not
translate C++ line by line: ask how the idea is best said here.

## Workflow (always)

1. Write `sol.cp`. Read constraints as types: `read<1..=2e5>()`.
2. `cpl check sol.cp` until clean. Every error names the fix; `cpl fix` applies it.
3. `cpl run sol.cp < sample.in` on every sample (checked: traps on bad index,
   overflow, `None`).
4. Add a brute `@alt(program) fn` in the same file; `cpl stress sol.cp` until
   1000 cases agree. A failure is shrunk and saved to `sol.tests/`.
5. `cpl lint sol.cp`; apply what it suggests unless clarity suffers.
6. `cpl build sol.cp` writes the C++ to submit (`--judge <name>`, `cpl judges`).

`R0402 todo reached` means a stdlib body is unwritten: write that piece in the
solution and say so. Never paper over a checker error with casts.

## The shape of a solution

```cp
let n = read<1..=2e5>()
domain V = 1..=n
let a = read<[V]Int>()
let p = Prefix(a)
print(p.fold(V(1)..=V(n)))
print(V.first_true(v => a[v] > 2), none: -1)
print(yes(a.all(x => x > 0)))
// input: 4
// input: 1 3 5 2
// output: 11
// output: 2
// output: YES
```

```cp
let n = read<1..=8>()
domain V = 1..=n
let a = read<[V]-5..=5>()
print(V.all.map(v => a[v]).max()!)

@alt(program)
fn brute():
    let n = read<1..=8>()
    let a = read<[n]-5..=5>()
    var best = a[0]
    for x in a: best max= x
    print(best)
// input: 3
// input: 1 -2 3
// output: 3
```

## Core syntax

Python-shaped blocks: `if`/`elif`/`else`, `for x in xs`, `while`, `and`/`or`/
`not`, `match`. One-line `if c: a elif d: b else: e` is an expression.
`x op= e` exists for every operator plus `min=`/`max=`, which as a condition
are true when the target improved.

```cp
let n = read<1..=100>()
let kind = if n % 15 == 0: "FizzBuzz" elif n % 3 == 0: "Fizz" else: "-"
print(kind)
var best = 0
for x in [3, 9, 4]:
    if not (x % 2 == 0) and best max= x: print(x)
// input: 9
// output: Fizz
// output: 3
// output: 9
```

## Practices

1. **Name the algorithm.** If the stdlib has it (`Prefix`, `SegTree`, `Dsu`,
   `first_true`, `compress`, `Graph`, `Mod<P>`), call it; don't hand-roll.
   Each `stdlib/<module>.md` says what is "In prelude"; anything else needs
   `use std.<module>.{name}` (the E0201 help names the line).
2. **Whole-sequence ops for reductions, searches and builds** (`sum`, `max`,
   `count(p)`, `all`, `sorted`, `[V] v => ...`); plain loops for real algorithms.
3. **Name an index space** (`domain V = 1..=n`) when arrays share it or two
   spaces could be confused (vertices vs edges, rows vs columns). Read 1-based
   input straight into it: `read<[V]Int>()`, `read<[m](u: V, v: V)>()`.
4. **Positions step, cuts bound.** `.prev`/`.next` keep the kind (`?V`);
   `.l`/`.r` switch between a position and its cuts. `v.after`, `v.before`
   are the positions after/before `v`. `v + 1` is an `Int`; get back with
   `v.next`, `V(x)` (traps) or `V.at(x)` (`?V`). Ranges `l..=r` of positions.
5. **Absence is `?T`, never `-1`/`INF`.** `+` lifts through `None`; `min=`/
   `max=` ignore it; unwrap with `??`, `if let x = opt:` or `x!` (traps).
6. **Constraints once, as types:** ranges and domains give width, overflow
   proofs and input validation.
7. **One home per fact:** derive, don't duplicate (`V.count`, not `n` twice).
8. **Values by default.** `let` unless reassigned; `mut` at both ends for
   in-place changes; heap values move, `.clone()` copies.
9. **Per-test state lives under `@!multitest`** (input starts with `t`); bindings
   above it (a sieve, a `Comb` table) are built once and shared by every test.
10. **Costs are visible:** no-paren members are O(1) (`a.len`, `v.next`);
    work has parens (`a.sorted()`, `a.sum()`).
11. **Print what you mean:** `yes(b)` for Bool (`answers: ("Yes", "No")`, or
    `with io(answers: ...)` once); `print(x, none: -1)`; `sep:`, `end:`.
12. **Evidence beside the solution:** `@alt` brutes, `@gen`, `@check`, `assert`.
13. **One spelling per idea:** options are named args (`by:`), not `_by` twins.
14. **Escape hatches explicit, rare, local** (`unchecked:`, inline C++).
Clear beats short: never trade readability for tokens.

## C++ habit → CP Lang

| C++ | CP Lang |
| --- | --- |
| `cin >> n; vector<ll> a(n)` | `let a = read<[n]Int>()` or `read<[V]Int>()` |
| `dist(n, INF)`, `== INF ? -1` | `var d: [V]?Int = None`; `print(d, none: -1)` |
| `cout << (ok ? "YES" : "NO")` | `print(yes(ok))` |
| `while (lo < hi) { mid = ... }` | `(lo..=hi).first_true(x => ok(x))` |
| `pre[i+1] = pre[i] + a[i]` | `let p = Prefix(a)`; `p.fold(l..=r)` |
| `sort + unique + lower_bound` | `let (domain X, xs) = compress(vals)` |
| `auto b = a;` | `let b = a.clone()` (assignment moves) |
| `void f(vector<int>& a)` | `fn f(mut a: List<Int>):`, call `f(mut a)` |
| `p[find(a)] = find(b)` | `var d = Dsu<V>()`; `d.unite(a, b)`; `d.same(a, b)` |
| `x % MOD` everywhere | `Mod<P>` values: `z ** k`, `z.inv` |
| `for (i = 0; i < n; i++) cnt += p(a[i])` | `a.count(x => p(x))` |
| `find_if(a.begin(), a.end(), p)` | `a.first(x => p(x))` (`?T`; sorted or monotone: `first_true`) |

## Traps that compile

- `count(p(x) for x in xs)` counts every element. Write `xs.count(x => p(x))`.
- `print(true)` is an error (E0606): Bool has no `Print`; use `yes(b)`.
- `m[k]` on a missing `Map` key traps; use `m.get(k) ?? 0`.

## Supplements (read when needed)

- `io.md`: readers, records, `print` options, `yes`, multitest, interaction.
- `domains-and-cuts.md`: domains, positions, cuts, ranges, `[R, C]T`, binary search.
- `ownership.md`: moves, lends, `mut`, `.clone()`, views, `@cache` fields.
- `functions-and-generators.md`: closures, `-> yields T`, `@get`, `@memo(array|map)`.
- `errors.md`: reading diagnostics; top `E`/`W`/`L`/`R` codes and fixes.
- `debugging-and-stress.md`: `@alt`, `@gen`, `@check`, `cpl stress`, traces, `cpl dap`.
- `idioms.md`: worked before/after solutions (DP, graphs, search, structures).
- `stdlib/README.md`: every module with signatures, costs and traps.
