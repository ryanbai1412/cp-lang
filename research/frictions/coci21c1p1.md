# coci21c1p1 friction

The old port read `n: Int` and each spray field by field, kept two score variables and
nested an `if let` inside the `if` for the bonus. The port reads the sprays as one
`[n](t, who, hit)` table, scores with `last[who].is_some_and(...)` and keeps the totals in
`[Team]Int`.

## Dream

The existing `dream.cp` is the target; the port follows it.

## Team of a player

- **Where:** `score[Team(who.ord / 4)]`. A player's team is arithmetic on its ordinal; a
  sub-domain split (`Player.split(4)` giving `(team, member)`) would name the idea.
- **General fix:** domain partitions into equal blocks.
- **Status:** open.

## Spray time bound

- **Where:** `t: Int`. The statement in the repo's sources gives `n <= 100` and increasing
  times but no bound on `t` that could be checked here, so `t` stays `Int`.
- **Status:** open (needs the statement's bound).

Measured: `cpl stress` agrees with the old port and with the accepted C++ (4080824) on 30
cases each.
