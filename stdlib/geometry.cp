//! Plane geometry on integer points. Every predicate is exact: no epsilons,
//! no floating point until you ask for a length. Coordinates up to about
//! 1e9 are safe (a cross product is a sum of two products).
use std.algebra.{Order}

/// A point or a vector with integer coordinates.
pub struct Pt(x: Int, y: Int):
    /// The squared length.
    pub @get fn norm2(self) -> Int = x * x + y * y

    /// The length, as a `Float`.
    pub @get fn len(self) -> Float = sqrt(Float(x * x + y * y))

    /// Rotated a quarter turn counter-clockwise.
    pub @get fn perp(self) -> Pt = Pt(-y, x)

impl Add for Pt:
    fn add(a, b) = Pt(a.x + b.x, a.y + b.y)

impl Sub for Pt:
    fn sub(a, b) = Pt(a.x - b.x, a.y - b.y)

impl Neg for Pt:
    fn neg(a) = Pt(-a.x, -a.y)

pub fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y

/// Positive when `b` is counter-clockwise from `a`.
pub fn cross(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x

/// Which way the path `a -> b -> c` turns: 1 left, -1 right, 0 straight.
pub fn turn(a: Pt, b: Pt, c: Pt) -> Int:
    let k = cross(b - a, c - a)
    if k > 0: 1 else: if k < 0: -1 else: 0

/// Left to right, then bottom to top.
pub const ByXY = Order<Pt>(lt: (a, b) => a.x < b.x or (a.x == b.x and a.y < b.y))

/// Counter-clockwise by direction, starting from the positive x axis. Exact;
/// vectors in the same direction compare equal, the zero vector comes first.
pub const ByAngle = Order<Pt>(lt: (a, b) => if half(a) != half(b): half(a) < half(b) else: cross(a, b) > 0)

/// 0 for the zero vector, 1 for directions in [0, pi), 2 for [pi, 2 pi).
fn half(a: Pt) -> Int:
    if a.x == 0 and a.y == 0: return 0
    if a.y > 0 or (a.y == 0 and a.x > 0): 1 else: 2

/// Is `p` on the closed segment from `a` to `b`?
pub fn on_segment(p: Pt, a: Pt, b: Pt) -> Bool = cross(a - p, b - p) == 0 and dot(a - p, b - p) <= 0

/// Do the closed segments `a b` and `c d` share a point? Touching counts.
pub fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt) -> Bool:
    let (t1, t2, t3, t4) = (turn(a, b, c), turn(a, b, d), turn(c, d, a), turn(c, d, b))
    if t1 * t2 < 0 and t3 * t4 < 0: return true
    on_segment(c, a, b) or on_segment(d, a, b) or on_segment(a, c, d) or on_segment(b, c, d)

/// Twice the signed area of a polygon: positive when the vertices run counter-clockwise. O(n).
pub fn area2(poly: List<Pt>) -> Int:
    var total = 0
    for i in 0..poly.len: total += cross(poly[i], poly[(i + 1) % poly.len])
    total

/// The convex hull, counter-clockwise from the lowest of the leftmost points,
/// without repeated or collinear points (Andrew's monotone chain). O(n log n).
pub fn hull(pts: List<Pt>) -> List<Pt>:
    var ps = copy(pts)
    sort(mut ps, by: ByXY)
    var uniq = List<Pt>()
    for p in ps:
        if uniq.is_empty or uniq.last != p: uniq.push(p)
    if uniq.len <= 2: return uniq
    var out = List<Pt>()
    for pass in 0..2:
        let base = out.len
        for p in (if pass == 0: uniq else: uniq.rev):
            while out.len >= base + 2 and turn(out[out.len - 2], out[out.len - 1], p) <= 0: out.pop()
            out.push(p)
        out.pop()
    out

/// Where `p` is relative to a simple polygon (either orientation):
/// 1 inside, 0 on the boundary, -1 outside. O(n), by counting crossings of a ray to the right.
pub fn locate(poly: List<Pt>, p: Pt) -> Int:
    var inside = false
    for i in 0..poly.len:
        var (a, b) = (poly[i], poly[(i + 1) % poly.len])
        if on_segment(p, a, b): return 0
        if a.y > b.y: swap(mut a, mut b)
        // the edge spans the ray's height (bottom end included, top end not) and p is left of it
        if a.y <= p.y and p.y < b.y and cross(a - p, b - p) > 0: inside = not inside
    if inside: 1 else: -1

// ---- tests

fn random_points(n: Int, span: Int) -> List<Pt> = [Pt(rand(-span, span), rand(-span, span)) for _ in 0..n]

@test
fn vectors_and_turns():
    let (a, b) = (Pt(3, 4), Pt(-1, 2))
    assert a + b == Pt(2, 6) and a - b == Pt(4, 2) and -a == Pt(-3, -4)
    assert dot(a, b) == 5 and cross(a, b) == 10 and a.norm2 == 25 and a.len == 5.0
    assert a.perp == Pt(-4, 3) and dot(a, a.perp) == 0
    assert turn(Pt(0, 0), Pt(1, 0), Pt(1, 1)) == 1
    assert turn(Pt(0, 0), Pt(1, 0), Pt(1, -1)) == -1
    assert turn(Pt(0, 0), Pt(1, 1), Pt(3, 3)) == 0

@test
fn angle_order_goes_once_around():
    var dirs = [Pt(1, -1), Pt(-1, 0), Pt(0, 1), Pt(1, 0), Pt(-2, -2), Pt(0, -3), Pt(1, 1), Pt(-1, 1)]
    sort(mut dirs, by: ByAngle)
    assert dirs == [Pt(1, 0), Pt(1, 1), Pt(0, 1), Pt(-1, 1), Pt(-1, 0), Pt(-2, -2), Pt(0, -3), Pt(1, -1)]
    assert laws_hold(ByAngle, dirs)
    assert laws_hold(ByXY, dirs)

@test
fn segments_touching_crossing_and_apart():
    let o = Pt(0, 0)
    assert segments_meet(o, Pt(4, 4), Pt(0, 4), Pt(4, 0))
    assert segments_meet(o, Pt(2, 2), Pt(2, 2), Pt(5, 0))
    assert segments_meet(o, Pt(4, 0), Pt(2, 0), Pt(6, 0))
    assert not segments_meet(o, Pt(1, 0), Pt(2, 0), Pt(3, 0))
    assert not segments_meet(o, Pt(4, 4), Pt(1, 0), Pt(5, 3))
    assert segments_meet(o, o, o, o) and not segments_meet(o, o, Pt(1, 1), Pt(1, 1))
    for _ in 0..200:
        let p = random_points(4, 3)
        let m = segments_meet(p[0], p[1], p[2], p[3])
        assert m == segments_meet(p[2], p[3], p[0], p[1]) and m == segments_meet(p[1], p[0], p[3], p[2])
        // brute force on a grid fine enough to hold every intersection of segments in -3..=3
        if p[0] != p[1] and p[2] != p[3] and cross(p[1] - p[0], p[3] - p[2]) != 0:
            // lines cross at p0 + t (p1 - p0), t = num / den
            let den = cross(p[1] - p[0], p[3] - p[2])
            let t = cross(p[2] - p[0], p[3] - p[2])
            let u = cross(p[2] - p[0], p[1] - p[0])
            let within = if den > 0: 0 <= t <= den and 0 <= u <= den else: den <= t <= 0 and den <= u <= 0
            assert m == within

@test
fn hull_is_convex_and_contains_every_point():
    assert hull([]) == [] and hull([Pt(1, 1), Pt(1, 1)]) == [Pt(1, 1)]
    assert hull([Pt(0, 0), Pt(2, 2), Pt(1, 1)]) == [Pt(0, 0), Pt(2, 2)]
    assert hull([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2), Pt(1, 1), Pt(1, 0)]) == [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
    for _ in 0..40:
        let pts = random_points(rand(3, 14), 5)
        let h = hull(pts)
        if h.len < 3: continue
        for i in 0..h.len:
            assert h[i] in pts
            assert turn(h[i], h[(i + 1) % h.len], h[(i + 2) % h.len]) == 1
        for p in pts:
            assert locate(h, p) >= 0
            assert (locate(h, p) == 0) == (sum(1 for i in 0..h.len if on_segment(p, h[i], h[(i + 1) % h.len])) > 0)
        assert area2(h) > 0 and area2(h.rev) == -area2(h)

@test
fn locate_agrees_with_half_planes_on_convex_polygons():
    for _ in 0..40:
        let h = hull(random_points(8, 4))
        if h.len < 3: continue
        for x in -5..=5:
            for y in -5..=5:
                let p = Pt(x, y)
                let turns = [turn(h[i], h[(i + 1) % h.len], p) for i in 0..h.len]
                let want = if min(turns) > 0: 1 else: if min(turns) == 0: 0 else: -1
                assert locate(h, p) == want
                assert locate(h.rev, p) == want
    // a concave polygon: an L shape
    let l = [Pt(0, 0), Pt(4, 0), Pt(4, 2), Pt(2, 2), Pt(2, 4), Pt(0, 4)]
    assert locate(l, Pt(1, 3)) == 1 and locate(l, Pt(3, 3)) == -1 and locate(l, Pt(2, 3)) == 0
    assert locate(l, Pt(3, 1)) == 1 and locate(l, Pt(5, 2)) == -1 and locate(l, Pt(-1, 0)) == -1
