# ioi18p1 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and `corpus/problems/ioi18p1/dream.cp`.

Reads like the editorial: the first letter in two presses, each middle letter in one press of
the four-part string scored by growth, the last letter in two again: N + 2 presses. The
`extern fn` local-judge gap is reported with a repro.

## Checklist failures

- `s.push([c, a, b][grew])` (line 27) decodes the press by indexing a literal list; the dream's
  `match got: 0 => r, 1 => p, _ => q` names each case.

## Language gaps

- A local judge for an `extern fn` grader task and `cpl check` missing what `cpl run`
  rejects in a `@grader` body: in research/frictions/ioi18p1.md.

## Dream notes

- At bar.
