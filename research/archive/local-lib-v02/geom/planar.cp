//! Planar geometry recipes built on `std.geometry`:
//! polygon kernel via `half_plane_intersect`, convex polygon overlap via
//! `minkowski_sum` + `locate_convex`, and domain-indexed `closest_pair`.
use std.geometry.{
    Pt,
    Vec,
    HalfPlane,
    Location,
    convex_hull,
    minkowski_sum,
    locate_convex,
    closest_pair,
    half_plane_intersect,
    polygon_area,
    close,
}

/// Kernel of a CCW simple polygon: the set of points that see the whole interior. O(n log n).
pub fn polygon_kernel(poly: List<Vec>) -> List<Vec>:
    let n = poly.len
    if n < 3: return []
    let planes = [HalfPlane(poly[i], poly[(i + 1) % n] - poly[i]) for i in 0..n]
    half_plane_intersect(planes)

/// Whether two non-empty CCW convex polygons share at least one point. O(|a| + |b|).
pub fn convex_polygons_overlap(a: List<Pt>, b: List<Pt>) -> Bool:
    let neg_b = convex_hull([-p for p in b])
    let diff = minkowski_sum(a, neg_b)
    locate_convex(diff, Pt(0, 0)) != Location.Outside

@test
fn polygon_kernel_of_convex_and_star_polygons():
    // Convex square: kernel equals the whole square (area 4).
    let sq = [Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]
    let k_sq = polygon_kernel(sq)
    assert close(polygon_area(k_sq), 4.0, eps: 1e-6)
    // Concave arrow: kernel is non-empty and strictly smaller than the polygon.
    let arrow = [Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(2.0, 1.0), Vec(4.0, 2.0), Vec(0.0, 2.0)]
    let k_arrow = polygon_kernel(arrow)
    assert k_arrow.len >= 3 and polygon_area(k_arrow) > 0.0 and polygon_area(k_arrow) < polygon_area(arrow)

@test
fn minkowski_overlap_matches_point_witness():
    let a = [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
    let touching = [Pt(2, 1), Pt(4, 1), Pt(4, 3), Pt(2, 3)]
    let overlapping = [Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]
    let disjoint = [Pt(3, 0), Pt(5, 0), Pt(5, 2), Pt(3, 2)]
    assert convex_polygons_overlap(a, touching)
    assert convex_polygons_overlap(a, overlapping)
    assert not convex_polygons_overlap(a, disjoint)

@test
fn domain_closest_pair_matches_brute():
    domain P = 0..12
    let coords = [
        Pt(0, 0),
        Pt(10, 0),
        Pt(5, 5),
        Pt(2, 1),
        Pt(9, 8),
        Pt(3, 2),
        Pt(-4, 7),
        Pt(6, -3),
        Pt(2, 2),
        Pt(11, 11),
        Pt(-5, -5),
        Pt(8, 1),
    ]
    let pts: [P]Pt = [coords[p.to_idx] for p in P]
    let best = closest_pair(pts)
    var want = (pts[P(0)] - pts[P(1)]).norm2
    for i in P:
        for j in P:
            if i < j:
                let d = (pts[i] - pts[j]).norm2
                if d < want: want = d
    assert best.a != best.b
    assert best.dist2 == want
    assert (pts[best.a] - pts[best.b]).norm2 == want
