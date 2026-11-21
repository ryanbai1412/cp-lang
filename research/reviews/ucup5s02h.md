# ucup5s02h

Score: 3/5
Dream: 3/5

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

## Cold review (HEAD)

Dream 3/5 (new), port → 3/5, rescored on current main; sent back. The header derives the spanning-tree colouring and the hull's monotonicity.
- Bug: `(a.x - b.x) ^ 2` (line 24) is xor; §70 spells powers `**`.
- Owed: the dream calls `pull` (line 31), `never_weakest` (line 35) and `hull_pairs` (line 68), but defines none of them. Bring over the port's `never_weakest`, or name the stdlib facility a friction item asks for.
- Reported gaps: `Frac`, the `C | Pin` sum domain, `ParityDsu` component ids and count, `GroupTree.others`, `pop_while`.
- Minor: `colour.unite(c, Pin, ...)` (line 52) passes a `C` for a `Node` (E0301), which the sum domain would fix.

Port 3/5: the hull test is hand `I128` cross-multiplication (lines 21-30, 57-62; `Frac` is reported).

## Cold review (HEAD)

Dream 3/5, rescored cold on current main; sent back. `**` replaces `^`, and `pull`, `never_weakest` and `hull_pairs` are now defined.
- Owed: `never_weakest` (lines 30-35) is the port's hand `I128` cross-multiplication, in a dream that already has `Frac`. The condition `d * d >= 4 * a1 * a3` isn't derived from line 29's claim. Write it with `Frac`/`pull`, or derive the inequality.
- Owed: line 82's `max_by= (...), by: ...` doesn't parse (E0107), and line 84's `for e in best if let e:` isn't an accepted filter (E0103). Neither is in decisions.md or a friction item.
- Owed: line 88 passes `C.filter(...)`, a `List<C>`, where `hull_pairs` takes `List<Charge>` (E0302); map through `ch`.
- Reported gaps: `Frac`, the `C | Pin` sum domain (the `Pin`/`Node` errors), `ParityDsu` component ids and `side`, `GroupTree.others`, `min_by` and `pop_while`.

Port 3/5 unchanged: the hull test is the same hand `I128` algebra (`Frac` is reported).
