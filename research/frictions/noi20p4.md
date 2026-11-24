# Friction Report: noi20p4

No friction left: the per-dish linear scans for the smallest and largest ingredient over a
rebuilt `live` list are one std `SortedBag` of (grams left, ingredient), read at `first` and
`last`; the ingredients are a domain, the case loop is `@multitest` with typed `n, m, k`, the
subset search is `halves() -> ?(List<Ing>, List<Ing>)` instead of a `continue` on -1, and the
grams are read as `[Ing]1..=m * k`.

Evidence: 30/30 stress against the v0.2 port and 30/30 against the accepted C++, both with the
plan checker; on the max case the port takes 0.58 s against 0.84 s for the C++, and the checker
accepts its plan.
