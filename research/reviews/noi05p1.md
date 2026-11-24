# noi05p1

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`). The editorial's sliding-window DP: per tilt, walk each line in the tilt's direction, reset at furniture, keep the best `slide - place` in reach in a `Window(max)` (lines 45-59). At `--max` it takes 0.10 s against the 400 ms limit (friction updated; it still cited the older 0.45 s).

## Checklist failures

- idiomatic-code/R5: `from: Int, to: Int` (line 22) leave the time bounds out of the read and the header's Bounds line.
- The place along a line is kept by hand (`var i = -1`, `i += 1`, lines 50-52) beside the `for`.
- `starts` and `along` (lines 25-37) decode the numbered direction twice; `line` (lines 39-41) is `s` followed by `s.ray(d)`. The friction's dream (`Cell.lines(way:)`, `run`s split at furniture) is the missing std piece, reported without a run.

## Owed rewrites

- Bound `from`/`to` from the statement; one table from `way` to (direction, start edge); iterate the line with its index.
- Quote a `cpl run` for `Cell.lines`/`split` on a line, or keep it listed here.

## Language gaps

- Lines of a grid along a direction and splitting them into runs: in the friction's dream, unreproduced.

## Dream notes

Three lines that state the recurrence; `run.sliding(max, width:, of:)` names the window, though it hides how the window is keyed by place.
