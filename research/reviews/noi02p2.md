# noi02p2

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`, a fragment). The ring argument reads well: each child's answers about both neighbours (lines 26-27), `Child.find` for the first disagreeing edge (line 30), then the two walks away from the referee (lines 34-37).

## Checklist failures

- idiomatic-code/R4: lines 35 and 37 write the same inference twice, `if answer == (role[neighbour] == Star): Star else: Moon`, once per walk direction. Not reported.
- `role[c].ord` (line 38) prints the team code through the declaration order of `Role`; the encoding (0 Star, 1 Moon, 2 referee) is stated nowhere.

## Owed rewrites

- `fn team(answer: Bool, of: Role) -> Role` used by both walks, and a comment or explicit match for the printed code.

## Language gaps

- No ring successor on a domain (`Child.wrap(Int(p) + 1)`, lines 26-30), `ask` answers read as `0..=1` and compared to 1 (lines 22-23), and no Bounds gate for `ask` reads: all in the friction.

## Dream notes

`p.ring_next` and a typed `ask.says_star` name the right things, but the fragment covers two lines; the walks and the output should be in it, with the inference written once.
