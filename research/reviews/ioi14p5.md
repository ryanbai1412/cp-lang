# ioi14p5 review

Score: 4/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi14p5.md).

Reads like the editorial: undo the growth from the last joiner, keep `(pick, skip)` per
group, and merge by protocol with one line each, commented with the rule. Integer-coded
protocols and the `Joiner <: P` conversion are reported; the grader task was not.

## Checklist failures

- `pick`/`skip` are two parallel arrays for one pair per person; the dream's
  `[P](pick: Int, skip: Int)` says it as one.

## Language gaps

- Enum reads by ordinal and the `Joiner` subdomain: in research/frictions/ioi14p5.md.
- The judge's function task (`findSample` over C arrays) has no CP Lang form: added to
  research/frictions/ioi14p5.md (same E0501 as ioi14p1).

## Dream notes

- The dream stops at `...` inside the loop, which is the whole solution. It should show the
  three merges, e.g. `match protocol[j]: Host => ...; Friends => ...; Both => ...`.
