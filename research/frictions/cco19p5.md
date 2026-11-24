# Friction Report: cco19p5

## SortedSet small-to-large is 10x the C++ set merge
- **Where:** `above[f].absorb(mut above[v])`.
- **What felt wrong:** the port is the dream's spelling, but at N = M = 1e5 it takes 0.60 s where the
  accepted C++ (`std::set` swap-and-insert) takes 0.06 s; `absorb` plus `first`/`remove` on the
  std ordered set cost far more than the red-black tree.
- **Ideal:** `SortedSet.absorb` at `std::set` speed (swap the larger in, insert the smaller).

## Dream
The committed `dream.cp`; the port now matches it except for reading `n` and `m` with bounds.

- Tokens: port 139, original 436
- Max input (`gen.py --max`): port 0.60 s, 20844 KB, original 0.06 s, 12800 KB
