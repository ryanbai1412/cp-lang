# How std gets its powers

| | |
|---|---|
| Kind | Design |
| Owner | stdlib |
| Status | Active |
| Describes | `std/*.cp`, the compiler features they use, and the remaining compiler-known traits |

std is ordinary CP Lang. Every power a std module uses is a public language
feature that a user library can use the same way. This page lists those
features, with the std type that shows each, and the few places where the
compiler still knows a std name.

## By example: `Subset<B>`

```cp
pub struct Subset<B>:
    bits: U64

impl Index for Subset<B>:
    let count = subset_count(B.count)
    @get fn ord(self) -> Int = Int(bits)
    fn nth(k: Int) -> Subset<B> = Subset(U64(k))

impl Subset<B>:
    pub fn each(self) -> yields B:
        for b in self.items:
            yield b
```

`impl Index` makes `Subset<B>` a library domain: `[Subset<V>]Int` is an
array over it and `for s in Subset<V>` walks all of its values. `each` makes
`for b in s` walk the members. No line of the compiler names `Subset`; a user
can write `Pair<V>`, `Perm<N>` or `Grid<R, C>` the same way.

## The features

| Feature | What it gives a library | std example |
|---|---|---|
| `impl Index for T` (`count`, `ord`, `nth`) | a type usable as a domain: arrays over it, `for x in T`, `T.count` | `Subset<B>`, `Pair<V>` |
| `impl Into<Int>` | a value usable where an `Int` key is expected | `Subset<B>`, `Pair<V>` |
| an `each` method that `yields` | `for x in value` over the type's members | `Subset.each`, `Bitset.each` |
| generators (`yields`) | lazy walks with no allocation | `graph.adj`, `tree.tour`, `seq.windows` |
| `@get` getters | O(1) members that read like fields | `s.len`, `x.ord` |
| `lends` and `lends mut` projections, with write-back | `st.at(i) += x` that updates a structure | `SegTree.at`, `matrix.at` |
| operation values (`Monoid`, `Action`, fn values) | structures parametrized by an op, with no virtual call | `SegTree`, `LazySegTree`, `Window` |
| generics with value parameters | `Mod<P>` with a constant or read `P` | `std.mod`, `std.ntt` |
| operator traits and `T.zero`, `T.one` | `+`, `*`, `-` on library types | `Mod<P>`, `Matrix`, geometry points |
| fresh domains returned from a fn | a new index space per call | `scc`, `compress`, `suffix_array` |
| field-wise derives | `Hash`, `Eq`, `Show` for any struct, from its fields | `std.derive` |
| inherent methods on a generic struct | methods on a view that win over trait methods | `Rev.take`, `Slice.iter` |
| `@test` fns and `laws_hold` | tests and law checks next to the code | every module |

Intrinsics, functions with no CP Lang body, are a fixed public list (bit
operations, float math, `read`, `print`, `flush`, `rand`, `clock`). They are
documented like any std fn and a user library calls them the same way.

## Where the compiler still knows a std name

Each one is logged as debt and counted by
`plan/domains/stdlib/measure.py privilege`, which only goes down.

| Compiler knowledge | Why | The general feature that removes it |
|---|---|---|
| `Indexed`, `Bidir`, `Keyed` membership | traits met by built-in views | user-declarable conditional impls |
| zip's later arguments accept `.rev` of an `Indexed` | `.rev` is not `Indexed` but reads `nth(k)` in walk order | the same conditional impls |
| builtin folds (`sum`, `count`, `all`, `any`, `first`, `min`, `max`) | they take a walk expression, which is syntax | std bodies, moved one at a time |

## Rules

- A std module may use only the public language, the intrinsics list and
  other std modules.
- If std needs a new power, it becomes a core feature for every library, or
  the module waits. A std-only special case is a bug in this page.
- A std module copied into a user project must behave the same. The
  copy-a-module test that checks it is `missing` in
  [the stdlib register](../domains/stdlib.md).

## Changes

- 2026-11-23: first version.
