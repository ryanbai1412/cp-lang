# coci21c1p1

Score: 4/5
Dream: 4/5

Cold review against `dream.cp`; the port is the dream with the sprays read as a typed table and the totals in `[Team]Int`. Each player's last spray is `?Int` (line 14) and the bonus is `is_some_and(s => t - s <= 10)` (line 17), as the header says.

## Checklist failures

- None beyond the reported gaps (team by `who.ord / 4`, line 19; `t: Int`, line 13).

## Owed rewrites

- None.

## Language gaps

- A domain split into equal blocks (`Player.split(4)`): in the friction.
- The bound on `t`: in the friction, open until the statement is checked.

## Dream notes

- Reads like the editorial: two header lines state the rule. `score[if p.ord < 4: 0 else: 1]` over a bare `[0, 0]` is its one rough spot, which the port's `Team` domain improves.
