# Friction report: dmopc21c7p3

No gap. The cycle test is std `reachable` from the lit figures, then `topo` on the links leaving them, in place of a hand-written iterative DFS with colour codes, `head`/`nxt` edge chains and `-1` sentinels. The in-link parity is `fed[w.v] ^= up[w.u]`.

## Dream

```cp
let up: [Figure]Bool = read()            // the 0/1 flags
var fed: [Figure]Bool = false
for w in links: fed[w.v] ^= up[w.u]
...
else: print(count(fed[v] != up[v] for v in Figure))
```

The rest is the port as written.

## The figures' 0/1 states cannot be read as `[Figure]Bool`
- **Where:** `let up: [Figure]0..=1 = read()` and `var fed: [Figure]0..=1`, XORed as integers.
- **What felt wrong:** up and fed are booleans; the port carries them as 0/1 integers because `read()` of a `Bool` is E0315.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `[Figure]Bool` tables.
- **General fix:** `read()` of a `Bool` from a `0`/`1` token (the same gap as dmopc20c6p3). Owner: Core (12cfdb46).
- **Severity:** low.

## Measurements

gen.py --max (N = M = 1e6, acyclic): port 0.35 s, accepted C++ 0.78 s.
