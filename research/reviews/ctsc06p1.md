# ctsc06p1

Score: 4/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Expected waiting time for a name is m to the power of each border length, summed: four lines over std `borders` and `Mod<10_000>`, printed zero-padded.

## Checklist failures

- `Z(m) ** name.len + sum(Z(m) ** b for b in borders(name))` adds the whole name as its own term because `borders` leaves the full sequence out. Summing over "borders including the whole" would be one term.

## Gaps that need the language

- `borders` excludes the whole sequence: research/frictions/ctsc06p1.md.
