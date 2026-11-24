# dmopc21c8p5

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Small widths get a direct DP; large ones are reached by levels of halving ranges, each level's costs filled from the one below.

## Checklist failures

- `below` and `below_lo` are a window into the previous level's costs, indexed by `x - below_lo` by hand.
- `half(x) = x / 2 - k / 2 - 1` and `base = 2 * k + 2` are bounds of the editorial's argument with only a short comment.
- `a[Deg(i + 1)]` and `a[Deg.first!]` convert between widths and degrees by hand.
- The inner `min(...)!` sits inside a comprehension that the formatter splits over four lines.
