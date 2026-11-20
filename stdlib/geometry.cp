//! Plane geometry on integer points. Every predicate is exact: no epsilons,
//! no floating point until you ask for a length. Coordinates up to about
//! 1e9 are safe (a cross product is a sum of two products).

/// A point or a vector with integer coordinates.
pub struct Pt(x: Int, y: Int):
    /// The squared length.
    pub @get fn norm2(self) -> Int = x * x + y * y

    /// The length, as a `F64`.
    pub @get fn len(self) -> F64 = sqrt(F64(x * x + y * y))

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

/// Counter-clockwise by direction, starting from the positive x axis. Exact;
/// vectors in the same direction compare equal, the zero vector comes first.
pub struct ByAngle(p: Pt)

impl Ord for ByAngle:
    fn lt(a, b) = if half(a.p) != half(b.p): half(a.p) < half(b.p) else: cross(a.p, b.p) > 0

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

fn random_points(n: Int, span: Int) -> List<Pt> = [
    Pt(rand(-span, span), rand(-span, span)) for _ in 0..n
]

@test
fn vectors_and_turns():
    let (a, b) = (Pt(3, 4), Pt(-1, 2))
    assert a + b == Pt(2, 6) and a - b == Pt(4, 2) and -a == Pt(-3, -4)
    assert dot(a, b) == 5 and cross(a, b) == 10 and a.norm2 == 25 and abs(a.len - 5.0) < 1e-9
    assert a.perp == Pt(-4, 3) and dot(a, a.perp) == 0
    assert turn(Pt(0, 0), Pt(1, 0), Pt(1, 1)) == 1
    assert turn(Pt(0, 0), Pt(1, 0), Pt(1, -1)) == -1
    assert turn(Pt(0, 0), Pt(1, 1), Pt(3, 3)) == 0

@test
fn angle_order_goes_once_around():
    var dirs = [
        Pt(1, -1),
        Pt(-1, 0),
        Pt(0, 1),
        Pt(1, 0),
        Pt(-2, -2),
        Pt(0, -3),
        Pt(1, 1),
        Pt(-1, 1),
    ]
    sort(mut dirs, by: ByAngle)
    assert dirs == [
        Pt(1, 0),
        Pt(1, 1),
        Pt(0, 1),
        Pt(-1, 1),
        Pt(-1, 0),
        Pt(-2, -2),
        Pt(0, -3),
        Pt(1, -1),
    ]

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
        assert m == segments_meet(p[2], p[3], p[0], p[1]) and m == segments_meet(
            p[1],
            p[0],
            p[3],
            p[2],
        )
        // brute force on a grid fine enough to hold every intersection of segments in -3..=3
        if p[0] != p[1] and p[2] != p[3] and cross(p[1] - p[0], p[3] - p[2]) != 0:
            // lines cross at p0 + t (p1 - p0), t = num / den
            let den = cross(p[1] - p[0], p[3] - p[2])
            let t = cross(p[2] - p[0], p[3] - p[2])
            let u = cross(p[2] - p[0], p[1] - p[0])
            let within = if den > 0: 0 <= t <= den and 0 <= u <= den else: den <= t <= 0 and den <= u <= 0
            assert m == within

@test
fn area_and_locate_on_convex_and_concave_polygons():
    let poly = [Pt(0, 0), Pt(4, 0), Pt(5, 2), Pt(2, 4), Pt(-1, 2)]
    assert area2(poly) == 32 and area2(poly.rev) == -32
    for x in -2..=6:
        for y in -1..=5:
            let p = Pt(x, y)
            let turns = [turn(poly[i], poly[(i + 1) % poly.len], p) for i in 0..poly.len]
            let want = if min(turns)! > 0: 1 else: if min(turns)! == 0: 0 else: -1
            assert locate(poly, p) == want
            assert locate(poly.rev, p) == want
    // a concave polygon: an L shape
    let l = [Pt(0, 0), Pt(4, 0), Pt(4, 2), Pt(2, 2), Pt(2, 4), Pt(0, 4)]
    assert area2(l) == 24
    assert locate(l, Pt(1, 3)) == 1 and locate(l, Pt(3, 3)) == -1 and locate(l, Pt(2, 3)) == 0
    assert locate(l, Pt(3, 1)) == 1 and locate(l, Pt(5, 2)) == -1 and locate(l, Pt(-1, 0)) == -1
