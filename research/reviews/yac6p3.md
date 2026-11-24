# yac6p3 review

Score: 4/5
Dream: 3/5

Cold review: the four corner answers are named for what they are (least x + y, least y - x,
least x - y, greatest x + y) instead of a/b/c/d. One `cell(sum, diff) -> ?Cell` turns a sum and
a difference into a grid cell or None (parity or off the grid), so the hand range check and the
`lower_pair` flag are gone: the fifth question is an `if let ... , near(first) == 0` guard and
both pairings read as cell(sum, diff) twice. Queries take a typed `Cell`.

## Cold review (HEAD)

Port 4/5, dream 3/5. The port reads better than its dream. It names the four corner answers (`low_sum`, `low_y_minus_x`, ...) and turns a sum/difference pair into a cell through `cell(sum, diff) -> ?Cell` with `X.at`/`Y.at` (lines 25-27). The dream uses bare `a, b, c, d` and repeats the halving arithmetic in its `print`.
- `near` (lines 20-22) needs `let d: 0..=2e9 = ask(...)` to type the reply. That's the E0501 gap, Owner: core, in the friction.
- `cell(...)!` (lines 36-38) unwraps cells the judge guarantees. Fine, since the input is non-adaptive.
- Dream: the four magic-letter bindings and the inline `d - (d - c) / 2` arithmetic hide the pairing argument the port spells out. It should be rewritten from the port.
