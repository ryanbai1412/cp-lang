# ioi00p4

Score: 3/5
Dream: 3/5

## Cold review (HEAD)

Port 3/5 (provisional: the dream is under 4), dream 3/5. Regions adjacent by a shared wall, std `floyd_warshall` on the `?Int` hop matrix, then each region's sum over members of the nearest region at their town, `argmin` (lines 20-40): the header's plan.
- Line 28 wraps the border by hand with `border[Side.wrap(s.ord + 1)]`; std `cyclic_pairs` (std/seq.cp) is what the dream uses on line 14, and the port owes it.
- `let len: Int = read()` (line 24) has no bound; a border has at most n towns.
- `hops[r, h]!` (line 37) unwraps on the plane graph being connected; in the friction.

## Dream notes

- dream.cp writes the wall key `(min(a, b), max(a, b))` twice (lines 16 and 20) and the symmetric `hops` assignment as two lines (17-18); the port already says each once (lines 30, 32). A fact twice caps the dream at 3.
- The friction's own dream (`Graph.undirected(Region, ...)`, `all_pairs_bfs`) is the better target, but it is not dream.cp. dream.cp has no `Source:`/`Problem:`/`Solution:` header.
