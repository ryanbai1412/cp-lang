# Friction report: dmopc21c2p2

No gap: the port is the dream (a permutation and its inverse over typed `Pos`/`Val` domains).

## Dream

The port as written. `perm` and its inverse `at_pos` are the editorial's two arrays; `swap_positions` is the one place both change.

## Measurements

gen.py 1 300000 (n = 300, q = 3e5, 3000 Q): port 0.08 s, accepted C++ 0.17 s.
