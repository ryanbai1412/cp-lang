# ioi20p3 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi20p3.md).

Reads like the editorial: the `n * k / 2` best upgrades, decreasing per colour, then each round
gives its `n / 2` colours with the most highs left their largest ticket. The partial
selection, the hand count by colour and the `round` table's type are reported.

## Checklist failures

- `for u in taken: high[u.c] += 1` (line 32) counts by key by hand; reported (`count_by`).
- `round: [Colour, Ticket]?Int` (line 37) loses the round bound; reported.

## Language gaps

- All in research/frictions/ioi20p3.md.

## Dream notes

- At bar; `let t = if ...: high[c] -= 1; left[c].pop()! else: ...` puts a statement inside an
  `if` expression with `;`, which the port's two branches say more plainly.
