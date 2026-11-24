# ioi00p2

Score: 2/5
Dream: 2/5

## Cold review (HEAD)

Port 2/5, dream 2/5, from the friction's `## Dream`. The header (lines 6-10) explains the chain walk and why each round fixes about w - 1 cars; the body is that walk, kept in raw `Int` slots.

## Checklist failures

- The driver budget `moves.len + 4 <= 2 * w` is written twice (lines 30 and 35), and `moves` stores (from, to) as two adjacent `Int`s, recovered by `moves.len / 2` (line 47) (idiomatic-code/R4). A `List<(from: Slot, to: Slot)>` with `print(moves.len, moves)` per round says it once.
- `while i < n and park[i] == want[i]: i += 1` (line 31) is a first-match search by hand (idiomatic-code/R3): `(i..n).first(j => park[j] != want[j]) ?? n`.
- Slots and types stay `Int` (lines 18, 22). The friction reports it as low and blames `.next`/`.label` arithmetic, but the walk only steps `cursor[car]` and prints `+ 1`; a `domain Slot = 1..=n` read 1-based would drop both `+ 1`s.

## Dream notes

- The dream shows the reads and the print only; `for round in rounds:` assumes the whole round construction. Sent back: it should show the chain walk with a named move and the driver budget once.
