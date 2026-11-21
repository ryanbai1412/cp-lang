# Geometry benchmarks

Matched-algorithm comparison of `std.geometry` algorithms against hand-written
C++ on identical deterministic inputs (`python3 bench/geometry/run.py --runs 7`).
Each pair reads the same input, runs the same algorithm, checks output equality,
and reports best-of-7 wall time (`g++ -std=c++17 -O2`).

| bench | input | cpl (s) | hand C++ (s) | ratio |
|---|---|---:|---:|---:|
| convex_hull | 200k points | 0.035 | 0.033 | 1.06x |
| half_plane_intersect | 100k half-planes | 0.056 | 0.068 | 0.81x |
| closest_pair | 100k points | 0.030 | 0.028 | 1.07x |
| any_segments_meet | 100k segments | 0.129 | 0.122 | 1.06x |
| convex_hull_3d | 20k 3D points | 0.028 | 0.025 | 1.09x |

All five benchmarks are under the 1.1x target. Profiling the initial runs found
and fixed three stdlib hot-path costs:

- `closest_pair` (1.28x -> 1.07x): switched `ord` to flat `(x, y, id)` tuples
  for direct `std::sort` instead of `cpl::sort_by`, and inlined `dx*dx + dy*dy`.
- `any_segments_meet` (1.28x -> 1.06x): hoisted treap helpers out of nested
  functions (avoiding `std::function` indirect calls), replaced `sh_split_le`
  with comparison-free `sh_pop_leftmost`, and narrowed `I128(b.x - a.x)` so GCC
  emits 64x64->128 multiplies.
- `convex_hull_3d` (1.87x -> 1.09x): replaced the O(n^2) `mark` table with
  O(h) horizon-edge deduplication and an early `any_vis` skip, used
  `I128(b.x - a.x)` in `face_side128`, and indexed points through a local
  `domain V = 0..n` so inner-loop face checks skip bounds checks.
