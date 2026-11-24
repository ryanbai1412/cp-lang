# cco23p4

Score: 3/5
Dream: 3/5

Cold review against the friction's `## Dream`. The relabel so T starts with `'0'` (`plain`, line 13), `windows` counting (line 17), `split('1')` gaps (line 46) and one `match` over the pattern shapes (lines 42-48) are clean.

## Checklist failures

- R1: the header (lines 5-9) states the per-pattern formulas without deriving them, e.g. `"010" => (hits() + 1) / 2` (line 45) needs "one reversal removes at most two occurrences, and two always can be".
- R2: `spread` (lines 19-40) is a four-branch counter loop; the header's "two at a time into empty gaps, else into gaps of size 1" is not visible in it, and the `g == 3` branches are unexplained.

## Owed rewrites

- Derive the formulas and the `g == 3` greedy choice in the header.
- The window friction is stale: line 17 already compares `x == p` without `Str(x)`; update or drop it.

## Language gaps

- Length-bounded `Str` reads (lines 11-12): in the friction, without a `cpl run` repro (R11).

## Dream notes

- Three lines over the port (bounded `Str`, `count(p)`), so the composite keeps `spread`'s undervived counter loop and the stated-not-derived formulas: capped at 3. The dream should write the `"000"` greedy as the rule it is.
