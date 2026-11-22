# sparse doc notes

- `SparseTable.new`, `DisjointSparseTable.new`, `SparseTable2D.new`, `LinearRmq.new`: the spec is silent on empty input; chose that building succeeds and every asserting query traps (`try_*` gives `None`), matching `fold` asserting non-empty ranges.
- `SparseTable.all`, `DisjointSparseTable.all`: "fold of the entire non-empty domain" read as a trap on an empty domain, the same as `fold` on an empty range.
- `idempotent`: the stub says "on these samples"; chose to test only `op(x, x) == x` per sample (not associativity), `true` on no samples.
- `LinearRmq.best`, `try_best`: ties are not stated for the value forms; chose the value at the leftmost best position, consistent with `argbest`.
- Contradiction (minor): `SparseTable`'s struct doc lists `op` laws as associative and idempotent, but nothing defines the result of a non-idempotent `op`; documented as unspecified rather than a trap, since the law cannot be checked in O(1).
- Contradiction: `spec/lib/ops-ranges.md` section 8 says "`DisjointSparseTable` becomes `SparseTable`", but the stub keeps `DisjointSparseTable` as its own struct. Documented the stub as written.
