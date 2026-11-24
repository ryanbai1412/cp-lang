# Friction report: dmopc21c8p5

No gap. The v0.2 port memoised the split recursion in a `Map<Int, Int>` (15 s at max once rewritten in v0.3, 7.9 s for the old port). The weights each depth needs form a window of about 2K, so the port fills one list per depth bottom-up: 0.40 s. Bounds are types and the degree costs are a `[Deg]` table.

## Measurements

gen.py --max (N = 1e9, K = 3000): port 0.40 s, accepted C++ 1.39 s.
