# sac22cc3p4

Score: 5/5
Dream: 4/5 (the port as written, per the friction's `## Dream`)

Scored cold against the checklist and the port.

The port answers offline in reverse: walk the queries backwards, uniting on removals and answering a reachable pair with `tree.dist`, then print the answers in order. Typed reads, a std `Dsu<V>` and `Rooted`, and `?Int` answers printed with `?? -1`.

## Checklist failures

- None. `kind: 1..=2` is compared to `2` once; that is the statement's encoding, decoded at its only use.
