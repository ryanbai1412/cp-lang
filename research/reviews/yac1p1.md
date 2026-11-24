# yac1p1

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against the friction's dream ("the port as written"), without the C++.
The header derives the exchange argument (pair in sorted order; every increment goes to the top
pair, first levelling its sides, then split evenly) and the cost. The code is that argument:
two `sorted()` reads, the levelling `min(k, abs(x - y))`, the even split, and one `Mod<p>` sum.

## Checklist failures
- None that a contestant would change.

## Owed rewrites
- None.

## Language gaps
- None.

## Dream notes
The dream is the port. 4/5 rather than 5: `sum(Z(a[i]) * Z(b[i]) for i in 0..n - 1) + Z(x) * Z(y)`
reads the "all pairs but the top" split through an index range; it is clear, but a zip of
`a` and `b` with the top pair replaced would state it without `i`.
