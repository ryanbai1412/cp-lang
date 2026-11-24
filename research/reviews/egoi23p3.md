# egoi23p3

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the friction's `## Dream`. A snake sweep, then a row probe chosen by `tells_apart`, filtering candidates by simulating `robot` (lines 28-51): the header's two-query plan.

## Checklist failures
- idiomatic-code/R8: line 39, `wire.ask<(Row, Col)>(...)`, spells out `Cell` because the friction says neither the expected type nor the alias works. Both now run on a fresh `cpl` (`let got: Cell = wire.ask("?")` and `wire.ask<Cell>("?")` read `3 4` and print `3`), so the gap is stale and the dream line is writable today.
- idiomatic-code/R1: `fallback` (line 46) is a fixed probe whose reason for separating every remaining candidate is not derived in the header.

## Owed rewrites
- Line 39: `let got: Cell = wire.ask("? {moves}")`, the dream; drop the friction's gap line.
- Derive the fallback probe in the header.

## Language gaps
None now: the reported tuple-read gap no longer reproduces (output added to the friction).

## Dream notes
The dream is the port's `ask` with the alias type.
