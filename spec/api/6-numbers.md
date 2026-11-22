# Round 6: numbers, widths and niches

Status: proposal for review. Scope: how numbers stay small in memory
without the contestant thinking about bytes, and how `?T` costs nothing
for ints, structs, records, enums and arrays. Modular arithmetic, floats
and overflow semantics are summarized where they touch layout; their
detailed rules stay in `design/spec/11-types.md`.

## The model in one paragraph

**Values compute in 64 bits; storage takes the fewest bytes the type
allows.** A local `Int` lives in a register, where width is free, so
locals and arithmetic are always `Int` (i64) and nobody tunes them.
Bytes matter only where values are stored many times: arrays, lists,
struct fields, tables inside library structures. There, the storage
width comes from the type you already wrote for a different reason: a
domain (`[V]T`, `V` positions), a range from the statement
(`read<1..=1e9>`), a `Mod<P>`, a `Bool`. Every type also has a set of
**spare values** it never holds (`Int.MIN`, `hi + 1` for a range, `P` for
`Mod<P>`, 2 for `Bool`); `?T` stores `none` in one of them, so it is as
wide as `T`. A struct's spare values are its fields' spare values, so
`?Edge` is as wide as `Edge` with no annotation. Both choices are
automatic only because they never lose (same operations, never more
bytes, never slower), and `cpl explain` prints every layout.

## 1. Where widths come from

| Written | Storage | Why it is the type you'd write anyway |
|---|---|---|
| `Int`, no bound | 8 bytes | the default; nothing is guessed |
| `lo..=hi` (static ends) | narrowest of u8/u16/u32/u64 (i8.. when `lo < 0`) that holds the range; `?` adds a byte only when no spare is left (`0..=255`) | statement bounds: `read<[V]1..=1e9>` |
| `0..=V.count`, `1..=V.count` | as `V`'s positions | counts, sizes, depths |
| a domain position `V` | narrowest that holds `V.count + 1` values | every index |
| `Mod<P>` | u32 when `P < 2^31`, else u64 | the modulus |
| `Bool` | 1 byte; `Bits<V>` when you ask for bits | |
| `Char` | 1 byte, ASCII `0..=127` | strings and grids |
| `U64`, `I128` | exactly that | hashes and bit tricks (wrapping `U64`); products past i64 (`I128`) |
| `F64` | 8 bytes | |

- **Domain widths follow the static bound of the count.** `let n =
  read<1..=2e5>(); domain V = 0..n` gives 4-byte positions; `n: 1..=5000`
  gives 2-byte positions. Every `[V]V` parent array, `List<V>` stack and
  `[V]0..=V.count` size array shrinks with it.
- **Locals never narrow.** `let x: 0..=1e9 = ...` is a contract checked
  at assignment, but it lives in an i64 register. Narrowing a local saves
  nothing and costs conversions.
- **Struct fields need static bounds** (unchanged): layout never depends
  on input. A field may be `V` (a domain position), since the domain's
  width bound is static.
- **Loads widen, stores check.** Reading `a[i]` from a `[V]0..=1e9` is
  one zero-extending load; arithmetic is in `Int`, so `a[i] * a[j]` can't
  overflow 32 bits. Storing into a range-typed slot checks the range
  unless the checker proves it (`a[i] = a[i] % 7` needs no check). A range
  type is a contract: `cpl run` and checked builds trap on a violation;
  release builds don't check (decision 100), the same deal as `Int`
  overflow, so valid programs pay nothing on the judge.
- **No `I32`, `U32` or `U8`.** A narrow slot is a range: `[V]0..=255` is
  one byte per element, `[V]0..=4e9` four. Decision 1.7 keeps the number
  types to `Int`, `U64`, `I128` and `F64`; a byte of text is `Char`.
- **Narrow arithmetic when proven.** Semantically every operation is in
  `Int`. When the range proof shows an expression and its operands fit in
  32 bits (`a[i] + b[i]` with both `0..=1e9` is at most `2e9`, a u32), the
  C++ computes it in 32 bits. Same answer, never slower, and twice the
  lanes when the loop vectorizes, so it majorizes; `cpl explain` marks it.

Why this split: in registers, 64-bit `+ - * < &` cost the same as 32-bit
on every judge CPU; only division by a runtime value is slower on older
Intel. Width costs in memory (half the values per cache line, MLE) and in
SIMD (4 i64 lanes vs 8 i32 per 256-bit register). So locals stay `Int`,
storage narrows, and vectorizable arithmetic narrows only when proven.

### Ops decide what a structure stores

Generic structures store their element type, so the narrow width flows
through `SegTree`, `Heap`, `List`, `Dsu` with no extra syntax. The fold op
decides the element type of the folds:

| Structure | `a: [V]0..=1e9` stores | Why |
|---|---|---|
| `SegTree(max, a)` | 4-byte nodes | `max` is `Selective`: the result is one of the inputs, so it stays in `0..=1e9` |
| `SegTree(+, a)` | 8-byte nodes | `+` is not closed on a range; with `n <= 2e5` sums reach `2e14`, so `T` is `Int` |
| `SegTree(+, b)`, `b: [V]1..=1e4`, `n <= 2e5` | 4-byte nodes | a node sums at most `V.count` inputs: `0..=2e9`, proven |
| `SegTree<0..=4e9>(+, a)` | 4-byte nodes | you named the node type; node writes are range stores |
| `SegTree(+, a.map(Z))` | 4-byte nodes | `Mod<P>` is closed under `+` |
| `Prefix(+, a)` | `[V.cuts]Int` | sums |
| `Heap(by: .d)` of `(d: Int, v: V)` | 16 bytes (12 + padding) | each field at its own width |

This is the law machinery of round 5 doing layout work: `Selective`
already promised "returns one of its operands", which is exactly the fact
that keeps a range closed. `+` gets the same treatment from its interval
rule: with `N` the static bound of `V.count`, a fold of at most `N`
inputs in `lo..=hi` lies in `min(0, N*lo) ..= max(0, N*hi)`, and the node
type narrows when that fits. The bound comes from the static bound of `V.count`, so it
holds for every input; with `act:` or point writes of unbounded values
there is no bound and nodes stay `Int`.

To force a width, name the type parameter: `SegTree<0..=4e9>(+, a)`. Every
node write is then a range store under decision 148: checked builds trap
with the node and the value, release truncates.

### Seeing and steering it

- `cpl explain` lists every array, list and structure with its element
  layout and its peak bytes at the input bounds, e.g. `dist: [V]?Int, 8 B,
  1.6 MB at n = 2e5`.
- `cpl build` warns when the static peak exceeds the judge profile's
  memory limit, naming the three largest arrays.
- **Lint `wide_array`**: an array with a static size of at least 1e6
  whose element type is `Int` while every value written into it has a
  narrower static range (a read, a domain position, a constant, `min`/`max`
  of those). It suggests the annotation, e.g. `[V]0..=1e9`. Purely local:
  it looks only at the writes in the same function.
- Override with an explicit type in either direction: `[V]Int` widens,
  `[V]0..=4e9` stores 4 bytes.

## 2. Spare values

Every type has a fixed set of **spare values**: bit patterns of its
storage that no valid value uses. The set is a property of the type, never
of a use, so two `?Edge`s always agree on layout.

| Type | Spare values | `none` is |
|---|---|---|
| `Int` | `INT64_MIN` (`Int` is `-(2^63-1)..=2^63-1`) | `INT64_MIN` |
| `lo..=hi` in width W | every W value outside `lo..=hi` | `hi + 1` if it fits, else `lo - 1` |
| domain position `V` | indices `>= V.count` | the all-ones pattern |
| `Mod<P>` | `P ..` up to the width | `P` |
| `Bool` | 2..=255 | 2 |
| unit `enum` with k variants | k..=255 | k |
| `F64` | quiet-NaN payloads (NaN traps in checked runs) | one fixed payload |
| `Char` | 128..=255 | 128 |
| `U64`, `I128` | none | needs a flag |
| `?T` | `T`'s spares minus the one `none` took | |

### Structs, records and tuples

**`?S` borrows the first field, in declaration order, that has a spare
value.** It writes `none` into that field and leaves the others
undefined. So a struct of `Int`s has a niche because `Int` has one:

```python
struct Edge: to: V; w: Int          # 4 + 8 -> 16 B (padded)
let best: [V]?Edge = none           # 16 B per slot, not 24
record (lo: Int, hi: Int)           # ?(lo, hi) is 16 B
```

- Records and tuples follow the same rule, since they are structs without
  names.
- Nesting is free while spares remain: `?(?Int)`, a `[V]?Edge` inside a
  `?Paths`, `Dsu` parents stored as `[V]?V`.
- **No user `Niche` trait** (decision 28.10). A library could name a value
  its own code later produces, turning a wrong `none` into silent
  corruption. Every struct that has a spare value already has one in a
  field, and a range type on one field is how you give it one:
  `struct H: h: U64; len: 0..=1e6` puts `?H` in `len`.

### Enums with payloads

The tag goes into a spare value when that is free; otherwise it is a
byte:
- **One payload variant plus k unit variants** (`Option`-like, `enum Cell:
  Wall, Empty, Box(id: V)`): the unit variants take k spare values of the
  payload. `Cell` is 4 bytes.
- **Several payload variants:** the largest variant (first on a tie) sets
  the layout. Every other variant is one spare value of a single field of
  it, the first field in declaration order with enough spares, provided
  that variant's own fields fit in the bytes outside that field. `enum Ev:
  Add(v: V), Ask(l: V, r: V)` is 8 bytes: `Add` keeps `v` in `l`'s slot and
  writes all-ones into `r`'s slot, so a valid `r` means `Ask`. `?Ev` takes
  another spare of `r` and is also 8.
- **Otherwise a tag byte** beside the largest payload, rounded up to
  alignment: when no field has enough spares (every bit pattern is a valid
  value) or a smaller variant can't fit around the field holding them.
- The rule is Rust's niche filling, restricted to "first field with enough
  spares", so it is predictable without reading the compiler; `cpl explain`
  prints the encoding.

### Arrays of optionals

`[V]?T` is `[V]T` with spares wherever `T` has one: the distance array
`[V]?Int` is 8 bytes per vertex, `[V]?V` parents 4 bytes. When `T` has no
spare (`[V]?U64`), the flag is stored inline next to each element. The
compiler never splits it into a side bitset on its own: that would be
smaller but costs a second load per access, so it doesn't majorize. Ask
for it with `Opt<V, T>` (a library struct of `[V]T` plus `Bits<V>`) if
memory is the limit.

### Field order

Storage order is chosen to minimize padding (largest alignment first,
then declaration order); visible order (constructors, patterns, `print`,
derives, `..R` rows) stays declaration order. Nothing in the language
observes addresses, so reordering never changes behaviour, and it only
removes padding. `cpl explain` shows the storage order.

### Cache layout

Width is half of cache behaviour; layout is the other half. The defaults
already favour locality: narrow storage packs more values per 64-byte
line, `Graph` is CSR (a vertex's arcs are contiguous), trees reorder by
`t.Pre` (subtrees are ranges, heavy chains are runs), segment trees are
one flat array, and niches mean `?T` never costs a second miss on a flag
array.

Padding stays when a struct's fields don't fill its alignment
(`(d: Int, v: V)` is 12 bytes of data in 16), as in C++. Two explicit
fixes, neither automatic because neither is never-worse:

| Spelling | Bytes | Trade |
|---|---|---|
| `Packed<S>` (decision 1.2) | data only | unaligned loads; some elements straddle a line |
| `layout: columns` on `[V]S`, `List<S>`, `Heap<S>` | data only | one array per field: a scan or sift touching one field reads only that column; touching all fields reads several lines |

`cpl explain` flags any structure above 1 MB whose padding exceeds 20% of
its element and names both fixes.

## 3. Operations on niche types

- **Testing `none` is one compare** with the fixed pattern.
- **`max=` on `?Int` is branch-free**: `INT64_MIN` is below every `Int`,
  so `none` is already the identity of `max` (the tropical rule in
  decision 1.3). `min=` on `?Int` compiles to one compare and a
  conditional move.
- **Producing a spare value is an overflow.** `Int` arithmetic that lands
  on `INT64_MIN` traps in debug like any overflow; in release it is part
  of the wrap contract (a wrapped value may read as `none`). An
  out-of-range store is the same: checked builds trap before it writes a
  spare; release truncates, and the result may read as `none`.
- **Reading input** rejects a value equal to a spare (`read<Int>` of
  `-9223372036854775808` is an input error naming the bound).
- **Generic code** sees no difference: `?T` inside `SegTree<T>` or a user
  struct gets `T`'s layout after monomorphization.

## 4. Overflow, in one table

Ranges carry the proofs; the build decides what an unproven step does.

**The rule for every check.** On the judge only valid programs matter, so
release optimizes for them: no runtime check is on by default, and a wrong
program gets a wrong answer. Checked builds (`cpl run`, stress, `-DLOCAL`)
trap on every violation, so bugs show up before submission. The emitted
C++ still has no undefined behaviour (wrapping goes through unsigned
casts, which compile to the same instructions), so `-O2` never
miscompiles the valid paths around a bug. To keep a check on the judge,
opt in: `with checks(ranges: on, overflow: on)` on a block or file, or
`release_checks:` in the judge profile.

| Situation | Debug and `cpl run` | Release on the judge |
|---|---|---|
| `Int` arithmetic proven in range | no check | no check |
| `Int` arithmetic unproven | trap with values | wraps |
| store into a range type, unproven | trap | no check: truncates to the width |
| `U64` | wraps | wraps |
| `I128` arithmetic unproven | trap with values | wraps |
| `wrapping_add` and friends | wraps | wraps |
| result range exceeds i64 (`1..=1e9` cubed) | the result is plain `Int`; overflow checks apply | |

Proofs are intraprocedural interval arithmetic on `let`s and declared
ranges (already decided); nothing crosses calls, so compile time stays
linear.

## 5. Modular numbers

`Mod<P>` is stored in `0..P`, so it is 4 bytes for every common prime,
`?Mod<P>` is free (`P` is spare) and `SegTree(+, a.map(Z))` stores 4-byte
nodes. Multiplication widens to u64 (or `I128` for moduli above `2^32`,
which the profile must allow). A runtime modulus is a scoped type
(`Mod<p>` with `p` read once) using Barrett reduction, one local edit
from the constant form.

## 6. Examples

```python
let n = read<1..=2e5>()
domain V = 0..n
let a = read<[V]1..=1e9>()           # 4 B per element
var par: [V]?V = none                # 4 B, none = all ones
var dist: [V]?Int = none             # 8 B, none = INT64_MIN
var mx = SegTree(max, a)             # 4 B nodes: max is Selective
var sm = SegTree(+, a)               # 8 B nodes: sums are Int
struct Item: w: 1..=1e9; v: 1..=1e9; id: V    # 12 B, ?Item 12 B
```

`cpl explain` on this prints, per line, the storage and the reason
("`a`: u32, from `1..=1e9`; spare `1e9 + 1`").

## 7. What this needs

- Core: storage width for static range types, domain positions and
  domain-relative ranges (implemented for 1D and nested arrays); niche
  borrowing for structs (28.10, implemented for `Int` and position
  fields); spare-value niches for enums, `Bool`, `Mod`, ranges
  and records (codegen debt); padding-minimizing field order (new).
- Library: `Opt<V, T>` (new), `Bits<V>` (exists).
- Tools: `cpl explain` layouts and peak memory, the `wide_array` lint,
  the build-time memory warning.

## Later

- **Vectorization by judge profile.** A profile names the instruction set
  (`isa: avx2`); codegen emits the matching `#pragma GCC target` and, for
  loops whose shape it can prove (narrow-width element loops, `Group` and
  `Selective` folds, prefix scans), vectorizes them itself. `cpl explain`
  says which loops vectorized and why the others didn't. Not designed yet.

## Open

1. **Automatic field reordering.** It only removes padding, but C++
   interop through `@cpp` would see the storage order. Proposal: reorder,
   and `@cpp`-visible structs keep declaration order.
2. **`wide_array` threshold.** 1e6 cells is a guess; the goal is to flag
   the arrays that decide MLE and stay quiet otherwise.
