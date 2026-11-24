# noip09p4

Score: 3/5
Dream: 4/5

Cold review: the fewest-options backtracking over row, column and box `Subset`s matches the header, but the cell pick is a hand min loop that std already names, and five `WISH` comments stand in for friction entries.

## Checklist failures

- R3: `pick`/`fewest` (lines 36-44) hand-select the blank with fewest options; `min(by:)` over the blanks (dream.cp line 34) is available.
- R5: blanks are `0` in `board: [Line, Line]0..=9` (line 15), tested and reset as `0` (lines 40, 53).
- R4: `fits` is counted per digit (line 41), then re-tested per digit (line 49), where one `options(r, c)` subset says both.

## Owed rewrites

- `options(r, c) -> Digits` as in dream.cp line 12, and the blank chosen by `min(by: options.len)`.

## Language gaps

- `Subset.set` inside a fn being dropped (P0, routed), `^=` on an indexed `Subset` and `Subset.single` needing its type: in research/frictions/noip09p4.md. The `WISH L20` read of `?Digit` with 0 for none is the gap filed in noip06p2.

## Dream notes

- None.
