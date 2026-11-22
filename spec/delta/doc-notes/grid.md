# grid.cp doc notes

## Contradictions

- `ray`: the `lib/graphs-trees.md` table says "the cells from `p` to the edge", which leaves open whether `p` is included; the stub says `self` is not included. Documented the stub.

## Choices

- `Dir4.from`: `letters` that are not exactly four characters long trap. A repeated letter means the first direction it stands for (up, right, down, left order).
- `Dir8.all`: numbered clockwise from up as up, up-right, right, down-right, down, down-left, left, up-left. Examples index `Dir8.all()` because `Dir8` has no named constructors.
- `adj4` / `adj8`: they keep `Dir4.all()` / `Dir8.all()` order and skip cells off the grid, so the order is fixed even at borders.
- `transposed`: an empty grid gives an empty grid.
