# ioi06p2

Score: 3/5
Dream: 4/5

## Cold review (HEAD)

Port 3/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-8) gives the plan, 2D prefix sums and a sliding-window minimum of chamber sums, and the port follows it, but in 0-based `Int` offsets into nested lists.
- `block(r, c, ...)` rebuilds `Row(r + 1)`/`Col(c + 1)` from offsets (lines 26-27), and `+ 1` recurs on lines 37, 42-43; in the friction (corners are plain offsets).
- The two 1D passes with the transpose on line 33 are in the friction (no 2D `window_fold`).
- Lines 39-41 search the chamber window a second time to recover where the minimum sits, because `down` keeps only its value; a window fold that keeps the argmin would say it once. Not in the friction.

## Dream notes

- The dream types corners as a domain and slides in 2D, but does not show how the chamber's position is recovered.
