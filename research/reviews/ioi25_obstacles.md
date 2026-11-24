# ioi25_obstacles

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port. Both are the official layer forest:
- rows with a new record temperature open runs of drier columns;
- each run absorbs the open layers inside it, through the `tightest` drop pair;
- `Lift` answers a query as an LCA plus two folds.

The domains `Row` and `Col`, the structs `Layer` and `Drop`, and `?`-typed `layer`/`up`/`home` carry every index space and "not yet".

## Checklist failures

- The port's run walk (`var at`, `while true`, `end = cols.last!`, `at = Col(end.ord + 1)`) is a hand cursor where the dream has `open.drain(after: run.l.l, while: (c, _) => c in run)`. `placed: [Col]Bool` stands in for the dream's `distinct(by:)`. The port does this to fit the judge size target: 28465 bytes against the dream's 47342 (`cpl build --judge codeforces --size`). The friction routes the size to codegen.
- `run.l.l` (the cut before a range) is opaque. The friction asks for `drain(in: range)`.

## Gaps that need the language

- Grader-module state, the std names the dream wanted, `CpHash` for `Range` keys, and the runtime size: research/frictions/ioi25_obstacles.md.
