# subset.cp doc notes

## Contradictions

- None found against `lib/` or `decisions.md`.

## Choices

- `Subset.with` / `without`: adding a member already present, or removing one absent, leaves the subset unchanged; neither traps.
- `Subset.subsets`: uses the standard `(t - 1) & self` walk, so the order is decreasing as masks. This is consistent with the stub's "`self` first and `{}` last".
- `Subset.supersets`: uses the `(t + 1) | self` walk, so the order is increasing as masks, consistent with "`self` first, `B` last".
- `Subset.first`: no ties are possible; it returns the least member in `B`'s order.
