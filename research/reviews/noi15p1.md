# noi15p1

Score: 5/5
Dream: 5/5

Scored cold against the checklist, the dream and the port.

The port is the dream: one typed read of the constraints, `compress` over both sides into a domain `X`, a `Dsu<X>` united on the equalities, and `yes(...)` over the inequalities. It agrees with the v0.2 port on 30 stress cases and passes 12 package cases.

## Checklist failures

- None. `at` is a one-line closure over `ids.index_of`, and it is the only helper. Every read carries its statement bound, and there are no sentinels or index arithmetic.
