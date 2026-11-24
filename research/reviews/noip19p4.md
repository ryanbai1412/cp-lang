# noip19p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

## Checklist failures

- `var next = ways.clone()` copies the whole `Lead` table per method so that the update reads old values. A double buffer or an in-place pass in the right direction is the usual spelling (see the friction).
- `for d in Lead(1 - i.label)..=Lead(i.label - 1)` limits the loop by converting labels back into the domain by hand.
- `[Lead] d => Z(if d.label == 0: 1 else: 0)` writes a unit vector through a conditional on the label.

## Gaps that need the language

- The rolling DP's clone and hand-limited range: research/frictions/noip19p4.md.
