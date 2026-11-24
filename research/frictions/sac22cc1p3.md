# Friction report: sac22cc1p3

There is no dream.cp. The port is the ideal: a plain simulation on a `[Row, Col]Char` grid.
- `slide_left` walks each W left with `offset(-1)` while the cell is empty.
- `fall` stages the drops in a list, so that all of them see the board from before the step.
- Action 1 repeats both until neither moves.

## Dream

The port as written.

There are no dream/port gaps. The staged `drops` list is the problem's simultaneity rule, not a workaround. Each W moves left or down at least once per round, so the loop ends; that is O((NM)^2) per action at N, M <= 100.

## Measurements

- **Tokens:** port 363, accepted C++ 618 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.00 s / 3 MB, accepted C++ 0.00 s / 3 MB, with the same output.
- **Emitted C++:** 22440 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
