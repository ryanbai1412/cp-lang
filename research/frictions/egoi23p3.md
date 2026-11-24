# Friction report: egoi23p3

Rethought. The v0.2 port built every move string by pushing chars through an `append_rep` helper, spelled the H = 1 case as a separate early-return branch, and repeated the `print("!", ..)` four times. The v0.3 port builds the moves with `Str.repeat`, folds H = 1 into the first query (just right to the wall), and steps with `Col.at(..).map(..)` optionals. It keeps the survivors with `filter`, tells them apart with `distinct().len`, and picks the second query with `(0..h).map(row_probe).find(..) ?? fallback`. The package gains gen.py and an interactor that answers queries and requires the box within 2 queries.

Gap: `wire.ask` cannot read a tuple when its type comes from the expected type (`let got: (Row, Col) = wire.ask(..)`) or from an alias (`wire.ask<Cell>(..)`), only when spelled out (`wire.ask<(Row, Col)>(..)`). Sent to Core.

## Dream

```
fn ask(boxes: List<Cell>, moves: Str) -> List<Cell>:
    let got: Cell = wire.ask("? {moves}")
    boxes.filter(b => robot(b, moves) == got).collect()
```

## Measurements

gen.py --max (50 x 50, box in the last row): 0.11 s under the interactor. Port, v0.2 port and accepted C++ each find the box on 121 generated grids.
