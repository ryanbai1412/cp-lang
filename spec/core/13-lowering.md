# 13. Lowering to C++

Status: draft for 0.3.0. This is the contract between the language and
the generated C++17: what each form becomes, what it costs, and what
`cpl explain` reports. Sources: decisions 31, 52, 148, 149; archived
`design/spec/18`; `03-numbers-layout.md` for widths and niches.

## The model in one paragraph

Every program becomes one deterministic C++17 file, aiming under 32 KB
and failing at 64 KB: a stamp, the runtime pieces used, the library
instances used, then `main` on a thread with a large stack. Each form
has a stated C++ shape and cost, so what looks \(O(1)\) is \(O(1)\), and
`cpl explain` shows the choices (widths, niches, specializations). The
lowering adds no undefined behaviour to valid programs.

## 13.1 The output file

- One deterministic C++17 file: a stamp, the used runtime pieces, the
  used library instances, then `main`. Same source and pins give the
  same bytes. Warn above 32 KB (the target for every solution), fail at
  64 KB (profile-adjustable). (R13.1.1)
- `main` starts a thread with the profile's stack (default 1 GB,
  halving down to 8 MB) and runs the program on it. (R13.1.2)
- Input is one bulk `fread` and a hand-written parser; output is one
  buffer written at exit or flush. (R13.1.3)
- Release builds compile with the profile's flags and emit no checks
  (chapter 9). The lowering adds no undefined behaviour to valid
  programs, and avoids it wherever that is free: signed arithmetic that
  may overflow is done in unsigned and cast back, shifts are masked, and
  every array is filled at construction. Unchecked indexes and `x!` on an
  invalid program are as unsafe as the same C++ (§9.2). (R13.1.4, permits)

## 13.2 Values

| Form | C++ |
|---|---|
| `Int`, ranges | `int64_t` in locals; storage at the range's width (`uint8_t` ... `int64_t`) (R13.2.1) |
| `U64`, `I128`, `F64`, `Bool`, `Char` | `uint64_t`, `__int128`, `double`, `bool`, `char` (R13.2.2) |
| position `v: V` | unsigned of `V`'s storage width (R13.2.3) |
| cut | the same width, holding the count before it (R13.2.4) |
| `?T` with a niche | `T`'s storage with the niche value as `None` (R13.2.5) |
| `?T` without one | `T` plus a flag byte, aligned (R13.2.6) |
| tuple, record, struct | a C++ struct, fields in storage order (R13.2.7) |
| enum | niche-filled payload, or tag beside the largest payload (R13.2.8) |
| `[D]T` | pointer plus count, heap buffer; never a stack VLA (R13.2.9) |
| `List<T>`, `Str` | an owned growable buffer (`std::vector`-like runtime type) (R13.2.10) |
| `Mod<P>` constant | `uint32_t` or `uint64_t` with Montgomery or Barrett constants folded (R13.2.11) |
| `Mod(p)` runtime | the same plus constants in the fresh type's static slot (R13.2.12) |
| fresh-type data | one static slot per declaration site, saved and restored across recursive re-entry (R13.2.13) |

Large static arrays (over 4 KB) and every runtime-sized array live on
the heap. Locals of plain types live in registers or the stack.

## 13.3 Functions, closures, generators

- Every generic instantiation is a separate C++ function with a mangled
  name; there are no C++ templates in the output for language generics. (R13.3.1)
- A function value with a known target is passed as an empty struct and
  inlined; captures are passed as references at each call (chapter 6).
  There is no `std::function` and no indirect call. (R13.3.2)
- A non-recursive generator inlines its body with the loop body in place
  of `yield`. A recursive generator becomes a recursive function taking
  the loop body as a parameter; when the body can `break` or `return`, a
  stop flag is threaded through that specialization only. (R13.3.3)
- `@memo(array)` is a lazily allocated dense table of `?R` (jagged when
  bounds depend on earlier parameters); `@memo(map)` is an open-
  addressing table keyed by the parameters' storage. (R13.3.4)
- Views are ordinary structs holding a pointer (lent) or the value
  (owned) plus their closures; walking them is a loop with no
  allocation. (R13.3.5, permits)

## 13.4 Ownership

Ownership has no runtime representation. Lends are pointers or
references, moves are bitwise and leave the source dead (no destructor
runs for a moved-from value), and `.clone()` is an explicit deep copy.
A struct whose fields may be lent or owned is monomorphized on that
choice only when the field's storage differs (a lent `List` is a
pointer, an owned one is the buffer); `cpl explain` lists such copies.

## 13.5 Library specializations

Each library structure documents its representation and every
law-selected specialization, and codegen picks exactly that one:
iterative bottom-up segment trees in one flat array, no push-down when
tags commute, sliding folds by group/selective/general, sparse tables
for idempotent ops, CSR graphs, `Rooted` arrays in preorder. `cpl
explain` names the choice and the law that allowed it.

## 13.6 What `cpl explain` reports

For any program: every type's size, storage order, padding and niche;
every array's element width and peak bytes at the input bounds; total
peak memory against the profile; each specialization and the law or
bound behind it; each check remaining in checked builds and which are
kept in release; each `@cache`, `@trusted` and `@cpp` site; each
generic instance and context copy with its output bytes. The LSP shows
the same data inline (`tools/lsp.md`).

## 13.7 Later

Profile-driven vectorization: a judge profile names the instruction set
(`isa: avx2`), codegen emits the matching `#pragma GCC target`, narrows
arithmetic where ranges allow and vectorizes provable loop shapes;
`explain` says which loops vectorized and why not.

## 13.8 Cost model tables (migrated from design/COST.md)

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
- Anything that looks \(O(1)\) is \(O(1)\); anything \(O(n)\) or allocating looks it.
- No `std::function`, virtual call, hidden heap allocation or hidden
  traversal unless the row says so.
- Debug and release differ only in checks, never in results for valid
  programs.

## Scalars

| Form | Lowers to | Time | Alloc | Debug checks | Release checks |
|---|---|---|---|---|---|
| `Int` arithmetic | `int64_t` ops | \(O(1)\) | none | overflow traps | wraps |
| range type `lo..=hi` storage | narrowest of `int32_t`/`uint32_t`/`int64_t` | \(O(1)\) | none | store into a range-typed place traps if out of range | truncates to the width (R09.3.3) |
| `/`, `%` | floor/Euclidean helper (one extra branch) | \(O(1)\) | none | division by zero traps | defined fallback 0 (R09.3.6) |
| `**` | square-and-multiply | \(O(\log k)\) | none | overflow traps | wraps |
| `Mod<P>` ops | `uint32_t` with constant-divisor reduction | \(O(1)\); `inv` \(O(\log P)\) | none | none | none |
| chained comparison | one comparison per link, operands evaluated once (§130) | \(O(\mathit{links})\) | none | none | none |

## Absence

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `?Int`, `?V`, `?range` | niche value in the same word (`Int` excludes `INT64_MIN`) | \(O(1)\) | none | none | none |
| `?T` for other `T` | value + flag | \(O(1)\) | none | none | none |
| `x!` | load | \(O(1)\) | none | traps on `none` unless flow facts prove it | reads the niche as a value (R09.3.5) |
| `x ?? y`, `?`, `none + x` | branch | \(O(1)\) | none | none | none |

## Index spaces and collections

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `domain V = lo..=hi` | integer; runtime-sized domains read one hidden global slot (saved/restored on recursive re-entry) | \(O(1)\) | none | none | none |
| `D.cuts` cut domain | integer | \(O(1)\) | none | none | none |
| `a[v]` with `v: V`, `a: [V]T` | unchecked load | \(O(1)\) | none | none | none |
| `xs[i]` on `List`/`Str` | load | \(O(1)\) | none | bounds check unless proven | none unless `bounds` is kept (R09.3.2) |
| range `l..r` / cut range `j..i` | two integers; cut range → position range is free (§124) | \(O(1)\) | none | none | none |
| slice `xs[l..r]` | view (`ptr`, `len`) (§123) | \(O(1)\) | none | bounds check unless proven | none unless `bounds` is kept (R09.3.2) |
| `[V]T = x` | one heap array, filled | \(O(n)\) | heap, once | none | none |
| `[f(x) for x in s]` (no filter) | reserved vector + loop | \(O(n)\) | heap, once (reserved) | none | none |
| `xs.push(x)` | amortized vector push | \(O(1)\) amortized | amortized | none | none |
| `xs.sort(by: k)` | `std::sort` with inlined key | \(O(n \log n)\) | none | none | none |
| `for x in xs` / `for v in V` / product domains | plain loop / nested integer loops | \(O(n)\) | none | none | none |

## Binding, control and functions

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `let`, `var`, `let ... else`, `if let` | locals and branches | \(O(1)\) | none | none | none |
| labelled `break`/`continue`, loop `else` (§125) | `goto` / flag-free branch | \(O(1)\) | none | none | none |
| non-recursive generator in `for` | inlined; each `yield` runs the body in place | as written | none | none | none |
| recursive generator in `for` with `break`/`return` (§126) | recursive function, body inlined at the `yield`; one stop branch per frame only when the body can exit | as written + 1 branch/frame | none | none | none |
| closure argument | template functor parameter, inlined (`cpl explain`: static) | \(O(1)\) call | none | none | none |
| recursive lambda / nested `fn` (§127) | self-passing generic lambda | direct call | none | none | none |
| operator value (`by: <`, `+`) | functor, inlined | \(O(1)\) | none | none | none |
| `@memo` on provably small bounds | flat table | \(O(1)\) lookup | heap, once | none | none |

## Mutation and places

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `x op= y` (statement) | in-place op | \(O(1)\) | none | as for the op | as for the op |
| `x min= y` / `max=` as a Bool (§129) | one compare + store | \(O(1)\) | none | none | none |
| `let k = mut a[i]` | reference, index evaluated once | \(O(1)\) | none | none | none |
| `m[k] += 1` on `Map` | one lookup, inserts default (§128) | \(O(1)\) expected | may insert | none | none |
| return of a container | move (§128) | \(O(1)\) | none | none | none |
| `let b = a` (heap value, last use) | move | \(O(1)\) | none | none | none |
| `a.clone()` | deep copy | \(O(\mathit{size})\) | heap | none | none |
| `x.take()` | move out, leave empty | \(O(1)\) | none | none | none |
| `a.swap(i, j)`, `swap(mut a, mut b)` | swap | \(O(1)\) | none | none | none |
| `.absorb(mut other)` | small-into-large drain | \(O(\min \mathit{size})\) | amortized | none | none |

## I/O

| Form | Lowers to | Time | Alloc | Debug | Release |
|---|---|---|---|---|---|
| `read<T>()` | buffered fast reader, typed parse | \(O(\mathit{input})\) | none beyond the value | declared bounds checked | unchecked unless `reads` is kept (chapter 9) |
| `print(...)` | buffered writer | \(O(\mathit{output})\) | none | none | none |

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
