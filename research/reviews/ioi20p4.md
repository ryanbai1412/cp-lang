# ioi20p4

Score: 3/5
Dream: 3/5

Cold review: the port is correct and its header derives the condition x * (y mod 2^(i+1)) <= s[i], but the count is the C++'s cap walk over raw `Int` bits, with the padded table, the push-built `below` list and the grader loop all reported.

## Checklist failures

- R4: `domain Bit = 0..60` (line 18) and `for i in 0..60` (line 29) write the bit count twice.
- R6: `cap_below(i: Int)` (line 25) and `below[i]`/`below[j]` index bits by `Int`, so `Bit` only types `s`.

## Owed rewrites

- `for i in Bit` with `Int(i)` at the shifts, and `cap_below(i: Bit)`, would keep the bound in one place.

## Language gaps

- Grader entry, a `Bit`-indexed `a.get(i) ?? 0` over a shorter list, and a table defined from its own smaller indices: all in research/frictions/ioi20p4.md.

## Dream notes

- The `## Dream` section is a fragment: `below` is `...`, so the dream never says how the working y are counted (the walk over the cap's high bits). It should state that recurrence; until it does the port's score is provisional.
