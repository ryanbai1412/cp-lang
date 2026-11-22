# Library bodies the compiler can't express yet

One line per fn left `todo`, or written below its documented cost, because
of a compiler gap: module, fn, and what the compiler is missing. Remove a
line when the gap closes and the body lands.

- dsu: `Dsu.find`, `Dsu.same`, `DsuOf.find` and everything built on it: `@cache` fields don't parse (E0107), so read-only finds can't halve paths; they run in O(log n) (union by size) instead of O(alpha(n)) amortized. Mutating calls (`unite`, `merge`, `size`) on `Dsu` still halve paths.
- dsu: `PotentialDsu.new(op:)` with a non-`None` `op`: a struct field can't be typed by a fn's own generic `G`, so only the default `+` on `Int` is implemented; `op: Some(_)` reaches `todo`.
- subset: every `Subset` method: `Subset` is built into the compiler, so `cpl run` ignores these bodies, lacks `shifted`, `complement`, `supersets` and `for` over `items`, and a program's `Subset<D>` doesn't unify with the stub's (E0302 "expected `Subset<D>`, found `Subset<D>`"). The bodies check but can't be exercised by conformance cases yet.
- mod: `Comb` (all of `new`, `grow`, `ensure`, `fact`, `inv_fact`, `inv`, `choose`, `arrange`, `derange`, `multinomial`): its tables are `@cache` fields grown from read-only methods, and `@cache` fields don't parse (E0107).
- mod: `Barrett.reduce`, `Barrett.mul`: no 128-bit multiply-high on `U64`, so they reduce with `- mod: `Comb` (all of `new`, `grow`, `ensure`, `fact`, `inv_fact`, `inv`, `choose`, `arrange`, `derange`, `multinomial`): its tables are `@cache` fields grown from read-only methods, and `@cache` fields don't parse (E0107).
- mod: `Barrett.reduce`, `Barrett.mul`: no 128-bit multiply-high on `U64`, so they reduce with `%` (same O(1) cost, not division-free).
- rand: `Rng.perm`: explicit type arguments on a method call (`r.perm<V>()`) parse as comparisons (E0309), and `let p: [V]V = r.perm()` infers `D = Int` in `cpl run` (E0302); the body checks but no program can call it for a declared domain.
