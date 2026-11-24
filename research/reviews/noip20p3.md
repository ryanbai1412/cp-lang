# noip20p3

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port. The port this review found was a translation (score 2): globals, `ri()`, 0-based `Int` pillars, and a workaround comment for a fixed bug. The port is now the verified dream with typed reads. It agrees with the old port on 30 gen.py cases under `checker.py` and passes its 9 package cases.

The divide and conquer reads as the header states:
- each tube is sorted into a low block and a high block;
- mixed tubes with the same bottom kind merge until at most one pair is left;
- one trade fixes that pair, and the tubes split by bottom kind.

## Checklist failures

- `mv(helper, a, lows); mv(spare, a, m - lows)` puts two statements on one line in four places. That is compact, but the `if`/`else` arms differ only in order, so one ordered pair `(first, second)` chosen by the condition would say it once.
- `given = [if t == b: spare else: t for t in given]` and `spare = b` swap a list element with a var by rebuilding the list. `given.replace(b, with: spare)` would state it.

## Gaps that need the language

- None.
