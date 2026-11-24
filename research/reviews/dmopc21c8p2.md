# dmopc21c8p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Per parity of column, the blank columns and the allowed sums of the partly filled ones are counted, a difference array over `Sum` gives how many columns allow each sum, and the answer multiplies per parity.

## Checklist failures

- `allowed` is a difference array by hand (`+= 1` at `lo`, `-= 1` at `hi + 1`) with a `Sum` domain stretched one past the last value for it.
- `Parity(i.ord % 2)` derives the parity through ordinals.
- `blank` and `need` are two parallel `[Parity]Int` counters.
- `ways(x) = min(x - 1, 2 * m + 1 - x)` is commented but the reason for `2 * m + 1` is not.
