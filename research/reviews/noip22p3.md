# noip22p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: two_edge_components and condense give the bridge tree, Rooted orders it, and the DP is three Mod tables; the halving uses one hoisted inverse. The inline read(gen:) generator moved to gen.py, the one place generators live.

## Checklist failures

- `var cities: [Block]Int = 0; for v in City: cities[block[v]] += 1` counts cities per block with a hand loop.
- `some`, `one` and `total` are three parallel DP tables over `Block` that have one comment line each to say what they count. A struct per block would keep the recurrence in one place.

## Gaps that need the language

- Counting per block is a hand loop: research/frictions/noip22p3.md.
