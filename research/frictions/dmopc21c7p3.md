# Friction report: dmopc21c7p3

No gap. The cycle test is std `reachable` from the lit figures, then `topo` on the links leaving them, in place of a hand-written iterative DFS with colour codes, `head`/`nxt` edge chains and `-1` sentinels. The in-link parity is `fed[w.to] ^= up[w.from]`.

## Measurements

gen.py --max (N = M = 1e6, acyclic): port 0.35 s, accepted C++ 0.78 s.
