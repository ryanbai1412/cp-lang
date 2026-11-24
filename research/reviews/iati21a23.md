# iati21a23

Score: 3/5
Dream: 2/5

## Cold review (HEAD)

Port 3/5 (capped: the dream is under 4), dream 2/5, from the friction's `## Dream`. The header (lines 6-9) names the two strategies (one window when s <= c, two half-windows otherwise), and each is a loop over turns (lines 18-59).

## Checklist failures

- The window ends move by unexplained steps: `r - 2` with a width-2 `pair` case (lines 25-27), the shot at `l + 1` (line 29), `if r - l + 1 > 2: r -= 1` (line 30). The header doesn't derive why each shot and move is right (idiomatic-code/R1). The hand-moved ends are in the friction (low).
- The two-window branch writes the same flee/approach updates twice, once per window (lines 39-59). A `Window` struct with `flee()`/`approach()` clamped to `1..=n` would write each move once (idiomatic-code/R4).

## Dream notes

- The dream is two lines of pseudocode (`shoot(the end it moves toward)`) with an invented `IntervalSet.step`; it names no rule for which end to shoot. Sent back: it should define the window type and the shot rule the header gives.
