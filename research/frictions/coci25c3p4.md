# Friction report: coci25c3p4

Rethought lightly. The v0.2 port multiplied polynomials with its own `times` double loop. The v0.3 port uses std `convolve`, which is the O(nm) product for `Mod<1e9 + 7>` (not an NTT prime), so the O(36 n^2) bound holds.

## Gap

`without` still divides out a factor that starts with 1 by hand. std `div_mod` goes through a series inverse built on NTT products, which do not fit P = 1e9 + 7. An exact division (or a series inverse) that falls back to the O(nm) product for any modulus would let the rerooting step be `around[p] / down[v]`.

## Dream

```
if let p = tree.parent(v): around[v] *= hung(around[p] / down[v])
```

## Measurements

gen.py --max (n = 3000, depth 17): port 0.17 s, v0.2 port 0.26 s, accepted C++ 0.34 s, same answers. The C++ is 43.7 KB, over the generic 32 KiB target (W0201), the same warning the v0.2 port had.
