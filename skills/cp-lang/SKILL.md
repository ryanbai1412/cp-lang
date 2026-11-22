---
name: cp-lang
description: Write idiomatic, correct CP Lang v0.3 competitive programming solutions, use the cpl CLI workflow, and navigate the standard library.
---

# CP Lang v0.3

CP Lang compiles indentation-based competitive programming solutions to one
self-contained C++17 file. Types carry contest facts: index domains, value
ranges, optionals, and ownership. Write top-level statements directly (no
`fn main`); add `@!multitest` at the top when the input begins with `t`.

## Loop: Edit, Check, Run, Stress

```bash
cpl check sol.cp                 # fast type, domain, and borrow check
cpl run sol.cp < sample.in       # checked run (bounds, overflow, None traps)
cpl test sol.cp                  # run @test functions
cpl stress sol.cp brute.cp gen.cp --runs 500
cpl build sol.cp                 # release C++17 build (unchecked hot loops)
cpl submit sol.cp                # single-file C++17 ready for a judge
```

Use `cpl explain sol.cp` to inspect inferred widths/layouts, `cpl fmt` to
format, and `cpl lint` for style/complexity warnings.

## Core Idioms

- **Domains vs cuts:** `domain V = 1..=n` makes a fresh index type. Arrays
  `[V]Int` accept only `V`. `Cut<V>` marks boundaries *between* positions
  (`0..=n`); `Range<V>` is a slice of positions (`l..r`, `l..=r`). Convert
  raw ints via `V(x)` or `V.at(x)` (returns `?V`).
- **Optionals (`?T`):** `None` and `Some(x)` replace `-1` and `INF`. Arithmetic
  lifts (`a + b` is `None` if either is `None`). `min`/`max` and `min=`/`max=`
  ignore `None`. Unwrap with `x ?? default` or `x!` (traps on `None`). Print
  optionals with `print(x, none: -1)`.
- **Bool output:** `Bool` does not implement `Print`. Always use `yes(b)`
  (`YES`/`NO`), `yes(b, ("Yes", "No"))`, or `Int(b)`.
- **Input & print:** `read<T>()` dispatches on the expected type and validates
  ranges/domains: `let n = read<Int>()`, `let a = read<[V]Int>()`,
  `let edges = read<[E](u: V, v: V)>()`. `print(a, b)` separates arguments with
  spaces and ends with a newline; sequences print space-separated.
- **Lending, moves, `.clone()`:** Plain scalars copy; heap values (`List`,
  `[D]T`, `Set`, `Map`, structs holding them) move on assignment. Function
  parameters lend by default. Mutation requires `mut` in both signature and
  call site (`f(mut a)`). Use `.clone()` for an explicit heap copy.
- **Generators & O(1) getters:** `fn f() -> yields T:` pushes values with
  `yield x` and composes lazily without allocating; call `.collect()` to build
  a `List`. Parentheses mean work: O(1) `@get` properties have no `()`
  (`a.len`, `a.first`, `st.all`), while O(n)/allocating calls have `()`
  (`a.sorted()`, `a.sum()`).
- **Memoization & `@cache`:** `@memo(array)` caches recursive functions over
  bounded domains/ranges in flat arrays; `@memo(map)` handles general keys.
  `@cache` on a struct field lazily computes and memoizes derived data.

## C++ Habit to CP Lang Rewrite

| C++ habit | CP Lang v0.3 |
| --- | --- |
| `int main() { cin >> n; }` | Top-level statements; `let n = read<Int>()` |
| `vector<int> a(n);` with `0..n` | `domain I = 0..n; var a: [I]Int = 0` |
| `const long long INF = 1e18;` | `var dp: [V]?Int = None` + `dp[v] min= d` |
| `cout << (ok ? "YES" : "NO");` | `print(yes(ok))` |
| `cout << (ans == INF ? -1 : ans);` | `print(ans, none: -1)` |
| `void f(vector<int>& a)` / `f(a)` | `fn f(mut a: List<Int>):` / `f(mut a)` |
| `auto b = a;` (copy vector) | `let b = a.clone()` (assignment moves) |
| `sort(a.begin(), a.end(), cmp)` | `a.sort(by: x => desc(x))` or `a.sorted()` |
| `vector<vector<int>> g(n)` | `let g = Graph.undirected(edges)` |
| `# comment` / `/* ... */` | `// comment` (spaces only, no tabs) |
| `for (int i = 0; i < n; i++)` | `for i in 0..n:` or `for v in V:` |
| `if (x)` on int/optional | `if x != 0:` / `if let Some(v) = x:` |

## Common Mistakes & Error Codes

- `E0001` Tabs in indentation -> use 4 spaces.
- `E0004` `#` is not a comment -> use `//`.
- `E0110` Unparenthesized `??` mixed with other operators -> write `(a ?? 0) + 1`.
- `E0208` Uppercase value/function name -> values/fns are `snake_case`, types/domains `PascalCase`.
- `E0301` Wrong domain index -> index `[V]T` only with `V`, not `Int` or another domain.
- `E0307` Non-`Bool` condition -> write `if x != 0:` or `if let Some(v) = opt:`.
- `E0308` Ordering on `?T` -> unwrap first (`(a ?? 0) < b`) or use `min`/`max`.
- `E0401` Assigning to `let` -> declare with `var`.
- `E0402` Missing `mut` at call site -> pass `f(mut x)` to a `mut` parameter.
- `E0601` / `E0602` Conflicting mutable lend or write while lent -> finish using views/generators before mutating.
- `E0606` Printing `Bool` directly -> use `print(yes(b))`.
- `E0608` Implicit copy of a heap value -> move once or call `.clone()`.
- `L0608` Getter called with `()` or O(n) method missing `()` -> `a.len`, `a.sorted()`.
- `R0301` Runtime trap unwrapping `None` with `!` -> guard with `if let Some(v) = ...` or `??`.

## Supplements (Read When Needed)

- `io.md` — `read<T>()`, record/graph readers, `print`, `yes()`, `none:`, `@!multitest`, interactive flushing.
- `domains-and-cuts.md` — `domain`, positions vs `Cut<D>`, `Range<D>`, `[D]T`, 2D arrays, `Pair<D>`, `Subset<D>`, binary search.
- `ownership.md` — moves, lends, `mut` params, `.clone()`, views, `@cache`, avoiding `E0601`/`E0602`/`E0608`.
- `functions-and-generators.md` — closures, `-> yields T`, lazy pipelines, `.collect()`, `@memo(array)`/`@memo(map)`.
- `errors.md` — reading diagnostics and fixing top `E`/`W`/`L`/`R` codes.
- `debugging-and-stress.md` — `cpl run`, `cpl test`, `cpl stress`, `assert`, `debug`, `unchecked:`, judge builds.
- `idioms.md` — worked before/after CP Lang solutions (DP, graphs, binary search, data structures).
- `stdlib/README.md` — index of all 39 standard-library modules (`stdlib/<module>.md`).
