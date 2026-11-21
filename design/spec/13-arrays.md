# 13. Arrays over index spaces

**Layer:** core C7. Status: [proposed].

Core here: the `Index` trait as the interface `[D]T` requires, array layout, and `[(A, B)]T ≡ [A][B]T`.
Every `Index` impl (counted, `labels`, `Cut`, `ring`, `grid`, `Bits`, integer ranges) is library (`21`).

## 13.1 `[D]T`

**What.** `[D]T` is an array with one `T` per position of `D`, for any `D: Index`. Indexing takes a `D`.

**Why.** Most WA/RE bugs are an integer used as the wrong kind of index. If the array's index type is part of
its type, `deg[e]` with `e: E` is a type error, and no bounds check is needed when the index type can't be
out of range.

**Layer.** Core: the type former, indexing, and layout. What `D` can be is entirely the `Index` trait.

**Cost.** (pointer, length) on the heap; `D.count` elements, dense. Indexing with a `D` is one load, no
check. Statically sized arrays ≤ 4 KB are inline (chapter 18).

**Tooling.** The tracer draws every array with its index space; the LSP shows the index type of every
subscript.

**Rules.**
- `[n]T` with a bare length is indexed by the integer range `0..n` and bounds-checked (trap) [r8 2]. Only
  named domains are nominal. Each anonymous array has its own `xs.Cut`.
- `[len: 1..=n]T` in a read means "a length from the input, then that many items" [r12 R10]. A lint flags
  `[1..=n]T` in read position, one token away from it [r9 15].
- `D.map(f)` builds a `[D]T`; a domain counts as the array of its positions [r4 5].
- `.list()`, `return` and assignment from a temporary move the buffer; `.list()` on a named array copies, and
  `cp explain` reports it [r13 P5].

**Rejected.** Plain `vector<T>` with integer indices (the bug class); per-access bounds checks on nominal
indices (unneeded).

## 13.2 The `Index` trait

**What.**

```cp
trait Index:
    fn count() -> Int
    fn ord(self) -> 0..Self.count
    fn from_ord(i: 0..Self.count) -> Self
```

**Why.** One interface makes `[I]T`, `for x in I`, `SegTree<I, …>`, `Bits<I>` and `@memo` tables work for
every index space, user-defined ones included.

**Layer.** Core knows the trait (it's the bound on `[D]T`). Implementations are library. `D.at(i)` and
`D.get(i)` are library wrappers over `from_ord` [r7 18].

**Laws.** `from_ord(ord(x)) == x`, checked by `cp test` on random values. `from_ord` is total on
`0..count`.

**Rejected.** Special-casing domains in the compiler: then `Cut`, `ring` and `grid` would each need
compiler support.

## 13.3 Products and multi-dimensional arrays

**What.** A tuple of index spaces is an index space, and `[(A, B)]T` is the same type as `[A][B]T`
(diagnostics print `[A][B]T`). `t[(a, b)]` and `t[a][b]` both work. Iteration is `product(A, B)` [p2 Q1,
r14 T12–T13].

**Why.** DP states, grids and `@memo` keys are products; one rule covers all three.

**Layer.** Core: the type equality and layout. The `Index` impl for tuples is library.

**Cost.** One dense row-major buffer. `g[i]` is a fixed-length row slice, so rows are never invalidated. Binding a row with `var r = g[i]` copies it like any named
heap value (16.2); `let r = g[i]` is a read-only view [2impl Q44].
[p1 S6, r13 P25]. A product position is one ordinal; unpacking is a shift for static powers of two and a
Barrett constant for runtime inner counts, reported by `cp explain` [r16 R21].

**Rejected.** `vector<vector<T>>` (one allocation per row); a separate `Grid` type in core (it's library:
`21`).

## 13.4 Inline storage

- **Storage:** `[D]T` whose `D.count` is static and whose size is ≤ 4 KB is stored inline, like a C array: on the
  stack, in the enclosing struct, or in a segment-tree node. Larger static and all runtime-sized arrays use the
  heap. A read array whose `len:` bound is a literal ≤ 8 is inline. On the stack, inline storage is used in every function: the stack is big (chapter 18.3). `.list()` on an inline array copies, and the
  large-copy lint covers inline values. `cp explain` shows which.
