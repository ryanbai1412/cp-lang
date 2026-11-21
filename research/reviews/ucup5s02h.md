# ucup5s02h

Score: pending (header derives the spanning-tree colouring and the hull; named `Pt` and gap records; one `ParityDsu` with a pin node replaces both DSUs and the `2i+1` encoding; mirrored sweep is one `sweep(order, sign)`; random generator added)
Dream: pending (new dream)

A convex-hull trick written over untyped `(Int, Int, Int)` tuples with `.0`/`.1` and two parity-encoded DSUs; no header. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- No header (line 1): nothing states what `cmp`/`cmp3` compare or why the hull is monotone.
- Positional tuples (lines 13-35): `pts[idx].0, pts[idx].1` instead of a named record.
- Index encoding (lines 48-67): `Bip.at(2 * i + 1)`, `2 * n`, `2 * n + 1` for a two-colour DSU; the merge pattern is repeated four times.
- `let _ = uf.merge(...)` discards on every line (lines 55-67).

## Owed rewrites

- Name the hull point (`struct Charge(x, q, id)`), give the bipartite DSU a parity-union API or a `(Node, Side)` domain, and write the header.

## Language gaps

No friction report exists. Missing: a parity (bipartite) DSU in the stdlib if the lane could not find one.
