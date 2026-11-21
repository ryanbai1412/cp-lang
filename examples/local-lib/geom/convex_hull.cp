//! Convex hull of integer points (Andrew's monotone chain).
use std.geometry.{Pt, turn, on_segment, area2, locate}
use std.geometry.{turn, on_segment, area2, locate}

/// The convex hull, counter-clockwise from the lowest of the leftmost points,
/// without repeated or collinear points (Andrew's monotone chain). O(n log n).
pub fn convex_hull(pts: List<Pt>) -> List<Pt>:
    var ps = copy(pts)
    ps.sort(by: p => (p.x, p.y))
    var uniq = List<Pt>()
    for p in ps:
        if uniq.is_empty or uniq.last != p: uniq.push(p)
    if uniq.len <= 2: return uniq
    var out = List<Pt>()
    for pass in 0..2:
        let base = out.len
        for p in (if pass == 0: uniq else: uniq.rev):
            while out.len >= base + 2 and turn(out[out.len - 2], out.last!, p) <= 0: out.pop()
            out.push(p)
        out.pop()
    out

fn random_points(n: Int, span: Int) -> List<Pt> = [
    Pt(rand(-span, span), rand(-span, span)) for _ in 0..n
]

@test
fn hull_is_convex_and_contains_every_point():
    assert convex_hull([]) == [] and convex_hull([Pt(1, 1), Pt(1, 1)]) == [Pt(1, 1)]
    assert convex_hull([Pt(0, 0), Pt(2, 2), Pt(1, 1)]) == [Pt(0, 0), Pt(2, 2)]
    assert convex_hull(
        [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2), Pt(1, 1), Pt(1, 0)]
    ) == [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
    for _ in 0..40:
        let pts = random_points(rand(3, 14), 5)
        let h = convex_hull(pts)
        if h.len < 3: continue
        for i in 0..h.len:
            assert h[i] in pts
            assert turn(h[i], h[(i + 1) % h.len], h[(i + 2) % h.len]) == 1
        for p in pts:
            assert locate(h, p) >= 0
            assert (locate(h, p) == 0) == (
                sum(1 for i in 0..h.len if on_segment(p, h[i], h[(i + 1) % h.len])) > 0
            )
        assert area2(h) > 0 and area2(h.rev) == -area2(h)

@test
fn locate_agrees_with_half_planes_on_random_hulls():
    for _ in 0..40:
        let h = convex_hull(random_points(8, 4))
        if h.len < 3: continue
        for x in -5..=5:
            for y in -5..=5:
                let p = Pt(x, y)
                let turns = [turn(h[i], h[(i + 1) % h.len], p) for i in 0..h.len]
                let want = if turns.min()! > 0: 1 else: if turns.min()! == 0: 0 else: -1
                assert locate(h, p) == want
                assert locate(h.rev, p) == want
