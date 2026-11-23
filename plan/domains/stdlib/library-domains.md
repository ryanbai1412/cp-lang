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

A type is a domain when it implements the prelude trait `Domain`:

    pub trait Domain:
        fn count() -> Int          // number of values; may read its type arguments' counts
        fn ord(self) -> Int        // the value's ordinal, in 0..count
        fn nth(k: Int) -> Self     // the inverse of ord, for k in 0..count

R-new.1: `impl Domain for T` (std or user, same rule) makes `T` a domain.
`count` is evaluated once, when the domain is sized: at start, or when a
runtime-sized type argument is bound (the decision 124 slot). `ord` and
`nth` must be \(O(1)\) and inverse to each other; a checked build traps when
`ord` leaves `0..count`.

R-new.2: a library domain gets exactly the domain members that need no
order: `[T]E` (`count` cells, `a[x]` is cell `x.ord`), `for x in T` and
`T.rev` (walk by ordinal), `T.count`, `T.first`, `T.last`, `T.nth(k)`,
`T.at(k)`/`T(k)` (ordinal to value, `None`/trap outside `0..count`),
`x.ord`, `Int(x)` (= `x.ord`), `T.wrap`, every `Seq` method on `T`, and
`D: Domain` in generics (`Dsu<T>`, `SegTree<T>`, `@memo` keys, `[T]E` fields).

R-new.3: neighbours (`.prev`/`.next`), cuts (`T.cuts`, `.l`/`.r`), `Range<T>`
and position arithmetic stay with the ordered core domains (ranges, `Char`,
enums, products), as decision 291 already rules for `Subset` and `Pair`.
A later `Ordered` refinement could add them; this draft doesn't.

R-new.4: the core domains (ranges, `Bool`, enums, `?T`, tuples and records,
decision 2.16) satisfy `Domain` structurally, so `D: Domain` is one bound
for both kinds. Today `Domain` is a marker only the checker knows
(cpl-check types.rs accepts `Pos` and `Domain` types); it becomes a real trait.

Subset under the rule, no compiler knowledge:

    impl Domain for Subset<B>:
        fn count() -> Int = 1 << B.count      // [Subset<B>]T over |B| > 26 is the ordinary array-size trap
        fn ord(self) -> Int = Int(bits)
        fn nth(k: Int) -> Subset<B> = Subset(U64(k))

`Int(s)` stays the mask and the walk stays `0, 1, ..., 2^|B| - 1`, as now.
`Pair<V>` moves the same way (`count = n(n-1)/2`, `ord = hi(hi-1)/2 + lo`).

Questions for spec: (1) the trait's member names: `count`/`ord`/`nth`
match R04.2.2, R04.2.24 and R04.2.9. (2) Should `T(k)` on a library domain
take the ordinal (this draft) or be left out? (3) Is the R-new.3 split
right, or should neighbours come by default in ordinal order?

## Interface (for compiler)

- **check:** `Domain` is a prelude trait declared in std. Anywhere a domain is
  required (array dimension, `for x in T`, `T.count`, `T.at`, a `D: Domain`
  bound), a type with a `Domain` impl is accepted, and its members are typed
  from the trait. The `"Subset"` names in prelude.rs and expr.rs go.
- **ir:** a library domain instance `S<args>` gets a `DomainId` on first use
  (as `subset_domain` does now), recorded in `Program.library_domains:
  Vec<LibDomain { domain, ty, count: FnId, ord: FnId, nth: FnId }>`, which
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
  a submask walk) must stay within 1.1x of the current builtin in the C++
  build. The risk is std's `yields` generators (`s.subsets`, `s.items`) in
  place of `Iter::Subsets`/`Iter::Bits`.

## Plan

1. Spec signs R-new.1-4. Compiler signs the interface above. Tech-debt
   reviews the split.
2. Land `Domain` and library-domain lowering, with `Subset` moved to it, in one
   change, plus these conformance cases (each run in `cpl run` and C++):
   - a user struct implementing `Domain`: arrays, walk, `count`, `at`,
     `Dsu<T>`;
   - `[Subset<B>]T` mask DP;
   - `Subset` methods from std;
   - a `D: Domain` generic over both kinds;
   - the ord-out-of-range trap.
3. Move `Pair<V>`, then `Char` (128 written once), the same way.
4. Fold the rule into spec/core/04 and 12, then delete this file.
