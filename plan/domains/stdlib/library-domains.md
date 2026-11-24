# Library-declared domains (design draft, Q3)

Status: draft for spec (rule), compiler (check/ir/interp/codegen interface)
and tech-debt (architecture) sign-off. Once built, the rule folds into
`spec/core/04-indices.md` and `spec/core/12-powers.md`, and this file is deleted.

## Problem

`Subset<B>` is a struct in `std/subset.cp`, but `[Subset<B>]T`, `for s in
Subset<B>`, `Subset<B>.count` and every `Subset` method are compiler
built-ins: `cpl-check` types it as `Unknown` (prelude.rs, expr.rs), and
`cpl-ir` lowers it to a `Pos(d)` mask with about 100 `subset_base` sites,
`Iter::Subsets` and `Iter::Bits`. interp and codegen size it in
`size_subsets`. So std's own bodies are never run (6 doc examples fail:
`complement`, `shifted`, `single`), the builtin and std disagree (the
`supersets` order, fixed in 4750ee99c), and a user cannot write a domain of
their own (`Pair<V>`, a k-subset, a permutation rank, a bitset over 128
letters). That is std privilege (AGENTS.md, "no privileged stdlib";
compiler ARCHITECTURE, "std names are debt to remove").

## Rule (for spec)

The mechanism exists already. Decision 2.1 and decisions §9 (Ryan,
2026-11-20) say a bounded index is a sealed struct with `impl Index`, and
`domain V = 1..=n` is sugar for one. A user struct can already write:

    impl Index for Two:
        let K = 6                         // size, frozen at start (E0605 if it reads a `var`)
        fn to_idx(self) -> Int = ...      // ordinal in 0..K
        fn from_idx(k: Int) -> Two = ...  // its inverse

and `[Two]E`, `for t in Two` and `seen[t]` already work in both `cpl run` and
C++ (scratch check, 2026-11-23). This draft finishes that mechanism. It adds
no new trait. The spec text it needs:

R-new.1: `impl Index for T` (std or user, same rule) makes `T` a domain.
`K` is evaluated once, when the domain is sized: at start, or when a
runtime-sized type argument is bound (the decision 124 slot). `to_idx` and
`from_idx` must be \(O(1)\) and inverse to each other; a checked build
traps when `to_idx` leaves `0..K`.

R-new.2: a library domain gets exactly the domain members that need no
order, each meaning what its core namesake means: `[T]E` (`K` cells, `a[x]`
is cell `x.to_idx`), `for x in T` and `T.rev` (walk by ordinal), `T.count`
(= `K`), `T.first`, `T.last`, `T.nth(k)` (= `from_idx`, trap outside
`0..K`), `x.ord` (= `to_idx`), every `Seq` method on `T`, and use as a
domain in generics (`Dsu<T>`, `SegTree<T>`, `@memo` keys, `[T]E` fields).
`T(k)`, `T.at(k)`, `Int(x)` and `T.wrap` are not derived, because their core
spellings take a value, not an ordinal. Read, print and `Int(x)` come from
the type's own `Show`, read and conversions (`Subset` prints and converts as
its mask, decision 291).

R-new.3: neighbours (`.prev`/`.next`), cuts (`T.cuts`, `.l`/`.r`), `Range<T>`
and position arithmetic stay with the ordered core domains (ranges, `Char`,
enums, products), as decision 291 already rules for `Subset` and `Pair`.

R-new.4: core domains and `impl Index` types satisfy the same bound, so a
generic `D: Domain` accepts both. Today both `Domain` and `Index` are
markers only the checker knows (cpl-check types.rs). One of them becomes
the real trait, `Index`, the name decision 2.1 uses, and the other its alias.

Subset under the rule, no compiler knowledge:

    impl Index for Subset<B>:
        let K = 1 << B.count               // [Subset<B>]T over |B| > 26 is the ordinary array-size trap
        fn to_idx(self) -> Int = Int(bits)
        fn from_idx(k: Int) -> Subset<B> = Subset(U64(k))

`Int(s)` stays the mask and the walk stays `0, 1, ..., 2^|B| - 1`, as now.
`Pair<V>` moves the same way (`K = n(n-1)/2`, `to_idx = hi(hi-1)/2 + lo`).
What `impl Index` lacks today: `K` that reads a type argument (`B.count`),
generic structs (`Subset<B>` is one), `T.count`/`T.nth`/`x.ord`, the
ord-range trap, and spec text.

Spec signed off on R-new.1-4 (2026-11-23) under the names `Domain`,
`count`, `ord` and `nth`, with R-new.2 amended as above. Re-confirmation is
needed for the existing names `Index`, `K`, `to_idx` and `from_idx`.
Renaming those is Ryan's call, since §9 is his.

Compiler signed off on the interface below, with these conditions:
- one table for every library domain, with no Subset-only fields;
- `D: Domain` resolved through mono's normal impl dispatch;
- the interpreter in the cost gate.

## Interface (for compiler)

- **check:** `Index` is a prelude trait declared in std, with `Domain` as its
  alias. Anywhere a domain is required (array dimension, `for x in T`,
  `T.count`, `T.nth`, a `D: Domain` bound), a type with an `Index` impl is
  accepted, and its members are typed from the trait. The `"Subset"` names
  in prelude.rs and expr.rs go.
- **ir:** a library domain instance `S<args>` gets a `DomainId` on first use,
  as `subset_domain` and `impl Index`'s `index_domain` do now. It is recorded
  in `Program.library_domains: Vec<LibDomain { domain, ty, count, ord, nth }>`,
  which generalizes `StructDef.index_domain` and its `K` const to generic
  structs and
  replaces `Program.subsets`. Its values stay `Ty::Struct`, not `Ty::Pos`:
  `a[x]` lowers to an index by `ord(x)` on the `Dim::Pos(d)` dimension,
  `for x in S` to `for k in 0..len(d): let x = nth(k)`, and `S.count` to the
  domain's length. Methods are ordinary std functions. The `subset_base`
  sites, `Iter::Subsets` and `Iter::Bits` are deleted.
- **interp and codegen:** when a domain's type arguments are sized, size
  each library domain over them by calling its `count` (this generalizes
  `size_subsets`). They need no other knowledge. In C++, `ord` and `nth` are
  ordinary inline functions on a one-`uint64_t` struct, so `dp[s]` compiles
  to `dp[s.bits]`.
- **Cost check before landing:** a mask DP (`[Subset<B>]Int` over `|B| = 20`,
  a submask walk) must stay within 1.1x of the current builtin, in both the
  C++ build and `cpl run`. If `cpl run` is slow, inline small `ord` bodies in
  IR. Baseline on the builtin (scratch bench: `[Subset<B>]Int` over
  |B| = 20 with a member test, plus a submask walk over |B| = 16): C++
  0.22-0.23 s, `cpl run` 6.46 s. The builtin also lacks `s.contains` in
  `cpl run` and can't walk `s.items` in `cpl check`. The risk is std's `yields` generators (`s.subsets`, `s.items`) in
  place of `Iter::Subsets`/`Iter::Bits`.

## Plan

1. Spec signs R-new.1-4. Compiler signs the interface above. Tech-debt
   reviews the split.
2. Land `Domain` and library-domain lowering, with `Subset` moved to it, in one
   change, plus these conformance cases (each run in `cpl run` and C++):
   - a user struct implementing `Domain`: `[T]E`, `for x in T`, `count`,
     `Dsu<T>`;
   - `T.nth(k)` round-tripping `x.ord`;
   - `t.next` on a library domain, an error naming the core-only rule;
   - `[Subset<B>]T` mask DP;
   - `Subset` methods from std;
   - a `D: Domain` generic over both kinds;
   - the ord-out-of-range trap.
3. Move `Pair<V>`, then `Char` (128 written once), the same way.
4. Fold the rule into spec/core/04 and 12, with a decision number from
   `scripts/spec-gate.py decisions` and an R04 rule ID per item, then delete
   this file.
