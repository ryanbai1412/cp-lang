# Cost model

Every core form, what it lowers to in C++17, what it costs, and what each
build checks. "Costs are visible" is checked here: each row names a test
snippet under `tests/cost/` whose emitted C++ and
`cpl explain` output must match the row. A row without a passing snippet
is unverified; a form whose real lowering is worse than its row
is a bug, and a deliberate exception is listed under Debt.

Status: no row has its snippet yet, so every row is a target, not a
measurement. Rows land verified one at a time.

Rules every row follows:
- Anything that looks O(1) is O(1); anything O(n) or allocating looks it.
- No `std::function`, virtual call, hidden heap allocation or hidden
  traversal unless the row says so.
- Debug and release differ only in checks, never in results for valid
  programs.

## Scalars

| Form | Lowers to | Time | Alloc | Debug checks | Release checks |
|---|---|---|---|---|---|
| `Int` arithmetic | `int64_t` ops | O(1) | none | overflow traps | wraps |
| range type `lo..=hi` storage | narrowest of `int32_t`/`uint32_t`/`int64_t` | O(1) | none | store into a range-typed place traps if out of range | same |
| `/`, `%` | floor/Euclidean helper (one extra branch) | O(1) | none | division by zero traps | same |
| `**` | square-and-multiply | O(log k) | none | overflow traps | wraps |
| `Mod<P>` ops | `uint32_t` with constant-divisor reduction | O(1); `inv` O(log P) | none | none | none |
| chained comparison | one comparison per link, operands evaluated once (§130) | O(links) | none | none | none |

## Absence

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `?Int`, `?V`, `?range` | niche value in the same word (`Int` excludes `INT64_MIN`) | O(1) | none | none | none |
| `?T` for other `T` | value + flag | O(1) | none | none | none |
| `x!` | load | O(1) | none | traps on `none` unless flow facts prove it | same |
| `x ?? y`, `?`, `none + x` | branch | O(1) | none | none | none |

## Index spaces and collections

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `domain V = lo..=hi` | integer; runtime-sized domains read one hidden global slot (saved/restored on recursive re-entry) | O(1) | none | none | none |
| `D.cuts` cut domain | integer | O(1) | none | none | none |
| `a[v]` with `v: V`, `a: [V]T` | unchecked load | O(1) | none | none | none |
| `xs[i]` on `List`/`Str` | load | O(1) | none | bounds check unless proven | same |
| range `l..r` / cut range `j..i` | two integers; cut range → position range is free (§124) | O(1) | none | none | none |
| slice `xs[l..r]` | view (`ptr`, `len`) (§123) | O(1) | none | bounds check unless proven | same |
| `[V]T = x` | one heap array, filled | O(n) | heap, once | none | none |
| `[f(x) for x in s]` (no filter) | reserved vector + loop | O(n) | heap, once (reserved) | none | none |
| `xs.push(x)` | amortized vector push | O(1) amortized | amortized | none | none |
| `xs.sort(by: k)` | `std::sort` with inlined key | O(n log n) | none | none | none |
| `for x in xs` / `for v in V` / product domains | plain loop / nested integer loops | O(n) | none | none | none |

## Binding, control and functions

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `let`, `var`, `let ... else`, `if let` | locals and branches | O(1) | none | none | none |
| labelled `break`/`continue`, loop `else` (§125) | `goto` / flag-free branch | O(1) | none | none | none |
| non-recursive generator in `for` | inlined; each `yield` runs the body in place | as written | none | none | none |
| recursive generator in `for` with `break`/`return` (§126) | recursive function, body inlined at the `yield`; one stop branch per frame only when the body can exit | as written + 1 branch/frame | none | none | none |
| closure argument | template functor parameter, inlined (`cpl explain`: static) | O(1) call | none | none | none |
| recursive lambda / nested `fn` (§127) | self-passing generic lambda | direct call | none | none | none |
| operator value (`by: <`, `+`) | functor, inlined | O(1) | none | none | none |
| `@memo` on provably small bounds | flat table | O(1) lookup | heap, once | none | none |

## Mutation and places

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `x op= y` (statement) | in-place op | O(1) | none | as for the op | as for the op |
| `x min= y` / `max=` as a Bool (§129) | one compare + store | O(1) | none | none | none |
| `let k = mut a[i]` | reference, index evaluated once | O(1) | none | none | none |
| `m[k] += 1` on `Map` | one lookup, inserts default (§128) | O(1) expected | may insert | none | none |
| return of a container | move (§128) | O(1) | none | none | none |
| `let b = a` (heap value, last use) | move | O(1) | none | none | none |
| `a.clone()` | deep copy | O(size) | heap | none | none |
| `x.take()` | move out, leave empty | O(1) | none | none | none |
| `a.swap(i, j)`, `swap(mut a, mut b)` | swap | O(1) | none | none | none |
| `.absorb(mut other)` | small-into-large drain | O(min size) | amortized | none | none |

## I/O

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `read<T>()` | buffered fast reader, typed parse | O(input) | none beyond the value | declared bounds checked | same |
| `print(...)` | buffered writer | O(output) | none | none | none |

## Debt (forms that cost more than their row)

| Form | Today | Target | Owner |
|---|---|---|---|
| closures stored in struct fields, recursive local closures, fn params of recursive fns, reassigned fn slots | `std::function` | template parameter or self-passing lambda | codegen |

## Enforcement

Each row gets one snippet `tests/cost/<form>.cp` plus assertions on its
emitted C++ (e.g. no `std::function`, no allocation inside the loop, the
expected integer width) and on `cpl explain --json`. `scripts/check.sh`
runs them; a new core form lands with its row and snippet in the same
commit.
