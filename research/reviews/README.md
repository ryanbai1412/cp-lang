# Taste reviews

One file per port, `NAME.md`, written cold: the reviewer reads `AGENTS.md`,
`plan/PORTING.md`, the exemplars, the statement (or the friction report),
the dream and the port, never the C++ original. The bar is
`design/EXEMPLARS-PORTS.md`: a 5 reads like one of its exemplars, for the
port and for the dream. `scripts/review-check.py` reads the first `Score:`
line (the port) and `Dream:` line (the dream) and fails `scripts/check.sh --fast` when a
port newer than `baseline.json` has no dream or no review.

A port author commits `Score: pending` with the port; the reviewer replaces
it with a score.

## Scores

- **5** could be an exemplar in `design/EXEMPLARS-PORTS.md`: reads like the editorial, every fact once,
  nothing a contestant would want to change.
- **4** reads like the editorial; the gaps are minor and every one is in the
  friction report.
- **3** correct but translated, or a standard algorithm hand-rolled because
  the stdlib lacks it or is measured too slow, with the gap reported and
  measured. A port with no dream scores at most 3.
- **2** hand-rolled standard algorithms or bookkeeping the language already
  removes, or a fact kept in two places, unreported or reported with an
  excuse that does not hold.
- **1** unreadable.

## Dream score

`Dream: N/5` on the line after `Score:`, on the same scale, judged against
the exemplars in `design/EXEMPLARS-PORTS.md`: the header states the
invariant, standard algorithms are named, structures with an invariant get a
type, searches are `find`/`first`/`any`, index spaces are domains, each
bound once, no sentinels or fake monoids. A dream under 4 reads like the C++
and is sent back: the review says what the dream should say instead, and the
port is judged against that rewrite rather than the dream as written.

## Sections

- **Checklist failures:** each failed `plan/PORTING.md` item, with the line.
- **Owed rewrites:** gaps today's language can already close. The author
  owes these; each names the rewrite and, where it matters, the run that
  shows it works.
- **Language gaps:** gaps that need the language. Each must be in the
  friction report; missing ones are flagged.
- **Dream notes:** where the dream itself misses the exemplar bar.
