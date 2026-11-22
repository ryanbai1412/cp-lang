# std.geometry doc notes

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/geometry.cp` to the doc standard.

- `Pt.norm2`, `dot`, `cross`, `dist2`, `manhattan`, `chebyshev`, `area2`, `OnlineHull.area2`: chose to trap on `Int` overflow (the checked-arithmetic default), pointing to the `_128` forms in the prose.
- `Pt.*_128`: trap only if `I128` itself overflows (coordinates near `Int.MIN`), rather than claiming "never".
- `Pt.angle`: the zero vector gives 0.0 (`atan2(0, 0)`).
- `angle_less`: traps when `from` is the zero vector (no reference direction); directions along `from` come first, zero vector before everything.
- `arg`: same-direction points tie, so a stable sort keeps their input order.
- `on_segment`: a degenerate segment `a == b` contains only `a`, and nothing when `strict`.
- `lower_hull`, `upper_hull`, `*_hull_idx`: chose to trap on unsorted input (an O(n) check, same order as the work) instead of returning garbage; equal points count once.
- `convex_hull` with `collinear: true` on all-collinear input: every point once, in order from the least `(x, y)` (the usual monotone chain would list interior points twice).
- `OnlineHull`: keeps strict corners only; `add` on a boundary or duplicate point returns false; `extreme` ties go to the first corner in `vertices()` order; `vertices`/`iter` start at the least `(x, y)`; an empty hull locates everything `Outside`.
- `minkowski_sum`: result starts at the least `(y, x)` corner (the stub's private `min_yx_idx` suggests this) with collinear points dropped; `minkowski_upper`/`lower` drop collinear points too.
- `convex_polygon_intersect`: chose that an intersection of area at most `eps` (touching at a point or edge) is `[]`, vertices within `eps` merge, and the result starts at the least `(y, x)` corner.
- `locate`: an empty polygon gives `Outside`; boundary is tested before the crossing rule. `locate_convex`: one- or two-point hulls have only their points/segment on the `Boundary`.
- `diameter2`, `hull_width`, `min_bounding_box`: fewer than 3 vertices give the degenerate answer (0, 0.0, or a zero-area box) rather than trapping.
- `min_bounding_box`: area and perimeter minimized independently (both over the `n` edge orientations).
- `hull_extreme`: ties go to the smallest index; empty traps.
- `hull_tangents`: when `p` lies on an edge's line, the nearer of the two qualifying vertices.
- `closest_pair`, `farthest_pair`: `a` before `b` in key order; ties go to the first pair in key order. `closest_pair2` returns 0 for equal points at distinct positions.
- `hull_line_intersect`: edges as `(i, j)` with `i < j`; a line that only touches gives `None`; traps when `a == b`.
- `manhattan_mst`: chose the octant-nearest candidate set, each pair once with `u` before `v`, sorted by `(w, u, v)`. Open: ties for "nearest in an octant" are still not pinned, so two implementations can differ in the candidate set (not in the MST weight).
- `polygon_centroid`: either orientation; traps on zero area.
- `polygon_kernel`: area at most `eps` is `[]`; starts at the least `(y, x)` corner.
- `polygon_triangulate`: open — the stated O(n log n) cost rules out the simple "lowest-index ear" rule that would pin the triangles, so the docs promise only `n - 2` counter-clockwise triples in a deterministic order. Needs a decision on the algorithm.
- `pick_interior`: traps on fewer than 3 vertices (Pick's formula is meaningless there); `pick_boundary` counts the closing edge, 0 for empty.
- `close`, `sign_eps`, `turn_eps`, `line_intersect`, `circumcenter`: every `eps` is an absolute tolerance on the quantity named (difference, raw cross product), not relative or angular.
- `Vec.angle`: a `-0.0` `y` is treated as `0.0` so the range really is `(-pi, pi]`; zero vector gives 0.0.
- `Vec.unit`: pinned to dividing each coordinate by `len` (exact `Vec(0.6, 0.8)` for `(3, 4)`); traps on zero length ("asserts").
- `Vec.rot`: `by` need not be unit; it scales by `|by|`.
- `angle_between`: 0.0 when either vector is zero (`atan2(0, 0)`).
- `project`, `reflect`, `line_dist`: trap when `a == b` (no line). `seg_dist`: a segment shorter than `eps` is the point `a`.
- `segment_intersect`: points sorted by `(x, y)`; one point for touching, two for collinear overlap.
- `ray_hits_segment`: collinear overlap gives the segment point nearest `o`; traps on a zero `dir`.
- `Circle.area`, `Circle.circumference`: pinned evaluation order (`PI * r * r`, `2.0 * PI * r`) so results are bit-identical.
- `Circle.locate`: `Boundary` when `|d - r| <= eps`, tested before inside/outside.
- `incenter`: pinned the centre (side-weighted mean) and radius formulas so results are bit-identical.
- `circle_line_intersect`, `circle_intersect`, `circle_tangents`, `circle_circle_tangents`: fixed result orders (along `a -> b`; or the point clockwise of the centre ray first); tangency within `eps` gives one point; identical circles give `[]`.
- `circle_circle_area`: containment uses the smaller disk's `area` exactly.
- `invert`: traps when `p` is the centre.
- `circles_union_area`: equal circles count once; empty gives 0.0.
- `HalfPlane.through`, `from_line`: trap on a degenerate line; `from_line` pins `p` (nearest point to the origin) and `dir = (b, -a)` so the kept side is the `>= 0` side.
- `half_plane_intersect`: unbounded, empty and zero-area (`<= eps`) results are all `[]`; parallel planes keep the tighter; result starts at the least `(y, x)`; traps on a zero `dir`.
- `polygon_cut`: Sutherland-Hodgman walk from `poly[0]`; points within `eps` of the line are kept; result keeps the input orientation.
- `min_enclosing_circle`: the stub says "deterministic Welzl" with "O(n) expected"; chose input order with no internal shuffle, which makes the expectation hold only for unadversarial order. Contradiction-adjacent: a fixed internal seed would give true expected O(n) and stay deterministic; logged for a decision.
- `circle_polygon_area`: signed by the polygon's orientation; empty gives 0.0.
