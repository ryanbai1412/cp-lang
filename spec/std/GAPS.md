# Library bodies the compiler can't express yet

One line per fn left `todo`, or written below its documented cost, because
of a compiler gap: module, fn, and what the compiler is missing. Remove a
line when the gap closes and the body lands.

- dsu: `Dsu.find`, `Dsu.same`, `DsuOf.find` and everything built on it: `@cache` fields don't parse (E0107), so read-only finds can't halve paths; they run in O(log n) (union by size) instead of O(alpha(n)) amortized. Mutating calls (`unite`, `merge`, `size`) on `Dsu` still halve paths.
- dsu: `PotentialDsu.new(op:)` with a non-`None` `op`: a struct field can't be typed by a fn's own generic `G`, so only the default `+` on `Int` is implemented; `op: Some(_)` reaches `todo`.
