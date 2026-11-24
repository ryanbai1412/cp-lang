# Friction report: egoi23p2

Rethought. The v0.2 port read each match as two loose `Player`s, called the merge tree `children`/`rep`, and ordered the leader set by `(nights, -p.idx)`, a negated index standing in for "least index wins ties". The v0.3 port reads `[Match](x: Player, y: Player)` in one go, names the forest by what it means (`takes`, `last_win`), and keys the ordered set by `(nights, desc(player))`, so `board.max()` is the leader with no arithmetic on indices.

No language gap.

## Measurements

gen.py --max (N = M = 2e5, deep chains, 8 MB stack): port 0.15 s / 31 MB, accepted C++ 0.17 s / 28 MB, same output.
