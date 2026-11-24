# iati21a12

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-10) gives the greedy and why served units re-enter at value 0. The body follows it with std `MeldHeaps`: lazy `add_all` lifts a child's supply by the edge weight, `meld` merges it, and the serve loop takes the best lots (lines 36-53).
- `struct Lot` with `impl Add` (lines 25-27) is an addition that keeps the left count, there only so `add_all` can lift the value. It is in the friction (a keyed lift).
- `want`/`give` and the edge weight are `Int` (lines 17-18, 21): the statement is not public. In the friction.
- `kids` is built by hand from the parent list (lines 19-22), with `V.nth(p - 1)` shifting 1-based parents into a 0-based domain. Reading parents as `V` (`domain V = 1..=n`) would drop the shift.
- Lines 44-46 pop a lot, find it negative and push it back. A `pop_if(l => l.value >= 0)` on the heap would say "while worth at least 0" once.

## Dream notes

- The dream is the port with a lift of one field (`add_all(value: w)`) and `absorb`; with them `Lot` and its `impl Add` go.
