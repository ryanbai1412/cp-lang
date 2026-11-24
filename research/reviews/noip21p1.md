# noip21p1

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

## Checklist failures

- `for m in (x.label..=LIMIT).step(x.label): alive.remove(X(m))` walks multiples through labels and converts back into the domain.
- `LIMIT = 10000100` is the bound past the next survivor above 1e7, picked by hand. It has a comment, but no check that it is past the survivor.
- `alive.next(X(x + 1))!.label` builds the successor's search point with label arithmetic.

## Gaps that need the language

- Multiples of a domain value through labels: research/frictions/noip21p1.md.
