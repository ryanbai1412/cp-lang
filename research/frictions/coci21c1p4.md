# Friction report: coci21c1p4

Rethought. The v0.2 port kept the values of Z[ω] as two parallel Int arrays, unrolled the butterfly over six scalars, stepped the blocks with a `while`, and read n and k untyped. The v0.3 port names the value type `Eis = (r: Int, w: Int)` with `turn` (multiply by ω) and a three-way `plus`. Each digit's pass is `for base in (0..size).step(3 * stride)`, the size is `3 ** k`, and n and k are typed.

No language gap.

## Measurements

gen.py --max (k = 12, every card): port 0.10 s, accepted C++ 0.14 s, same output.
