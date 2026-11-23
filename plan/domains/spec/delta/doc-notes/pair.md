# pair: doc-standard notes

- `Pair.new` / `at` examples: write the domain as an anonymous range, `Pair<0..4>`, the same way `[0..=n]Int` names an anonymous domain; chosen so examples need no `domain` declaration.
- `symmetric`: `empty:` is the empty matrix for an empty `V` and `[[diag]]` for a one-position `V` (no pairs exist, so only the diagonal is filled); follows from the stated formula.
- `symmetric`: `order:` is row-major over `(V, V)`, the order every `[V, V]T` walks in.
