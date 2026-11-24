# ceoi17p2

Score: 4/5
Dream: 4/5

Cold review against `dream.cp`; the port is the dream with renamed variables. Sort each side best-first (lines 13-14), always extend the side that pays less, and keep the best `min(pays_a, pays_b) - (i + j)` (lines 16-24). The dream header (lines 1-3) derives the greedy.

## Checklist failures

- None beyond the reported gap.

## Owed rewrites


## Language gaps

- Closed `F64` ranges as read types (the odds' `1..=1000`, line 12): in the friction.
- The adaptive two-cursor walk (lines 17-23): the friction argues no std merge expresses a running-sum order; agreed.

## Dream notes

- Reads like the editorial: the header states why best-first per side is optimal and the profit formula; the loop is that rule.
