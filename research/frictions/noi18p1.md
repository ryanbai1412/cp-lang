# Friction Report: noi18p1

Same problem as etsr23r3p3 (NOI 2018 Return); the port and dream follow that pair.

## 1. Edge weight by field name, not by projection
- **Where:** `Graph.undirected(roads)` relies on the row's `w` field; `kruskal_tree(roads, w: r => desc(roads[r].alt))`.
- **What felt wrong:** the row has to name its length `w` for Dijkstra, and the Kruskal key takes an edge index, so the
  altitude is re-looked-up by index.
- **Goal it hurts:** express.
- **Ideal code:** `Graph.undirected(roads).weighted(.len)`, `kruskal_tree(roads, by: desc(.alt))`.
- **General fix:** weight/key arguments take a field projection of the row (`.len`, `desc(.alt)`) everywhere a std graph
  builder takes rows.
- **Severity:** low.

## 2. The query reads as a weight bound, not as the condition
- **Where:** `kt.reach(v, desc(p + 1))`.
- **What felt wrong:** "roads with altitude > p" becomes `desc(p + 1)`, an off-by-one encoding of the condition.
- **Ideal code:** `kt.top(v, ok: road => road.alt > p)` with the road row passed to `ok`.
- **General fix:** `KruskalTree.top` with an `ok` over the merging edge's row (today `ok` takes the edge key).
- **Severity:** low.

## 3. Forced-online decode has no `ring` form
- **Where:** `V.nth((v0.ord + k * last) % n)` and `(p0 + k * last) % (s + 1)`.
- **What felt wrong:** two hand-written modular shifts for "step around the domain".
- **Ideal code:** `V.ring.step(v0, k * last)`, `(0..=s).ring.step(p0, k * last)`.
- **General fix:** `.ring` on any domain or range (positions as a cycle); repro in etsr23r3p3.md (E0501).
- **Severity:** low.

## Measurements
Stress: 30/30 agree with AC 6512702 on gen.py inputs. Max input (`gen.py --max`, 17.6 MB): port 0.43 s,
C++ 0.55 s, same output. The frozen v0.2 port no longer checks (E0603 lent `anc`), so the AC is the oracle.

## Dream
The ungated ideal (does not check today; the gaps above are the difference):

```cp
@!multitest
let (n, m) = read<(1..=2e5, 1..=4e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, len: 1..=1e4, alt: 1..=1e9)>()
let home = dijkstra(Graph.undirected(roads).weighted(.len), from: [V.first!]).dist
let (domain N, kt) = kruskal_tree(roads, by: desc(.alt))
let least = kt.tree.up(min, leaf: v => home[v]!)
let (q, k, s) = read<(1..=4e5, 0..=1, 0..=1e9)>()
var last = 0
for (v0, p0) in read<[q](V, 0..=s)>():
    let v = V.ring.step(v0, k * last)
    let p = (0..=s).ring.step(p0, k * last)
    last = least[kt.top(v, ok: road => road.alt > p)]
    print(last)
```
