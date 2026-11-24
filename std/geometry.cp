//! Plane and 3D geometry on exact integer points (`Pt`, `Pt3`) and careful
//! floating-point vectors (`Vec`). Exact predicates compute cross and dot signs
//! in `I128` so integer geometry is the default overflow-free path:
//!
//! - `|x|, |y| <= 1e9` (e.g. `Pt<-1e9..=1e9>`): differences in `[-2e9, 2e9]`;
//!   single-vector `dot`/`cross` <= `2e18` fit `Int`, `turn`/`cross128`/`dot128`
//!   <= `8e18` fit `I128` with 19 orders of magnitude to spare, and `any_segments_meet`
//!   sweep comparisons <= `1.6e28` fit `I128`.
//! - `|x|, |y| <= 2e18`: `turn`/`cross128`/`dot128`/`dist2_128` <= `8e36` fit `I128`.
//! - 3D `|x|, |y|, |z| <= 1e9`: `cross3` components <= `8e18` fit `Int` (or `I128`
//!   up to `1e12`), and `convex_hull_3d` orientation tests <= `4.8e28` fit `I128`.

use std.bag.{SortedSet}
use std.seq.{collect}
use std.range.{first_true, last_true}

fn igcd(a0: Int, b0: Int) -> Int:
    var a = a0
    var b = b0
    while b != 0:
        let t = a % b
        a = b
        b = t
    a

// ---- exact integer points

/// \(O(1)\) value type. A 2D point or vector with exact integer coordinates.
pub struct Pt(pub x: Int, pub y: Int):
    /// The squared length `x*x + y*y`, in `Int`.
    ///
    /// Fits for `|x|, |y| <= 2e9`; use `norm2_128` beyond.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt(3, 4).norm2 == 25`
    /// example: `Pt(0, 0).norm2 == 0`
    @get pub fn norm2(self) -> Int = x * x + y * y

    /// The exact squared length `x*x + y*y`, computed in `I128`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `I128` (only for coordinates near `Int.MIN`)
    /// example: `Pt(3, 4).norm2_128 == 25`
    /// example: `Pt(3_000_000_000, 4_000_000_000).norm2_128 == I128(5_000_000_000) * 5_000_000_000`
    @get pub fn norm2_128(self) -> I128 = I128(x) * I128(x) + I128(y) * I128(y)

    /// The Euclidean length `sqrt(x*x + y*y)` as `F64` (`hypot`, so no intermediate overflow).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt(3, 4).len == 5.0`
    /// example: `Pt(0, 0).len == 0.0`
    @get pub fn len(self) -> F64 = sqrt(F64(self.norm2_128))

    /// `self` rotated 90 degrees counter-clockwise: `Pt(-y, x)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `y` is `Int.MIN`
    /// example: `Pt(1, 2).perp == Pt(-2, 1)`
    @get pub fn perp(self) -> Pt = Pt(-y, x)

    /// `self` rotated 90 degrees clockwise: `Pt(y, -x)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `x` is `Int.MIN`
    /// example: `Pt(1, 2).perp_cw == Pt(2, -1)`
    @get pub fn perp_cw(self) -> Pt = Pt(y, -x)

    /// The polar angle `atan2(y, x)` in `(-pi, pi]`; 0 for the zero vector.
    ///
    /// For exact angular comparisons use `arg` or `angle_less` instead.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt(1, 0).angle == 0.0`
    /// example: `Pt(0, -1).angle < 0.0`
    /// example: `Pt(-1, 0).angle > 3.14`
    /// example: `Pt(0, 0).angle == 0.0`
    @get pub fn angle(self) -> F64 = atan2(F64(y), F64(x))

    /// The reduced integer direction `(x / g, y / g)` with `g = gcd(|x|, |y|)`; `Pt(0, 0)` for the
    /// zero vector.
    ///
    /// Two nonzero vectors point the same way exactly when their `unit_dir`s are equal, so it is a
    /// hashable direction key.
    ///
    /// cost: \(O(\log \max(\lvert x \rvert, \lvert y \rvert))\)
    /// traps: never
    /// example: `Pt(4, -6).unit_dir() == Pt(2, -3)`
    /// example: `Pt(0, 5).unit_dir() == Pt(0, 1)`
    /// example: `Pt(0, 0).unit_dir() == Pt(0, 0)`
    pub fn unit_dir(self) -> Pt:
        if x == 0 and y == 0: return Pt(0, 0)
        let g = igcd(abs(x), abs(y))
        Pt(x / g, y / g)

    /// `self` as a floating-point `Vec(F64(x), F64(y))`: the explicit way across.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt(1, 2).to_vec == Vec(1.0, 2.0)`
    @get pub fn to_vec(self) -> Vec = Vec(F64(x), F64(y))

    /// The dot product `x * o.x + y * o.y`, in `Int`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result or a product overflows `Int`
    /// example: `Pt(1, 2).dot(Pt(3, 4)) == 11`
    /// example: `Pt(1, 0).dot(Pt(0, 1)) == 0`
    pub fn dot(self, o: Pt) -> Int = x * o.x + y * o.y

    /// The exact dot product `x * o.x + y * o.y`, computed in `I128`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the sum overflows `I128` (only when every coordinate is near `Int.MIN`)
    /// example: `Pt(1, 2).dot128(Pt(3, 4)) == 11`
    pub fn dot128(self, o: Pt) -> I128 = I128(x) * I128(o.x) + I128(y) * I128(o.y)

    /// The 2D cross product `x * o.y - y * o.x`, in `Int`: positive when `o` is counter-clockwise
    /// from `self`, negative when clockwise, 0 when parallel.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result or a product overflows `Int`
    /// example: `Pt(1, 0).cross(Pt(0, 1)) == 1`
    /// example: `Pt(1, 2).cross(Pt(3, 4)) == -2`
    /// example: `Pt(1, 2).cross(Pt(2, 4)) == 0`
    pub fn cross(self, o: Pt) -> Int = x * o.y - y * o.x

    /// The exact 2D cross product `x * o.y - y * o.x`, computed in `I128`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the difference overflows `I128` (only for coordinates near `Int.MIN`)
    /// example: `Pt(1, 2).cross128(Pt(3, 4)) == -2`
    pub fn cross128(self, o: Pt) -> I128 = I128(x) * I128(o.y) - I128(y) * I128(o.x)

    /// The squared Euclidean distance from `self` to `o`, in `Int`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt(0, 0).dist2(Pt(3, 4)) == 25`
    /// example: `Pt(1, 1).dist2(Pt(1, 1)) == 0`
    pub fn dist2(self, o: Pt) -> Int = (o - self).norm2

    /// The exact squared Euclidean distance from `self` to `o`, computed in `I128`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `I128` (coordinates beyond about `4e18` apart)
    /// example: `Pt(0, 0).dist2_128(Pt(3, 4)) == 25`
    pub fn dist2_128(
        self,
        o: Pt,
    ) -> I128 = (I128(o.x) - I128(x)) * (I128(o.x) - I128(x)) + (I128(o.y) - I128(y)) * (I128(o.y) - I128(y))

    /// The Euclidean distance from `self` to `o`, as `F64`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt(0, 0).dist(Pt(3, 4)) == 5.0`
    pub fn dist(self, o: Pt) -> F64 = sqrt(F64(self.dist2_128(o)))

    /// The Manhattan (L1) distance `|x - o.x| + |y - o.y|`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt(1, 1).manhattan(Pt(4, -3)) == 7`
    pub fn manhattan(self, o: Pt) -> Int = abs(o.x - x) + abs(o.y - y)

    /// The Chebyshev (L-infinity) distance `max(|x - o.x|, |y - o.y|)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt(1, 1).chebyshev(Pt(4, -3)) == 4`
    pub fn chebyshev(self, o: Pt) -> Int = max(abs(o.x - x), abs(o.y - y))

impl Add for Pt:
    fn add(a, b) = Pt(a.x + b.x, a.y + b.y)

impl Sub for Pt:
    fn sub(a, b) = Pt(a.x - b.x, a.y - b.y)

impl Neg for Pt:
    fn neg(a) = Pt(-a.x, -a.y)

impl Mul<Int> for Pt:
    fn mul(a, k) = Pt(a.x * k, a.y * k)

impl Show for Pt:
    fn show(a) = "{a.x} {a.y}"

fn cross128_3(a: Pt, b: Pt, c: Pt) -> I128:
    (I128(b.x) - I128(a.x)) * (I128(c.y) - I128(a.y)) - (I128(b.y) - I128(a.y)) * (I128(c.x) - I128(a.x))

fn dot128_3(a: Pt, b: Pt, c: Pt) -> I128:
    (I128(b.x) - I128(a.x)) * (I128(c.x) - I128(a.x)) + (I128(b.y) - I128(a.y)) * (I128(c.y) - I128(a.y))

/// Which way the path `a -> b -> c` turns: 1 left (counter-clockwise), -1 right, 0 straight.
///
/// The sign of `(b - a).cross(c - b)`, computed exactly in `I128`, so it is exact for
/// coordinates up to `1e18`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `turn(Pt(0, 0), Pt(1, 0), Pt(1, 1)) == 1`
/// example: `turn(Pt(0, 0), Pt(1, 0), Pt(1, -1)) == -1`
/// example: `turn(Pt(0, 0), Pt(1, 0), Pt(2, 0)) == 0`
pub fn turn(a: Pt, b: Pt, c: Pt) -> Int:
    let k = cross128_3(a, b, c)
    if k > I128(0): 1 else: if k < I128(0): -1 else: 0

/// Whether `a` and `b` are both nonzero and point the same way: parallel with a positive dot
/// product, exact in `I128`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `same_dir(Pt(1, 2), Pt(2, 4)) == true`
/// example: `same_dir(Pt(1, 2), Pt(-1, -2)) == false`
/// example: `same_dir(Pt(0, 0), Pt(0, 0)) == false`
pub fn same_dir(a: Pt, b: Pt) -> Bool = cross128(a, b) == I128(0) and dot128(a, b) > I128(0)

// ---- angular sort

/// 0 for the zero vector, 1 for directions in `[0, pi)`, 2 for `[pi, 2*pi)`.
fn half(a: Pt) -> Int:
    if a.x == 0 and a.y == 0: return 0
    if a.y > 0 or (a.y == 0 and a.x > 0): 1 else: 2

fn half_from(a: Pt, from: Pt) -> Int:
    if a == Pt(0, 0): return 0
    let c = from.cross128(a)
    if c > I128(0) or (c == I128(0) and from.dot128(a) > I128(0)): 1 else: 2

/// Whether `a` comes before `b` counter-clockwise, starting from the ray `from` (default `+x`).
///
/// The order is exact in `I128`: the zero vector comes first, then directions by angle in
/// `[0, 2*pi)` measured from `from`. Vectors in the same direction compare equal, so neither is
/// less than the other.
///
/// cost: \(O(1)\)
/// traps: when `from` is the zero vector
/// example: `angle_less(Pt(1, 0), Pt(0, 1)) == true`
/// example: `angle_less(Pt(0, -1), Pt(-1, 0)) == false`
/// example: `angle_less(Pt(0, 0), Pt(1, 0)) == true`
/// example: `angle_less(Pt(2, 0), Pt(1, 0)) == false`
/// example: `angle_less(Pt(-1, 0), Pt(1, 0), from: Pt(0, 1)) == true`
pub fn angle_less(a: Pt, b: Pt, from: Pt = Pt(1, 0)) -> Bool:
    assert from != Pt(0, 0), "`angle_less` needs a nonzero `from`"
    let ha = half_from(a, from)
    let hb = half_from(b, from)
    if ha != hb: return ha < hb
    ha != 0 and a.cross128(b) > I128(0)

/// \(O(1)\) comparison key. Counter-clockwise by direction, starting from the positive x axis.
/// Exact in `I128`; vectors in the same direction compare equal, zero comes first.
pub struct ByAngle(p: Pt)

impl Ord for ByAngle:
    fn lt(a, b) = angle_less(a.p, b.p)

impl Eq for ByAngle:
    fn eq(a, b) = not angle_less(a.p, b.p) and not angle_less(b.p, a.p)

/// \(O(1)\) comparison key. Counter-clockwise by direction starting from ray `base`.
/// Exact in `I128`; vectors in the same direction compare equal, zero comes first.
pub struct ByAngleFrom(base: Pt, p: Pt)

impl Ord for ByAngleFrom:
    fn lt(a, b) = angle_less(a.p, b.p, from: a.base)

impl Eq for ByAngleFrom:
    fn eq(a, b) = not angle_less(a.p, b.p, from: a.base) and not angle_less(b.p, a.p, from: a.base)

/// An exact, float-free angular sort key for `p`: `ByAngle(p)`, ordered as `angle_less` from `+x`.
///
/// Use `sort(mut pts, by: arg)`, or `sort(mut pts, by: p => arg(p - base))` around another
/// centre. Points in the same direction tie, so a stable sort keeps their input order.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `var pts = [Pt(0, 1), Pt(-1, 0), Pt(1, 0)]; sort(mut pts, by: arg); pts == [Pt(1, 0), Pt(0, 1), Pt(-1, 0)]`
/// example: `arg(Pt(1, 1)) == arg(Pt(2, 2))`
pub fn arg(p: Pt) -> ByAngle = ByAngle(p)

// ---- segments

/// Whether `p` lies on the closed segment from `a` to `b`, exact in `I128`.
///
/// With `strict: true` the endpoints do not count. A degenerate segment `a == b` contains only
/// `a`, and nothing when strict.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `on_segment(Pt(1, 1), Pt(0, 0), Pt(2, 2)) == true`
/// example: `on_segment(Pt(0, 0), Pt(0, 0), Pt(2, 2), strict: true) == false`
/// example: `on_segment(Pt(3, 3), Pt(0, 0), Pt(2, 2)) == false`
pub fn on_segment(p: Pt, a: Pt, b: Pt, strict: Bool = false) -> Bool:
    if cross128_3(p, a, b) != I128(0): return false
    let d = dot128_3(p, a, b)
    if strict: d < I128(0) else: d <= I128(0)

/// Whether the closed segments `a b` and `c d` share a point, exact in `I128`.
///
/// With `strict: true` they must cross properly, at one point interior to both: touching at an
/// endpoint and collinear overlap do not count.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `segments_meet(Pt(0, 0), Pt(2, 2), Pt(0, 2), Pt(2, 0)) == true`
/// example: `segments_meet(Pt(0, 0), Pt(1, 1), Pt(1, 1), Pt(2, 0)) == true`
/// example: `segments_meet(Pt(0, 0), Pt(1, 1), Pt(1, 1), Pt(2, 0), strict: true) == false`
/// example: `segments_meet(Pt(0, 0), Pt(1, 0), Pt(0, 1), Pt(1, 1)) == false`
pub fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool:
    let (t1, t2, t3, t4) = (turn(a, b, c), turn(a, b, d), turn(c, d, a), turn(c, d, b))
    if t1 * t2 < 0 and t3 * t4 < 0: return true
    if strict: return false
    on_segment(c, a, b) or on_segment(d, a, b) or on_segment(a, c, d) or on_segment(b, c, d)

// ---- convex hull and Minkowski sum

/// The lower convex hull chain, left to right, of points sorted by `(x, y)` (Andrew's monotone
/// chain).
///
/// Equal points count once. With `collinear: true` points on the chain's straight stretches are
/// kept; otherwise only the corners. One point gives itself.
///
/// cost: \(O(n)\) time and memory
/// empty: `[]`
/// order: left to right, by `(x, y)`; a new `List`, keyed from 0
/// ties: equal points count once; with `collinear: false` the chain keeps only the two ends of a straight stretch
/// traps: when `sorted_pts` is not sorted by `(x, y)`
/// example: `lower_hull([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [Pt(0, 0), Pt(1, -1), Pt(2, 0)]`
/// example: `lower_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0)]) == [Pt(0, 0), Pt(2, 0)]`
/// example: `lower_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0)], collinear: true) == [Pt(0, 0), Pt(1, 0), Pt(2, 0)]`
pub fn lower_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt>:
    var h: List<Pt> = []
    for p in sorted_pts:
        if h.len > 0:
            assert h.last! <= p, "`lower_hull` needs `sorted_pts` sorted by `(x, y)`"
            if h.last! == p: continue
        while h.len >= 2:
            let t = turn(h[h.len - 2], h.last!, p)
            if t < 0 or (not collinear and t == 0): h.pop()
            else: break
        h.push(p)
    h

/// The upper convex hull chain, left to right, of points sorted by `(x, y)`; the mirror of
/// `lower_hull`, with the same rules for duplicates and `collinear`.
///
/// cost: \(O(n)\) time and memory
/// empty: `[]`
/// order: left to right, by `(x, y)`; a new `List`, keyed from 0
/// ties: equal points count once; with `collinear: false` the chain keeps only the two ends of a straight stretch
/// traps: when `sorted_pts` is not sorted by `(x, y)`
/// example: `upper_hull([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [Pt(0, 0), Pt(1, 1), Pt(2, 0)]`
pub fn upper_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt>:
    var h: List<Pt> = []
    for p in sorted_pts:
        if h.len > 0:
            assert h.last! <= p, "`upper_hull` needs `sorted_pts` sorted by `(x, y)`"
            if h.last! == p: continue
        while h.len >= 2:
            let t = turn(h[h.len - 2], h.last!, p)
            if t > 0 or (not collinear and t == 0): h.pop()
            else: break
        h.push(p)
    h

/// The convex hull of `pts`, counter-clockwise from the least `(x, y)` point.
///
/// Equal points count once. With `collinear: true` boundary points on edges are kept, each once;
/// otherwise only the corners. When all points are collinear the hull is the two ends (or, with
/// `collinear: true`, every point in order from the least); one distinct point gives `[p]`.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]`
/// order: counter-clockwise, starting at the least `(x, y)`; a new `List`, keyed from 0
/// traps: never
/// example: `convex_hull([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2), Pt(1, 1)]) == [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]`
/// example: `convex_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], collinear: true) == [Pt(0, 0), Pt(1, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]`
/// example: `convex_hull([Pt(2, 2), Pt(1, 1), Pt(0, 0)]) == [Pt(0, 0), Pt(2, 2)]`
/// example: `convex_hull([Pt(5, 5), Pt(5, 5)]) == [Pt(5, 5)]`
pub fn convex_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt>:
    if pts.len <= 1: return copy(pts)
    var s = copy(pts)
    s.sort()
    var uniq: List<Pt> = []
    for p in s:
        if uniq.len == 0 or uniq.last! != p: uniq.push(p)
    if uniq.len <= 1: return uniq
    let lo = lower_hull(uniq, collinear:)
    let hi = upper_hull(uniq, collinear:)
    if lo.len == uniq.len and hi.len == uniq.len:
        // all unique points are collinear
        return lo
    var res = lo
    if hi.len >= 2:
        for i in (1..hi.len - 1).rev:
            res.push(hi[i])
    res

/// The convex hull of `pts` as domain positions `D` instead of copied points; otherwise as
/// `convex_hull`.
///
/// Of several positions with equal coordinates, the smallest in key order stands for them.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]`
/// order: counter-clockwise, starting at the least `(x, y)`; a new `List` of keys of `pts`
/// traps: never
/// example: `convex_hull_idx([Pt(1, 1), Pt(0, 0), Pt(2, 0), Pt(0, 0)]) == [1, 2, 0]`
pub fn convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>:
    var order = D.collect()
    if order.len <= 1: return order
    order.sort(by: i => pts[i])
    var uniq: List<D> = []
    for i in order:
        if uniq.len == 0 or pts[uniq.last!] != pts[i]: uniq.push(i)
    if uniq.len <= 1: return uniq
    var lo: List<D> = []
    for i in uniq:
        let p = pts[i]
        while lo.len >= 2:
            let t = turn(pts[lo[lo.len - 2]], pts[lo.last!], p)
            if t < 0 or (not collinear and t == 0): lo.pop()
            else: break
        lo.push(i)
    var hi: List<D> = []
    for i in uniq:
        let p = pts[i]
        while hi.len >= 2:
            let t = turn(pts[hi[hi.len - 2]], pts[hi.last!], p)
            if t > 0 or (not collinear and t == 0): hi.pop()
            else: break
        hi.push(i)
    if lo.len == uniq.len and hi.len == uniq.len:
        return lo
    var res = lo
    if hi.len >= 2:
        for k in (1..hi.len - 1).rev:
            res.push(hi[k])
    res

/// The lower hull chain of `(x, y)`-sorted `pts` as domain positions, left to right; otherwise
/// as `lower_hull`, and equal points keep the smallest position.
///
/// cost: \(O(n)\) time and memory
/// empty: `[]`
/// order: left to right; a new `List` of keys of `pts`
/// ties: equal points keep the smallest position
/// traps: when `pts` is not sorted by `(x, y)` in key order
/// example: `lower_hull_idx([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [0, 1, 3]`
pub fn lower_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>:
    var h: List<D> = []
    var prev: ?D = None
    for i in D:
        let p = pts[i]
        if prev != None:
            let q = pts[prev!]
            assert q <= p, "`lower_hull_idx` needs `pts` sorted by `(x, y)`"
            if q == p: continue
        prev = Some(i)
        while h.len >= 2:
            let t = turn(pts[h[h.len - 2]], pts[h.last!], p)
            if t < 0 or (not collinear and t == 0): h.pop()
            else: break
        h.push(i)
    h

/// The upper hull chain of `(x, y)`-sorted `pts` as domain positions, left to right; otherwise
/// as `upper_hull`, and equal points keep the smallest position.
///
/// cost: \(O(n)\) time and memory
/// empty: `[]`
/// order: left to right; a new `List` of keys of `pts`
/// ties: equal points keep the smallest position
/// traps: when `pts` is not sorted by `(x, y)` in key order
/// example: `upper_hull_idx([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [0, 2, 3]`
pub fn upper_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>:
    var h: List<D> = []
    var prev: ?D = None
    for i in D:
        let p = pts[i]
        if prev != None:
            let q = pts[prev!]
            assert q <= p, "`upper_hull_idx` needs `pts` sorted by `(x, y)`"
            if q == p: continue
        prev = Some(i)
        while h.len >= 2:
            let t = turn(pts[h[h.len - 2]], pts[h.last!], p)
            if t > 0 or (not collinear and t == 0): h.pop()
            else: break
        h.push(i)
    h

/// \(O(n)\) memory. Incremental 2D convex hull of integer points backed by upper and lower chains.
pub struct OnlineHull:
    hull: List<Pt>
    pub area: Int

    /// An empty online convex hull.
    ///
    /// The hull keeps strict corners only: points on an edge or inside are absorbed.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `OnlineHull().is_empty == true`
    pub fn new() -> OnlineHull = OnlineHull([], 0)

    /// Insert `p` and report whether the hull changed (whether `p` is now a corner).
    ///
    /// A point inside, on the boundary, or equal to a corner returns false and changes nothing.
    ///
    /// cost: \(O(\log n)\) amortized
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)) == true`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.add(Pt(1, 1)) == false`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(0, 0)) == false`
    pub fn add(mut self, p: Pt) -> Bool:
        with lint(unproven_list_index: allow)
        if self.locate(p) != Location.Outside: return false
        var pts = hull.collect()
        pts.push(p)
        hull = convex_hull(pts, collinear: false)
        var a2: I128 = 0
        let m = hull.len
        if m >= 3:
            for i in 0..m: a2 += hull[i].cross128(hull[(i + 1) % m])
        area = Int(a2)
        true

    /// Where `p` is relative to the current hull: `Inside`, `Boundary` or `Outside`.
    ///
    /// An empty hull has everything `Outside`; a one-point or segment hull has only its points on
    /// the `Boundary`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.locate(Pt(1, 1)) == Location.Inside`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.locate(Pt(2, 0)) == Location.Boundary`
    /// example: `OnlineHull().locate(Pt(0, 0)) == Location.Outside`
    pub fn locate(self, p: Pt) -> Location:
        with lint(unproven_list_index: allow)
        let m = hull.len
        if m == 0: return Location.Outside
        if m == 1: return if p == hull[0]: Location.Boundary else: Location.Outside
        if m == 2: return if on_segment(
            p,
            hull[0],
            hull[1],
        ): Location.Boundary else: Location.Outside
        let c1 = cross128_3(hull[0], hull[1], p)
        let c2 = cross128_3(hull[0], hull[m - 1], p)
        if c1 < 0 or c2 > 0: return Location.Outside
        if c1 == 0: return if on_segment(
            p,
            hull[0],
            hull[1],
        ): Location.Boundary else: Location.Outside
        if c2 == 0: return if on_segment(
            p,
            hull[0],
            hull[m - 1],
        ): Location.Boundary else: Location.Outside
        var lo = 1
        var hi = m - 1
        while hi - lo > 1:
            let mid = (lo + hi) / 2
            if cross128_3(hull[0], hull[mid], p) >= 0: lo = mid else: hi = mid
        let s = cross128_3(hull[lo], hull[hi], p)
        if s < 0: Location.Outside
        elif s == 0: Location.Boundary
        else: Location.Inside

    /// The hull corner maximizing `v.dot128(dir)`, or `None` when the hull is empty.
    ///
    /// Ties go to the corner that comes first in `vertices()` order.
    ///
    /// cost: \(O(\log n)\)
    /// ties: the first in `vertices()` order
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.extreme(Pt(1, 1)) == Some(Pt(4, 0))`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.extreme(Pt(0, 1)) == Some(Pt(0, 4))`
    /// example: `OnlineHull().extreme(Pt(1, 0)) == None`
    pub fn extreme(self, dir: Pt) -> ?Pt:
        with lint(unproven_list_index: allow)
        if hull.is_empty: return None
        var best = hull[0]
        var best_v = best.dot128(dir)
        for i in 1..hull.len:
            let v = hull[i].dot128(dir)
            if v > best_v:
                best_v = v
                best = hull[i]
        Some(best)

    /// Twice the area of the current hull; 0 with fewer than 3 non-collinear points.
    ///
    /// cost: \(O(1)\)
    /// traps: when the value overflows `Int`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.area2 == 16`
    /// example: `OnlineHull().area2 == 0`
    @get pub fn area2(self) -> Int = area

    /// The number of hull corners.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.add(Pt(1, 1)); h.len == 3`
    @get pub fn len(self) -> Int = hull.len

    /// Whether no point has been added since creation or the last `clear`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `OnlineHull().is_empty == true`
    /// example: `var h = OnlineHull(); h.add(Pt(1, 1)); h.is_empty == false`
    @get pub fn is_empty(self) -> Bool = hull.is_empty

    /// The current hull corners, counter-clockwise from the least `(x, y)`.
    ///
    /// cost: \(O(n)\) time and memory
    /// order: counter-clockwise, starting at the least `(x, y)`; a new `List`, keyed from 0
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 4)); h.add(Pt(4, 0)); h.add(Pt(0, 0)); h.vertices() == [Pt(0, 0), Pt(4, 0), Pt(0, 4)]`
    pub fn vertices(self) -> List<Pt> = hull.collect()

    /// The current hull corners, yielded in `vertices()` order without building a list.
    ///
    /// cost: \(O(n)\)
    /// order: counter-clockwise, starting at the least `(x, y)`
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 4)); h.add(Pt(4, 0)); h.add(Pt(0, 0)); h.iter().collect() == [Pt(0, 0), Pt(4, 0), Pt(0, 4)]`
    pub fn iter(self) -> yields Pt:
        for p in hull: yield p

    /// Remove every point; the hull is then empty.
    ///
    /// cost: \(O(n)\)
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(1, 1)); h.clear(); h.is_empty`
    pub fn clear(mut self):
        hull = []
        area = 0

fn min_yx_idx(poly: List<Pt>) -> Int:
    var best = 0
    for i in 1..poly.len:
        if poly[i].y < poly[best].y or (poly[i].y == poly[best].y and poly[i].x < poly[best].x):
            best = i
    best

/// The Minkowski sum `{p + q}` of two non-empty counter-clockwise convex polygons, by merging
/// edges.
///
/// The result is convex, counter-clockwise from its least `(y, x)` corner, with collinear
/// points dropped. Inputs may start at any corner.
///
/// cost: \(O(\lvert a \rvert + \lvert b \rvert)\) time and memory
/// empty: traps
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// ties: the start is the least `(y, x)`, which is unique
/// traps: when `a` or `b` is empty
/// example: `minkowski_sum([Pt(0, 0), Pt(1, 0), Pt(1, 1), Pt(0, 1)], [Pt(0, 0), Pt(1, 0), Pt(0, 1)]) == [Pt(0, 0), Pt(2, 0), Pt(2, 1), Pt(1, 2), Pt(0, 2)]`
/// example: `minkowski_sum([Pt(1, 1)], [Pt(0, 0), Pt(1, 0), Pt(0, 1)]) == [Pt(1, 1), Pt(2, 1), Pt(1, 2)]`
pub fn minkowski_sum(a: List<Pt>, b: List<Pt>) -> List<Pt>:
    assert a.len > 0 and b.len > 0, "minkowski_sum requires non-empty polygons"
    if a.len == 1: return [a[0] + q for q in b]
    if b.len == 1: return [p + b[0] for p in a]
    let na = a.len
    let nb = b.len
    var ia = min_yx_idx(a)
    var ib = min_yx_idx(b)
    var res: List<Pt> = []
    var i = 0
    var j = 0
    while i < na or j < nb:
        let cur = a[ia] + b[ib]
        if res.len == 0 or res.last! != cur: res.push(cur)
        if i == na:
            j += 1
            ib += 1
            if ib == nb: ib = 0
        elif j == nb:
            i += 1
            ia += 1
            if ia == na: ia = 0
        else:
            let ia_nxt = if ia + 1 == na: 0 else: ia + 1
            let ib_nxt = if ib + 1 == nb: 0 else: ib + 1
            let ea = a[ia_nxt] - a[ia]
            let eb = b[ib_nxt] - b[ib]
            let c = cross128(ea, eb)
            if c > I128(0):
                i += 1
                ia = ia_nxt
            elif c < I128(0):
                j += 1
                ib = ib_nxt
            else:
                i += 1
                j += 1
                ia = ia_nxt
                ib = ib_nxt
    while res.len > 1 and res.last! == res[0]:
        res.pop()
    res

/// The upper hull of the Minkowski sum of two non-empty left-to-right upper hulls.
///
/// The result is again a left-to-right upper hull with collinear points dropped.
///
/// cost: \(O(\lvert a \rvert + \lvert b \rvert)\) time and memory
/// empty: traps
/// order: left to right; a new `List`, keyed from 0
/// ties: collinear sums are dropped, keeping the two ends
/// traps: when `a` or `b` is empty
/// example: `minkowski_upper([Pt(0, 0), Pt(1, 1), Pt(2, 0)], [Pt(0, 0), Pt(1, 0)]) == [Pt(0, 0), Pt(1, 1), Pt(2, 1), Pt(3, 0)]`
pub fn minkowski_upper(a: List<Pt>, b: List<Pt>) -> List<Pt>:
    assert a.len > 0 and b.len > 0, "minkowski_upper requires non-empty hulls"
    var res = [a[0] + b[0]]
    var i = 0
    var j = 0
    while i + 1 < a.len or j + 1 < b.len:
        if i + 1 == a.len:
            j += 1
        else:
            if j + 1 == b.len:
                i += 1
            else:
                let ea = a[i + 1] - a[i]
                let eb = b[j + 1] - b[j]
                // Upper hull edges go left-to-right with decreasing slope; take steeper (larger cross).
                let c = cross128(ea, eb)
                if c < I128(0):
                    i += 1
                else:
                    if c > I128(0):
                        j += 1
                    else:
                        i += 1
                        j += 1
        let nxt = a[i] + b[j]
        while res.len >= 2 and turn(res[res.len - 2], res.last!, nxt) >= 0:
            res.pop()
        if res.last! != nxt: res.push(nxt)
    res

/// The lower hull of the Minkowski sum of two non-empty left-to-right lower hulls; the mirror of
/// `minkowski_upper`.
///
/// cost: \(O(\lvert a \rvert + \lvert b \rvert)\) time and memory
/// empty: traps
/// order: left to right; a new `List`, keyed from 0
/// ties: collinear sums are dropped, keeping the two ends
/// traps: when `a` or `b` is empty
/// example: `minkowski_lower([Pt(0, 0), Pt(1, -1), Pt(2, 0)], [Pt(0, 0), Pt(1, 0)]) == [Pt(0, 0), Pt(1, -1), Pt(2, -1), Pt(3, 0)]`
pub fn minkowski_lower(a: List<Pt>, b: List<Pt>) -> List<Pt>:
    assert a.len > 0 and b.len > 0, "minkowski_lower requires non-empty hulls"
    let up = minkowski_upper(
        [-a[a.len - 1 - i] for i in 0..a.len],
        [-b[b.len - 1 - i] for i in 0..b.len],
    )
    [-up[up.len - 1 - i] for i in 0..up.len]

/// Whether two non-empty counter-clockwise convex polygons share at least one point, boundary
/// included. Exact in `I128`.
///
/// cost: \(O(\lvert a \rvert + \lvert b \rvert)\)
/// empty: traps
/// traps: when `a` or `b` is empty
/// example: `convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]) == true`
/// example: `convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(2, 2), Pt(3, 2), Pt(3, 3)]) == true`
/// example: `convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(5, 5), Pt(6, 5), Pt(6, 6)]) == false`
pub fn convex_polygons_overlap(a: List<Pt>, b: List<Pt>) -> Bool:
    assert a.len > 0 and b.len > 0, "convex_polygons_overlap requires non-empty polygons"
    locate_convex(minkowski_sum(a, [-q for q in b]), Pt(0, 0)) != Location.Outside

/// The intersection of two counter-clockwise convex polygons, as a counter-clockwise polygon.
///
/// Vertices within `eps` of each other merge. An intersection with area at most `eps` (disjoint,
/// or touching at a point or along an edge) is `[]`. The result starts at its least `(y, x)`
/// corner.
///
/// cost: \(O(\lvert a \rvert + \lvert b \rvert)\) time and memory
/// empty: `[]` when either input is empty
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// traps: never
/// example: `convex_polygon_intersect([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], [Vec(1.0, 1.0), Vec(3.0, 1.0), Vec(3.0, 3.0), Vec(1.0, 3.0)]) == [Vec(1.0, 1.0), Vec(2.0, 1.0), Vec(2.0, 2.0), Vec(1.0, 2.0)]`
/// example: `convex_polygon_intersect([Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0)], [Vec(5.0, 5.0), Vec(6.0, 5.0), Vec(5.0, 6.0)]) == []`
pub fn convex_polygon_intersect(a: List<Vec>, b: List<Vec>, eps: F64 = 1e-9) -> List<Vec>:
    if a.is_empty or b.is_empty: return []
    half_plane_intersect(edge_planes(a) + edge_planes(b), eps:)

// ---- polygons and rotating calipers

/// Twice the signed area of the polygon `poly` (shoelace): positive when the vertices run
/// counter-clockwise, negative when clockwise; 0 for fewer than 3 vertices.
///
/// cost: \(O(n)\)
/// empty: 0
/// traps: when the sum overflows `Int`
/// example: `area2([Pt(0, 0), Pt(2, 0), Pt(0, 2)]) == 4`
/// example: `area2([Pt(0, 0), Pt(0, 2), Pt(2, 0)]) == -4`
/// example: `area2([Pt(0, 0), Pt(1, 1)]) == 0`
pub fn area2(poly: List<Pt>) -> Int:
    let n = poly.len
    if n < 3: return 0
    var total = 0
    var a = poly[n - 1]
    for b in poly:
        total += a.x * b.y - a.y * b.x
        a = b
    total

/// Twice the signed area of `poly`, as `area2`, accumulated exactly in `I128`.
///
/// cost: \(O(n)\)
/// empty: 0
/// traps: when the sum overflows `I128`
/// example: `area2_128([Pt(0, 0), Pt(2, 0), Pt(0, 2)]) == 4`
pub fn area2_128(poly: List<Pt>) -> I128:
    let n = poly.len
    if n < 3: return I128(0)
    var total = I128(0)
    var a = poly[n - 1]
    for b in poly:
        total += cross128(a, b)
        a = b
    total

/// \(O(1)\) enum. Position of a point relative to a polygon or disk.
pub enum Location = Inside | Boundary | Outside

/// Where `p` is relative to the simple polygon `poly`, in either orientation: `Inside`,
/// `Boundary` or `Outside`. Exact in `I128`.
///
/// Boundary is tested first (on any edge, including vertices); otherwise ray crossing decides.
/// An empty polygon has everything `Outside`.
///
/// cost: \(O(n)\)
/// empty: `Location.Outside`
/// traps: never
/// example: `locate([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(1, 1)) == Location.Inside`
/// example: `locate([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(4, 2)) == Location.Boundary`
/// example: `locate([Pt(0, 0), Pt(0, 4), Pt(4, 4), Pt(4, 0)], Pt(5, 1)) == Location.Outside`
pub fn locate(poly: List<Pt>, p: Pt) -> Location:
    let n = poly.len
    if n == 0: return Location.Outside
    if n == 1: return if p == poly[0]: Location.Boundary else: Location.Outside
    var inside = false
    var prev = poly[n - 1]
    for cur in poly:
        var a = prev
        var b = cur
        prev = cur
        if a.y > b.y:
            let tmp = a
            a = b
            b = tmp
        if a.y <= p.y and p.y <= b.y:
            let c = I128(a.x - p.x) * I128(b.y - p.y) - I128(a.y - p.y) * I128(b.x - p.x)
            if c == I128(0):
                if I128(a.x - p.x) * I128(b.x - p.x) + I128(a.y - p.y) * I128(b.y - p.y) <= I128(0):
                    return Location.Boundary
            elif p.y < b.y and c > I128(0):
                inside = not inside
    if inside: Location.Inside else: Location.Outside

/// Where `p` is relative to the counter-clockwise convex polygon `hull`: `Inside`, `Boundary`
/// or `Outside`, by binary search on the fan from `hull[0]`. Exact in `I128`.
///
/// A hull of one or two points has only its points (or segment) on the `Boundary`; an empty hull
/// has everything `Outside`.
///
/// cost: \(O(\log n)\)
/// empty: `Location.Outside`
/// traps: never
/// example: `locate_convex([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(2, 2)) == Location.Inside`
/// example: `locate_convex([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(0, 0)) == Location.Boundary`
/// example: `locate_convex([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(-1, 2)) == Location.Outside`
pub fn locate_convex(hull: List<Pt>, p: Pt) -> Location:
    let n = hull.len
    if n == 0: return Location.Outside
    if n == 1: return if p == hull[0]: Location.Boundary else: Location.Outside
    if n == 2: return if on_segment(p, hull[0], hull[1]): Location.Boundary else: Location.Outside
    let t1 = turn(hull[0], hull[1], p)
    let t2 = turn(hull[0], hull[n - 1], p)
    if t1 < 0 or t2 > 0: return Location.Outside
    if t1 == 0: return if on_segment(p, hull[0], hull[1]): Location.Boundary else: Location.Outside
    if t2 == 0: return if on_segment(
        p,
        hull[0],
        hull[n - 1],
    ): Location.Boundary else: Location.Outside
    var lo = 1
    var hi = n - 1
    while hi - lo > 1:
        let mid = (lo + hi) / 2
        if turn(hull[0], hull[mid], p) >= 0: lo = mid
        else: hi = mid
    let t = turn(hull[lo], hull[lo + 1], p)
    if t > 0: Location.Inside elif t == 0: Location.Boundary else: Location.Outside

/// The squared diameter of the counter-clockwise convex polygon `hull`: the largest
/// `dist2` between two vertices, by rotating calipers.
///
/// cost: \(O(n)\)
/// empty: 0 (also for one point)
/// traps: when the result overflows `Int`
/// example: `diameter2([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 8`
/// example: `diameter2([Pt(0, 0), Pt(3, 4)]) == 25`
pub fn diameter2(hull: List<Pt>) -> Int:
    let n = hull.len
    if n <= 1: return 0
    if n == 2: return dist2(hull[0], hull[1])
    var best = 0
    var j = 1
    for i in 0..n:
        let ni = (i + 1) % n
        let edge = hull[ni] - hull[i]
        while true:
            let nj = (j + 1) % n
            if cross128(edge, hull[nj] - hull[j]) > I128(0): j = nj
            else: break
        best max= dist2(hull[i], hull[j])
        best max= dist2(hull[ni], hull[j])
    best

/// The least area and the least perimeter of a rectangle enclosing the counter-clockwise convex
/// polygon `hull`, each minimized on its own (by rotating calipers, exact in `I128` until the
/// final division).
///
/// One side of an optimal rectangle lies along a hull edge, so only those `n` orientations are
/// tried. Fewer than 3 vertices give a degenerate rectangle (area 0, perimeter twice the length).
///
/// cost: \(O(n)\)
/// empty: `(0.0, 0.0)`
/// ties: None; both results are values, not a chosen rectangle
/// traps: never
/// example: `min_bounding_box([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == (4.0, 8.0)`
/// example: `min_bounding_box([Pt(0, 0), Pt(3, 0)]) == (0.0, 6.0)`
pub fn min_bounding_box(hull: List<Pt>) -> (area: F64, perimeter: F64):
    let n = hull.len
    if n <= 1: return (area: 0.0, perimeter: 0.0)
    if n == 2: return (area: 0.0, perimeter: 2.0 * dist(hull[0], hull[1]))
    var best_area = 1e300
    var best_perim = 1e300
    var right = 1
    var top = 1
    var left = 1
    for i in 0..n:
        let ni = if i + 1 == n: 0 else: i + 1
        let e = hull[ni] - hull[i]
        if e.x == 0 and e.y == 0: continue
        while true:
            let nr = if right + 1 == n: 0 else: right + 1
            if dot128(e, hull[nr] - hull[right]) > I128(0): right = nr
            else: break
        while true:
            let nt = if top + 1 == n: 0 else: top + 1
            if cross128(e, hull[nt] - hull[top]) > I128(0): top = nt
            else: break
        if i == 0: left = top
        while true:
            let nl = if left + 1 == n: 0 else: left + 1
            if dot128(e, hull[nl] - hull[left]) < I128(0): left = nl
            else: break
        let elen = sqrt(F64(dot128(e, e)))
        let w = F64(dot128(e, hull[right] - hull[left])) / elen
        let h = F64(cross128(e, hull[top] - hull[i])) / elen
        best_area min= w * h
        best_perim min= 2.0 * (w + h)
    (area: best_area, perimeter: best_perim)

fn better_dir(a: Pt, b: Pt, dir: Pt) -> Bool:
    let da = dot128(a, dir)
    let db = dot128(b, dir)
    if da != db: return da > db
    dot128(a, dir.perp) > dot128(b, dir.perp)

/// The index of a vertex of the non-empty counter-clockwise convex polygon `hull` maximizing
/// `hull[i].dot128(dir)`, by binary search.
///
/// cost: \(O(\log n)\)
/// empty: traps
/// ties: the smallest index
/// traps: when `hull` is empty
/// example: `hull_extreme([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(1, 1)) == 2`
/// example: `hull_extreme([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(1, 0)) == 1`
pub fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int:
    let i = extreme_by_dir(hull, dir)
    let n = hull.len
    let d = dot128(hull[i], dir)
    var best = i
    var k = if i == 0: n - 1 else: i - 1
    while k != i and dot128(hull[k], dir) == d:
        best min= k
        k = if k == 0: n - 1 else: k - 1
    k = if i + 1 == n: 0 else: i + 1
    while k != i and dot128(hull[k], dir) == d:
        best min= k
        k = if k + 1 == n: 0 else: k + 1
    best

fn extreme_by_dir(hull: List<Pt>, dir: Pt) -> Int:
    let n = hull.len
    assert n > 0, "hull_extreme requires a non-empty hull"
    if n == 1 or (dir.x == 0 and dir.y == 0): return 0
    if n <= 4:
        var best = 0
        for i in 1..n:
            if better_dir(hull[i], hull[best], dir): best = i
        return best
    if better_dir(hull[0], hull[1], dir) and better_dir(hull[0], hull[n - 1], dir):
        return 0
    var lo = 0
    var hi = n
    while hi - lo > 2:
        let mid = (lo + hi) / 2
        let lo_nxt = if lo + 1 == n: 0 else: lo + 1
        let mid_nxt = if mid + 1 == n: 0 else: mid + 1
        let mid_prv = if mid == 0: n - 1 else: mid - 1
        let up_lo = better_dir(hull[lo_nxt], hull[lo], dir)
        let up_mid = better_dir(hull[mid_nxt], hull[mid], dir)
        if not up_mid and better_dir(hull[mid], hull[mid_prv], dir):
            return mid
        if up_lo:
            if up_mid and better_dir(hull[mid], hull[lo], dir): lo = mid
            else: hi = mid
        else:
            if not up_mid and better_dir(hull[mid], hull[lo], dir): hi = mid
            else: lo = mid
    var best = lo
    for k in lo..hi:
        let idx = if k == n: 0 else: k
        if better_dir(hull[idx], hull[best], dir): best = idx
    best

fn tangent_cmp(p: Pt, a: Pt, b: Pt, sign: Int) -> Bool:
    let t = turn(p, b, a) * sign
    if t != 0: return t > 0
    dist2_128(p, a) < dist2_128(p, b)

fn find_tangent(hull: List<Pt>, p: Pt, sign: Int) -> Int:
    let n = hull.len
    if n <= 4:
        var best = 0
        for i in 1..n:
            if tangent_cmp(p, hull[i], hull[best], sign): best = i
        return best
    if tangent_cmp(p, hull[0], hull[1], sign) and tangent_cmp(p, hull[0], hull[n - 1], sign):
        return 0
    var lo = 0
    var hi = n
    while hi - lo > 2:
        let mid = (lo + hi) / 2
        let up_lo = tangent_cmp(p, hull[(lo + 1) % n], hull[lo], sign)
        let up_mid = tangent_cmp(p, hull[(mid + 1) % n], hull[mid], sign)
        if not up_mid and tangent_cmp(p, hull[mid], hull[(mid + n - 1) % n], sign):
            return mid
        if up_lo:
            if up_mid and tangent_cmp(p, hull[mid], hull[lo], sign): lo = mid
            else: hi = mid
        else:
            if not up_mid and tangent_cmp(p, hull[mid], hull[lo], sign): hi = mid
            else: lo = mid
    var best = lo
    for k in lo..hi:
        let idx = k % n
        if tangent_cmp(p, hull[idx], hull[best], sign): best = idx
    best

/// The indices `(left, right)` of the tangent vertices from a strictly exterior point `p` to a
/// strictly convex counter-clockwise polygon `hull` (`hull.len >= 3`).
///
/// Every vertex `q` satisfies `turn(p, hull[left], q) <= 0` and `turn(p, hull[right], q) >= 0`:
/// seen from `p`, the whole polygon lies right of the ray to `left` and left of the ray to
/// `right`.
///
/// cost: \(O(\log n)\)
/// ties: when two vertices qualify (`p` on an edge's line), the one nearer to `p`
/// empty: traps (needs `hull.len >= 3`)
/// traps: when `hull.len < 3` or `p` is not strictly outside
/// example: `hull_tangents([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(4, 1)) == (1, 2)`
pub fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int):
    assert hull.len >= 3, "hull_tangents requires at least 3 hull vertices"
    assert locate_convex(hull, p) == Location.Outside, "hull_tangents needs `p` strictly outside"
    (left: find_tangent(hull, p, 1), right: find_tangent(hull, p, -1))

// ---- closest pair

fn cp_solve(
    mut ord: List<(x: Int, y: Int, id: Int)>,
    mut buf: List<(x: Int, y: Int, id: Int)>,
    l: Int,
    r: Int,
) -> (a: Int, b: Int, dist2: Int):
    if r - l <= 3:
        var best = Int.MAX
        var ba = ord[l].id
        var bb = ord[l + 1].id
        for i in l..r:
            let xi = ord[i].x
            let yi = ord[i].y
            for j in i + 1..r:
                let dx = xi - ord[j].x
                let dy = yi - ord[j].y
                let d = dx * dx + dy * dy
                if d < best:
                    best = d
                    ba = ord[i].id
                    bb = ord[j].id
        // sort ord[l..r] by y
        for i in l + 1..r:
            var k = i
            while k > l and ord[k].y < ord[k - 1].y:
                let tmp = ord[k]
                ord[k] = ord[k - 1]
                ord[k - 1] = tmp
                k -= 1
        return (ba, bb, best)
    let mid = (l + r) / 2
    let mid_x = ord[mid].x
    let left = cp_solve(mut ord, mut buf, l, mid)
    let right = cp_solve(mut ord, mut buf, mid, r)
    var best = left.dist2
    var ba = left.a
    var bb = left.b
    if right.dist2 < best:
        best = right.dist2
        ba = right.a
        bb = right.b
    // merge by y into buf[l..r]
    var i = l
    var j = mid
    var k = l
    while i < mid and j < r:
        if ord[i].y <= ord[j].y:
            buf[k] = ord[i]
            i += 1
        else:
            buf[k] = ord[j]
            j += 1
        k += 1
    while i < mid:
        buf[k] = ord[i]
        i += 1
        k += 1
    while j < r:
        buf[k] = ord[j]
        j += 1
        k += 1
    for t in l..r: ord[t] = buf[t]
    // reuse buf[l..r] as the strip scratch buffer (zero allocation per recursive call)
    var s_end = l
    let best128 = I128(best)
    for t in l..r:
        let dx = I128(ord[t].x) - I128(mid_x)
        if dx * dx < best128:
            buf[s_end] = ord[t]
            s_end += 1
    for si in l..s_end:
        let xsi = buf[si].x
        let ysi = buf[si].y
        var sj = si + 1
        while sj < s_end:
            let dy = I128(buf[sj].y) - I128(ysi)
            if dy * dy >= I128(best): break
            let dx = xsi - buf[sj].x
            let dy_i = ysi - buf[sj].y
            let d = dx * dx + dy_i * dy_i
            if d < best:
                best = d
                ba = buf[si].id
                bb = buf[sj].id
            sj += 1
    (ba, bb, best)

/// The least squared distance between two points of `pts` at distinct positions (0 when two are
/// equal), by divide and conquer.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: traps
/// traps: when `pts.len < 2`, or a distance overflows `Int`
/// example: `closest_pair2([Pt(0, 0), Pt(5, 5), Pt(1, 1), Pt(9, 9)]) == 2`
/// example: `closest_pair2([Pt(3, 3), Pt(3, 3)]) == 0`
pub fn closest_pair2(pts: List<Pt>) -> Int:
    assert pts.len >= 2, "closest_pair2 requires at least 2 points"
    var ord: List<(x: Int, y: Int, id: Int)> = [
        (x: pts[i].x, y: pts[i].y, id: i) for i in 0..pts.len
    ]
    ord.sort()
    for i in 1..ord.len:
        if ord[i].x == ord[i - 1].x and ord[i].y == ord[i - 1].y: return 0
    var buf = copy(ord)
    cp_solve(mut ord, mut buf, 0, ord.len).dist2

/// The closest pair of distinct positions `a` before `b` in key order, with their squared
/// distance.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: traps
/// ties: the pair `(a, b)` that comes first in key order, `a` then `b`
/// traps: when `pts` has fewer than 2 points, or a distance overflows `Int`
/// example: `closest_pair([Pt(0, 0), Pt(5, 5), Pt(1, 1)]) == (0, 2, 2)`
/// example: `closest_pair([Pt(0, 0), Pt(1, 0), Pt(2, 0)]) == (0, 1, 1)`
pub fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int):
    var ids: List<D> = []
    var ord: List<(x: Int, y: Int, id: Int)> = []
    var k = 0
    for i in D:
        ids.push(i)
        let p = pts[i]
        ord.push((x: p.x, y: p.y, id: k))
        k += 1
    assert ids.len >= 2, "closest_pair requires at least 2 points"
    ord.sort()
    for i in 1..ord.len:
        if ord[i].x == ord[i - 1].x and ord[i].y == ord[i - 1].y:
            return (ids[ord[i - 1].id], ids[ord[i].id], 0)
    var buf = copy(ord)
    let res = cp_solve(mut ord, mut buf, 0, ord.len)
    (ids[res.a], ids[res.b], res.dist2)

/// The least width of the counter-clockwise convex polygon `hull`: the smallest distance between
/// two parallel supporting lines, by rotating calipers. Fewer than 3 vertices give 0.0.
///
/// cost: \(O(n)\)
/// empty: 0.0
/// traps: never
/// example: `hull_width([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 2.0`
/// example: `hull_width([Pt(0, 0), Pt(4, 0), Pt(4, 1), Pt(0, 1)]) == 1.0`
pub fn hull_width(hull: List<Pt>) -> F64:
    let n = hull.len
    if n < 3: return 0.0
    var best = -1.0
    var j = 1
    for i in 0..n:
        let a = hull[i]
        let e = hull[(i + 1) % n] - a
        while cross128(e, hull[(j + 1) % n] - a) > cross128(e, hull[j] - a): j = (j + 1) % n
        let w = F64(cross128(e, hull[j] - a)) / e.len
        if best < 0.0 or w < best: best = w
    best

/// The two edges of the counter-clockwise convex polygon `hull` that the line through `a` and
/// `b` crosses, as `(i, j)` with `i < j`, or `None` if the line misses the interior.
///
/// Edge `i` runs from `hull[i]` to `hull[(i + 1) % n]`. A line that only touches the polygon (at a
/// vertex, or along an edge) returns `None`.
///
/// cost: \(O(\log n)\)
/// empty: `None`
/// traps: when `a == b`
/// example: `hull_line_intersect([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(-1, 1), Pt(3, 1)) == Some((1, 3))`
/// example: `hull_line_intersect([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(-1, 5), Pt(3, 5)) == None`
pub fn hull_line_intersect(hull: List<Pt>, a: Pt, b: Pt) -> ?(Int, Int):
    assert a != b, "hull_line_intersect needs two distinct points"
    let n = hull.len
    if n < 3: return None
    let nd = (b - a).perp
    let hi = hull_extreme(hull, nd)
    let lo = hull_extreme(hull, -nd)
    let side = k => (hull[k % n] - a).dot128(nd)
    if side(hi) <= I128(0) or side(lo) >= I128(0): return None
    let down = last_where(hi, (lo - hi) % n, k => side(k) > I128(0)) % n
    let up = last_where(lo, (hi - lo) % n, k => side(k) < I128(0)) % n
    Some(if down < up: (down, up) else: (up, down))

fn last_where(from: Int, span: Int, ok: Int -> Bool) -> Int:
    let lo = (1..=span).last_true(mid => ok(from + mid)) ?? 0
    from + lo

/// The farthest pair of distinct positions `a` before `b` in key order, with their squared
/// distance (convex hull, then rotating calipers).
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: traps
/// ties: the pair `(a, b)` that comes first in key order, `a` then `b`
/// traps: when `pts` has fewer than 2 points, or a distance overflows `Int`
/// example: `farthest_pair([Pt(0, 0), Pt(1, 1), Pt(3, 0)]) == (0, 2, 9)`
pub fn farthest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int):
    let keys = D.collect()
    assert keys.len >= 2, "farthest_pair requires at least 2 points"
    let h = convex_hull_idx(pts)
    if h.len == 1: return (keys[0], keys[1], 0)
    var best = (d: -1, a: h[0], b: h[0])
    let consider = (i: D, j: D) =>
        let (x, y) = if i.ord < j.ord: (i, j) else: (j, i)
        let d = pts[x].dist2(pts[y])
        if d > best.d or (
            d == best.d and (x.ord < best.a.ord or (x == best.a and y.ord < best.b.ord))
        ):
            best = (d:, a: x, b: y)
    let n = h.len
    if n == 2:
        consider(h[0], h[1])
        return (best.a, best.b, best.d)
    let area = (i: Int, k: Int) => cross128(
        pts[h[(i + 1) % n]] - pts[h[i]],
        pts[h[k % n]] - pts[h[i]],
    )
    var j = 1
    for i in 0..n:
        while area(i, j + 1) > area(i, j): j = (j + 1) % n
        consider(h[i], h[j])
        consider(h[(i + 1) % n], h[j])
        if area(i, j + 1) == area(i, j):
            consider(h[i], h[(j + 1) % n])
            consider(h[(i + 1) % n], h[(j + 1) % n])
    (best.a, best.b, best.d)

/// At most `4n` candidate edges `(u, v, w)` that contain a minimum spanning tree of `pts` under
/// the Manhattan distance `w = pts[u].manhattan(pts[v])`.
///
/// For each of the 8 octants of each point, the nearest point in that octant (sweep with a
/// Fenwick tree), each unordered pair once with `u` before `v` in key order. Run Kruskal on the
/// result for the MST itself.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]` (also for one point)
/// order: by `(w, u, v)` ascending; a new `List`, keyed from 0
/// traps: when a distance overflows `Int`
/// example: `manhattan_mst([Pt(0, 0), Pt(1, 1), Pt(5, 5)]).any(e => e.u == 0 and e.v == 1 and e.w == 2)`
/// example: `manhattan_mst([Pt(0, 0), Pt(2, 0)]) == [(0, 1, 2)]`
pub fn manhattan_mst<D>(pts: [D]Pt) -> List<(u: D, v: D, w: Int)>:
    let keys = D.collect()
    let n = keys.len
    var xs = [pts[k].x for k in keys]
    var ys = [pts[k].y for k in keys]
    var found: List<(w: Int, u: Int, v: Int)> = []
    for dir in 0..4:
        if dir == 1 or dir == 3:
            let t = xs
            xs = ys
            ys = t
        if dir == 2: xs = [-x for x in xs]
        var order = [i for i in 0..n]
        order.sort(by: i => (xs[i], ys[i]))
        var vals = [ys[i] - xs[i] for i in 0..n]
        vals.sort()
        var best = List.filled(n + 1, (w: Int.MAX, id: -1))
        for t in (0..n).rev:
            let i = order[t]
            let pos = n - lower_idx(vals, ys[i] - xs[i])
            var q = pos
            var got = (w: Int.MAX, id: -1)
            while q > 0:
                if best[q].w < got.w: got = best[q]
                q -= q & -q
            if got.id >= 0:
                let w = pts[keys[i]].manhattan(pts[keys[got.id]])
                found.push(if i < got.id: (w:, u: i, v: got.id) else: (w:, u: got.id, v: i))
            q = pos
            while q <= n:
                if xs[i] + ys[i] < best[q].w: best[q] = (w: xs[i] + ys[i], id: i)
                q += q & -q
    found.sort()
    var out: List<(u: D, v: D, w: Int)> = []
    for k in 0..found.len:
        if k > 0 and found[k] == found[k - 1]: continue
        out.push((u: keys[found[k].u], v: keys[found[k].v], w: found[k].w))
    out

/// The centroid (centre of mass) of the region inside the simple polygon `poly`, either
/// orientation.
///
/// cost: \(O(n)\)
/// empty: traps
/// traps: when the polygon has zero area
/// example: `polygon_centroid([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]) == Vec(1.0, 1.0)`
pub fn polygon_centroid(poly: List<Vec>) -> Vec:
    let n = poly.len
    assert n > 0, "polygon_centroid requires a polygon"
    var a2 = 0.0
    var cx = 0.0
    var cy = 0.0
    for i in 0..n:
        let p = poly[i]
        let q = poly[(i + 1) % n]
        let c = p.cross(q)
        a2 += c
        cx += (p.x + q.x) * c
        cy += (p.y + q.y) * c
    assert a2 != 0.0, "polygon_centroid requires a polygon with non-zero area"
    Vec(cx / (3.0 * a2), cy / (3.0 * a2))

/// The kernel of the counter-clockwise simple polygon `poly`: the points that see the whole
/// interior, as a counter-clockwise convex polygon (half-plane intersection of the edges).
///
/// A kernel with area at most `eps` is `[]`. The result starts at its least `(y, x)` corner.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]`
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// traps: never
/// example: `polygon_kernel([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]) == [Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]`
pub fn polygon_kernel(poly: List<Vec>, eps: F64 = 1e-9) -> List<Vec>:
    half_plane_intersect(edge_planes(poly), eps:)

fn edge_planes(poly: List<Vec>) -> List<HalfPlane>:
    var hs: List<HalfPlane> = []
    for i in 0..poly.len:
        let q = poly[(i + 1) % poly.len]
        if poly[i] != q: hs.push(HalfPlane.through(poly[i], q))
    hs

/// A triangulation of the counter-clockwise simple polygon `poly` into `n - 2` triangles of
/// vertex indices, each listed counter-clockwise.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]` (also for fewer than 3 vertices)
/// order: deterministic for a given input; each triple counter-clockwise
/// traps: when `poly` is not counter-clockwise
/// example: `polygon_triangulate([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]).len == 2`
/// example: `polygon_triangulate([Pt(0, 0), Pt(1, 0), Pt(0, 1)]).len == 1`
pub fn polygon_triangulate(poly: List<Pt>) -> List<(Int, Int, Int)>:
    with lint(unproven_list_index: allow)
    let n = poly.len
    if n < 3: return []
    let area2: I128 = sum(poly[i].cross128(poly[(i + 1) % n]) for i in 0..n)
    assert area2 > 0, "polygon_triangulate: polygon must be counter-clockwise"
    var rem: List<Int> = [i for i in 0..n]
    var tris: List<(Int, Int, Int)> = []
    while rem.len > 3:
        let m = rem.len
        var clipped = -1
        for i in 0..m:
            let ai = rem[(i + m - 1) % m]
            let bi = rem[i]
            let ci = rem[(i + 1) % m]
            let (a, b, c) = (poly[ai], poly[bi], poly[ci])
            if cross128_3(a, b, c) <= 0: continue
            var ok = true
            for j in 0..m:
                let qi = rem[j]
                if qi == ai or qi == bi or qi == ci: continue
                let q = poly[qi]
                if cross128_3(a, b, q) >= 0 and cross128_3(b, c, q) >= 0 and cross128_3(
                    c,
                    a,
                    q,
                ) >= 0:
                    ok = false
                    break
            if ok:
                tris.push((ai, bi, ci))
                clipped = i
                break
        assert clipped != -1, "polygon_triangulate: polygon is not simple"
        let next_rem: List<Int> = [rem[i] for i in 0..m if i != clipped]
        rem = next_rem
    tris.push((rem[0], rem[1], rem[2]))
    tris

/// The number of lattice points strictly inside the simple lattice polygon `poly`, either
/// orientation, by Pick's theorem: `I = (|area2| - B) / 2 + 1` with `B = pick_boundary(poly)`.
///
/// cost: \(O(n \log \mathit{max\_coord})\)
/// empty: traps
/// traps: when `poly` has fewer than 3 vertices, or the area overflows `Int`
/// example: `pick_interior([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 1`
/// example: `pick_interior([Pt(0, 0), Pt(4, 0), Pt(0, 4)]) == 3`
pub fn pick_interior(poly: List<Pt>) -> Int:
    assert poly.len >= 3, "`pick_interior` needs at least 3 vertices"
    (abs(area2(poly)) - pick_boundary(poly)) / 2 + 1

/// The number of lattice points on the boundary of the lattice polygon `poly`: the sum of
/// `gcd(|dx|, |dy|)` over its edges, including the closing edge.
///
/// cost: \(O(n \log \mathit{max\_coord})\)
/// empty: 0
/// traps: when the sum overflows `Int`
/// example: `pick_boundary([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 8`
/// example: `pick_boundary([Pt(0, 0), Pt(4, 0), Pt(0, 4)]) == 12`
pub fn pick_boundary(poly: List<Pt>) -> Int:
    var b = 0
    for i in 0..poly.len:
        let d = poly[(i + 1) % poly.len] - poly[i]
        b += igcd(abs(d.x), abs(d.y))
    b

// ---- careful floating-point geometry

/// \(O(1)\). Circle constant pi (canonical home: `std.math.PI`; re-exported here).
pub const PI: F64 = 3.14159265358979323846

/// Whether `|a - b| <= eps`: an absolute tolerance, the same test every `eps` in this module uses.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `close(0.1 + 0.2, 0.3) == true`
/// example: `close(1.0, 1.1) == false`
/// example: `close(1.0, 1.25, eps: 0.25) == true`
pub fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool = abs(a - b) <= eps

fn pi() -> F64 = atan2(0.0, -1.0)

/// The sign of `x` with tolerance: 1 if `x > eps`, -1 if `x < -eps`, else 0.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `sign_eps(2.0) == 1`
/// example: `sign_eps(-0.5) == -1`
/// example: `sign_eps(1e-12) == 0`
pub fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int = if x > eps: 1 else: if x < -eps: -1 else: 0

/// \(O(1)\) value type. A 2D floating-point point or vector. Kept distinct from `Pt` so exact
/// integer predicates never silently mix with approximate arithmetic.
pub struct Vec(pub x: F64, pub y: F64):
    /// The squared length `x*x + y*y`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(3.0, 4.0).norm2 == 25.0`
    @get pub fn norm2(self) -> F64 = x * x + y * y

    /// The Euclidean length (`hypot(x, y)`, so no intermediate overflow).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(3.0, 4.0).len == 5.0`
    @get pub fn len(self) -> F64 = sqrt(x * x + y * y)

    /// `self` rotated 90 degrees counter-clockwise: `Vec(-y, x)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(1.0, 2.0).perp == Vec(-2.0, 1.0)`
    @get pub fn perp(self) -> Vec = Vec(-y, x)

    /// `self` rotated 90 degrees clockwise: `Vec(y, -x)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(1.0, 2.0).perp_cw == Vec(2.0, -1.0)`
    @get pub fn perp_cw(self) -> Vec = Vec(y, -x)

    /// The polar angle `atan2(y, x)` in `(-pi, pi]`; a `-0.0` `y` counts as `0.0`, so the negative x
    /// axis gives `PI`, and the zero vector gives 0.0.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(1.0, 0.0).angle == 0.0`
    /// example: `Vec(-1.0, 0.0).angle == PI`
    /// example: `Vec(0.0, 1.0).angle == PI / 2.0`
    @get pub fn angle(self) -> F64 = atan2(y, x)

    /// The unit vector `Vec(cos(rad), sin(rad))`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec.from_angle(0.0) == Vec(1.0, 0.0)`
    /// example: `close(Vec.from_angle(PI / 2.0).y, 1.0)`
    pub fn from_angle(rad: F64) -> Vec = Vec(cos(rad), sin(rad))

    /// The unit vector in the direction of `self`: each coordinate divided by `self.len`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `self.len == 0.0`
    /// example: `Vec(3.0, 4.0).unit == Vec(0.6, 0.8)`
    @get pub fn unit(self) -> Vec:
        let l = self.len
        assert l != 0.0, "`unit` needs a nonzero vector"
        Vec(x / l, y / l)

    /// `self` rotated by the direction `by`, as the complex product `self * by`:
    /// `Vec(x * by.x - y * by.y, x * by.y + y * by.x)`. A unit `by` rotates without scaling; get
    /// one from `Vec.from_angle`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(1.0, 2.0).rot(Vec(0.0, 1.0)) == Vec(-2.0, 1.0)`
    pub fn rot(self, by: Vec) -> Vec = Vec(x * by.x - y * by.y, x * by.y + y * by.x)

    /// The dot product `x * o.x + y * o.y`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(1.0, 2.0).dot(Vec(3.0, 4.0)) == 11.0`
    pub fn dot(self, o: Vec) -> F64 = x * o.x + y * o.y

    /// The 2D cross product `x * o.y - y * o.x`: positive when `o` is counter-clockwise from `self`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(1.0, 2.0).cross(Vec(3.0, 4.0)) == -2.0`
    pub fn cross(self, o: Vec) -> F64 = x * o.y - y * o.x

    /// The squared Euclidean distance from `self` to `o`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(0.0, 0.0).dist2(Vec(3.0, 4.0)) == 25.0`
    pub fn dist2(self, o: Vec) -> F64 = (o - self).norm2

    /// The Euclidean distance from `self` to `o`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec(0.0, 0.0).dist(Vec(3.0, 4.0)) == 5.0`
    pub fn dist(self, o: Vec) -> F64 = (o - self).len

impl Add for Vec:
    fn add(a, b) = Vec(a.x + b.x, a.y + b.y)

impl Sub for Vec:
    fn sub(a, b) = Vec(a.x - b.x, a.y - b.y)

impl Neg for Vec:
    fn neg(a) = Vec(-a.x, -a.y)

impl Mul<F64> for Vec:
    fn mul(a, k) = Vec(a.x * k, a.y * k)

impl Div<F64> for Vec:
    fn div(a, k) = Vec(a.x / k, a.y / k)

/// Which way the path `a -> b -> c` turns, with tolerance: `sign_eps((b - a).cross(c - a), eps)`,
/// so 1 left, -1 right, 0 straight within `eps`.
///
/// `eps` bounds the raw cross product (twice the triangle's area), not an angle.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `turn_eps(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0)) == 1`
/// example: `turn_eps(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, -1.0)) == -1`
/// example: `turn_eps(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(2.0, 1e-12)) == 0`
pub fn turn_eps(
    a: Vec,
    b: Vec,
    c: Vec,
    eps: F64 = 1e-9,
) -> Int = sign_eps((b - a).cross(c - a), eps)

/// The signed angle from `a` to `b`, `atan2(a.cross(b), a.dot(b))`, in `(-pi, pi]`: positive when
/// `b` is counter-clockwise from `a`. 0.0 when either is the zero vector.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `angle_between(Vec(1.0, 0.0), Vec(0.0, 1.0)) == PI / 2.0`
/// example: `angle_between(Vec(1.0, 0.0), Vec(0.0, -1.0)) == -PI / 2.0`
/// example: `angle_between(Vec(1.0, 0.0), Vec(-1.0, 0.0)) == PI`
pub fn angle_between(a: Vec, b: Vec) -> F64 = atan2(a.cross(b), a.dot(b))

/// The orthogonal projection of `p` onto the line through `a` and `b`:
/// `a + (b - a) * ((p - a).dot(b - a) / (b - a).norm2)`.
///
/// cost: \(O(1)\)
/// traps: when `a == b`
/// example: `project(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == Vec(1.0, 0.0)`
pub fn project(p: Vec, a: Vec, b: Vec) -> Vec:
    assert a != b, "a line needs two distinct points"
    let ab = b - a
    a + ab * (vdot(p - a, ab) / ab.norm2)

/// The mirror image of `p` across the line through `a` and `b`: `project(p, a, b) * 2.0 - p`.
///
/// cost: \(O(1)\)
/// traps: when `a == b`
/// example: `reflect(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == Vec(1.0, -1.0)`
pub fn reflect(p: Vec, a: Vec, b: Vec) -> Vec:
    let proj = project(p, a, b)
    proj + (proj - p)

/// The perpendicular distance from `p` to the infinite line through `a` and `b`:
/// `|(b - a).cross(p - a)| / (b - a).len`.
///
/// cost: \(O(1)\)
/// traps: when `a == b`
/// example: `line_dist(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 1.0`
/// example: `line_dist(Vec(5.0, 3.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 3.0`
pub fn line_dist(p: Vec, a: Vec, b: Vec) -> F64:
    assert a != b, "a line needs two distinct points"
    abs(vcross(b - a, p - a)) / (b - a).len

/// The shortest distance from `p` to the closed segment `a b`: to the projection when it falls
/// inside the segment, else to the nearer endpoint. A segment shorter than `eps` counts as the
/// point `a`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `seg_dist(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 1.0`
/// example: `seg_dist(Vec(-3.0, 4.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 5.0`
/// example: `seg_dist(Vec(3.0, 4.0), Vec(0.0, 0.0), Vec(0.0, 0.0)) == 5.0`
pub fn seg_dist(p: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> F64:
    let ab = b - a
    let n2 = ab.norm2
    if n2 <= eps * eps: return (p - a).len
    let t = vdot(p - a, ab) / n2
    if t <= 0.0: return (p - a).len
    if t >= 1.0: return (p - b).len
    (p - (a + ab * t)).len

/// The intersection of the infinite lines `a b` and `c d`, or `None` when
/// `|(b - a).cross(d - c)| <= eps` (parallel or coincident).
///
/// The point is `a + (b - a) * t` with `t = (c - a).cross(d - c) / (b - a).cross(d - c)`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `line_intersect(Vec(0.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0), Vec(2.0, 0.0)) == Some(Vec(1.0, 1.0))`
/// example: `line_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == None`
pub fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec:
    let ab = b - a
    let cd = d - c
    let den = vcross(ab, cd)
    if abs(den) <= eps: return None
    Some(a + ab * (vcross(c - a, cd) / den))

/// The common points of the closed segments `a b` and `c d`: `[]` when disjoint, one point when
/// they cross or touch, and the two ends of the shared piece when they overlap collinearly.
///
/// cost: \(O(1)\)
/// empty: `[]` when disjoint
/// order: by `(x, y)` ascending; a new `List`, keyed from 0
/// traps: never
/// example: `segment_intersect(Vec(0.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0), Vec(2.0, 0.0)) == [Vec(1.0, 1.0)]`
/// example: `segment_intersect(Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(3.0, 0.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0), Vec(2.0, 0.0)]`
/// example: `segment_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == []`
pub fn segment_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> List<Vec>:
    let d1 = b - a
    let d2 = d - c
    let cr = d1.cross(d2)
    if abs(cr) > eps:
        let t = (c - a).cross(d2) / cr
        let u = (c - a).cross(d1) / cr
        if t >= -eps and t <= 1.0 + eps and u >= -eps and u <= 1.0 + eps: return [a + d1 * t]
        return []
    if abs((c - a).cross(d1)) > eps or abs((a - c).cross(d2)) > eps: return []
    let lo = vmax(vmin(a, b), vmin(c, d))
    let hi = vmin(vmax(a, b), vmax(c, d))
    if vless(hi, lo):
        if lo.dist(hi) <= eps: return [lo]
        return []
    if lo.dist(hi) <= eps: return [lo]
    [lo, hi]

fn vless(p: Vec, q: Vec) -> Bool = p.x < q.x or (p.x == q.x and p.y < q.y)
fn vmin(p: Vec, q: Vec) -> Vec = if vless(q, p): q else: p
fn vmax(p: Vec, q: Vec) -> Vec = if vless(p, q): q else: p

/// The first point where the ray from `o` in direction `dir` meets the closed segment `a b`, or
/// `None`.
///
/// When the ray runs along the segment, the hit is the segment point nearest `o` (`o` itself if
/// it lies on the segment).
///
/// cost: \(O(1)\)
/// traps: when `dir` is the zero vector
/// example: `ray_hits_segment(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(2.0, -1.0), Vec(2.0, 1.0)) == Some(Vec(2.0, 0.0))`
/// example: `ray_hits_segment(Vec(0.0, 0.0), Vec(-1.0, 0.0), Vec(2.0, -1.0), Vec(2.0, 1.0)) == None`
/// example: `ray_hits_segment(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(3.0, 0.0), Vec(5.0, 0.0)) == Some(Vec(3.0, 0.0))`
pub fn ray_hits_segment(o: Vec, dir: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> ?Vec:
    assert dir.x != 0.0 or dir.y != 0.0, "ray_hits_segment needs a non-zero direction"
    let d2 = b - a
    let cr = dir.cross(d2)
    if abs(cr) > eps:
        let t = (a - o).cross(d2) / cr
        let u = (a - o).cross(dir) / cr
        if t >= -eps and -eps <= u <= 1.0 + eps: return Some(o + dir * t)
        return None
    if abs((a - o).cross(dir)) > eps or abs((b - o).cross(dir)) > eps: return None
    let ta = (a - o).dot(dir)
    let tb = (b - o).dot(dir)
    if ta < -eps and tb < -eps: return None
    if (ta <= eps and tb >= -eps) or (tb <= eps and ta >= -eps): return Some(o)
    Some(if ta < tb: a else: b)

/// The centre of the circle through `a`, `b` and `c`, or `None` when
/// `|(b - a).cross(c - a)| <= eps` (collinear).
///
/// cost: \(O(1)\)
/// traps: never
/// example: `circumcenter(Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0)) == Some(Vec(3.0, 4.0))`
/// example: `circumcenter(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None`
pub fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec:
    let u = b - a
    let v = c - a
    let den = 2.0 * vcross(u, v)
    if abs(den) <= eps: return None
    let u2 = u.norm2
    let v2 = v.norm2
    Some(a + Vec(v.y * u2 - u.y * v2, u.x * v2 - v.x * u2) / den)

/// \(O(1)\) value type. A 2D circle with center `c` and non-negative radius `r`.
pub struct Circle(pub c: Vec, pub r: F64):
    /// The circle through `a`, `b` and `c`: centre `circumcenter(a, b, c, eps)`, radius its distance
    /// to `a`; `None` when collinear.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Circle.through(Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0)) == Some(Circle(Vec(3.0, 4.0), 5.0))`
    /// example: `Circle.through(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None`
    pub fn through(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle:
        let o = circumcenter(a, b, c, eps) else: return None
        Some(Circle(o, o.dist(a)))

    /// The disk area, computed as `PI * r * r`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Circle(Vec(0.0, 0.0), 1.0).area == PI`
    @get pub fn area(self) -> F64 = pi() * r * r

    /// The circumference, computed as `2.0 * PI * r`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Circle(Vec(0.0, 0.0), 0.5).circumference == PI`
    @get pub fn circumference(self) -> F64 = 2.0 * pi() * r

    /// Where `p` is relative to the disk, with `d = c.dist(p)`: `Boundary` when `|d - r| <= eps`,
    /// else `Inside` when `d < r`, else `Outside`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Circle(Vec(0.0, 0.0), 5.0).locate(Vec(3.0, 4.0)) == Location.Boundary`
    /// example: `Circle(Vec(0.0, 0.0), 5.0).locate(Vec(1.0, 1.0)) == Location.Inside`
    /// example: `Circle(Vec(0.0, 0.0), 5.0).locate(Vec(6.0, 0.0)) == Location.Outside`
    pub fn locate(self, p: Vec, eps: F64 = 1e-9) -> Location:
        let d = c.dist(p)
        if abs(d - r) <= eps: Location.Boundary
        elif d < r: Location.Inside
        else: Location.Outside

/// The inscribed circle of the triangle `a b c`, or `None` when `|(b - a).cross(c - a)| <= eps`
/// (collinear).
///
/// The centre is the side-length-weighted mean `(|bc| a + |ca| b + |ab| c) / (|ab| + |bc| + |ca|)`
/// and the radius `|(b - a).cross(c - a)| / (|ab| + |bc| + |ca|)`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `incenter(Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0)) == Some(Circle(Vec(2.0, 2.0), 2.0))`
/// example: `incenter(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None`
pub fn incenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle:
    let k = abs((b - a).cross(c - a))
    if k <= eps: return None
    let la = b.dist(c)
    let lb = c.dist(a)
    let lc = a.dist(b)
    let per = la + lb + lc
    Some(Circle((a * la + b * lb + c * lc) / per, k / per))

/// The points where `circle` meets the infinite line through `a` and `b`: two when it crosses,
/// one (the foot of the perpendicular from the centre) when the distance is within `eps` of `r`,
/// and None otherwise.
///
/// cost: \(O(1)\)
/// empty: `[]` when the line misses
/// order: along the direction `a -> b`; a new `List`, keyed from 0
/// traps: when `a == b`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 3.0), Vec(10.0, 3.0)) == [Vec(-4.0, 3.0), Vec(4.0, 3.0)]`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(10.0, 3.0), Vec(-10.0, 3.0)) == [Vec(4.0, 3.0), Vec(-4.0, 3.0)]`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 5.0), Vec(10.0, 5.0)) == [Vec(0.0, 5.0)]`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 9.0), Vec(10.0, 9.0)) == []`
pub fn circle_line_intersect(circle: Circle, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>:
    assert a != b, "circle_line_intersect needs two distinct points"
    let foot = project(circle.c, a, b)
    let dist = foot.dist(circle.c)
    if dist > circle.r + eps: return []
    if abs(dist - circle.r) <= eps: return [foot]
    let off = (b - a).unit * sqrt(circle.r * circle.r - dist * dist)
    [foot - off, foot + off]

/// The points where the circles `a` and `b` meet: two when they cross, one when tangent (within
/// `eps`), None when apart, nested, or identical (infinitely many).
///
/// cost: \(O(1)\)
/// empty: `[]`
/// order: the point clockwise of the ray from `a.c` to `b.c` first; a new `List`, keyed from 0
/// traps: never
/// example: `circle_intersect(Circle(Vec(0.0, 0.0), 5.0), Circle(Vec(8.0, 0.0), 5.0)) == [Vec(4.0, -3.0), Vec(4.0, 3.0)]`
/// example: `circle_intersect(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(2.0, 0.0), 1.0)) == [Vec(1.0, 0.0)]`
/// example: `circle_intersect(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)) == []`
pub fn circle_intersect(a: Circle, b: Circle, eps: F64 = 1e-9) -> List<Vec>:
    let d = b.c - a.c
    let dist = d.len
    if dist <= eps: return []
    if dist > a.r + b.r + eps or dist < abs(a.r - b.r) - eps: return []
    let x = (dist * dist + a.r * a.r - b.r * b.r) / (2.0 * dist)
    let mid = a.c + d * (x / dist)
    if abs(dist - (a.r + b.r)) <= eps or abs(dist - abs(a.r - b.r)) <= eps: return [mid]
    let h2 = a.r * a.r - x * x
    let h = if h2 > 0.0: sqrt(h2) else: 0.0
    let off = d.perp * (h / dist)
    [mid - off, mid + off]

/// The points of `circle` where a tangent line through `p` touches it: two when `p` is outside,
/// `[p]` when `p` is on the circle (within `eps`), None when inside.
///
/// cost: \(O(1)\)
/// empty: `[]` when `p` is inside
/// order: the point clockwise of the ray from the centre to `p` first; a new `List`, keyed from 0
/// traps: never
/// example: `circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0)]`
/// example: `circle_tangents(Circle(Vec(0.0, 0.0), 2.0), Vec(1.0, 0.0)) == []`
/// example: `let t = circle_tangents(Circle(Vec(0.0, 0.0), 3.0), Vec(5.0, 0.0)); t.len == 2 and close(t[0].x, 1.8) and close(t[0].y, -2.4) and close(t[1].y, 2.4)`
pub fn circle_tangents(circle: Circle, p: Vec, eps: F64 = 1e-9) -> List<Vec>:
    let v = p - circle.c
    let d2 = v.norm2
    let dist = sqrt(d2)
    let r = circle.r
    if abs(dist - r) <= eps: return [p]
    if dist < r: return []
    let base = circle.c + v * (r * r / d2)
    let off = v.perp * (r * sqrt(d2 - r * r) / d2)
    [base - off, base + off]

/// The common tangent segments `(p1, p2)` of the circles `a` and `b`, `p1` on `a` and `p2` on
/// `b`: the outer tangents, or the inner ones (crossing between the centres) with `inner: true`.
///
/// Two segments when they exist, one (with `p1 == p2`) when the circles touch on that side, None
/// otherwise, and None for identical circles.
///
/// cost: \(O(1)\)
/// empty: `[]`
/// order: the tangent clockwise of the ray from `a.c` to `b.c` first; a new `List`, keyed from 0
/// traps: never
/// example: `circle_circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(4.0, 0.0), 1.0)) == [(Vec(0.0, -1.0), Vec(4.0, -1.0)), (Vec(0.0, 1.0), Vec(4.0, 1.0))]`
/// example: `circle_circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(4.0, 0.0), 1.0), inner: true).len == 2`
/// example: `circle_circle_tangents(Circle(Vec(0.0, 0.0), 2.0), Circle(Vec(1.0, 0.0), 2.0), inner: true) == []`
pub fn circle_circle_tangents(
    a: Circle,
    b: Circle,
    inner: Bool = false,
    eps: F64 = 1e-9,
) -> List<(Vec, Vec)>:
    let d = b.c - a.c
    let rb = if inner: -b.r else: b.r
    let dr = a.r - rb
    let d2 = d.norm2
    let h2 = d2 - dr * dr
    if d2 <= eps or h2 < -eps: return []
    let h = if h2 > 0.0: sqrt(h2) else: 0.0
    var out: List<(Vec, Vec)> = []
    for sign in [-1.0, 1.0]:
        let v = (d * dr + d.perp * (h * sign)) / d2
        out.push((a.c + v * a.r, b.c + v * rb))
    if abs(h2) <= eps: out.pop()
    out

/// The area of the intersection of the disks `a` and `b`: 0.0 when apart, the smaller disk's
/// `area` when one contains the other (within `eps`), else the sum of the two circular segments.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `circle_circle_area(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)) == 0.0`
/// example: `circle_circle_area(Circle(Vec(0.0, 0.0), 2.0), Circle(Vec(0.0, 0.0), 1.0)) == PI`
/// example: `close(circle_circle_area(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(1.0, 0.0), 1.0)), 1.228369698608757)`
pub fn circle_circle_area(a: Circle, b: Circle, eps: F64 = 1e-9) -> F64:
    let d = a.c.dist(b.c)
    if d >= a.r + b.r - eps: return 0.0
    if d <= abs(a.r - b.r) + eps:
        let r = if a.r < b.r: a.r else: b.r
        return pi() * r * r
    lens_part(a.r, b.r, d) + lens_part(b.r, a.r, d)

fn lens_part(r: F64, o: F64, d: F64) -> F64:
    let c = (d * d + r * r - o * o) / (2.0 * d * r)
    let s = sqrt(1.0 - c * c)
    r * r * (atan2(s, c) - s * c)

/// The inverse of `p` in the circle `across`: `c + (p - c) * (r * r / (p - c).norm2)`.
///
/// cost: \(O(1)\)
/// traps: when `p == across.c`
/// example: `invert(Vec(4.0, 0.0), Circle(Vec(0.0, 0.0), 2.0)) == Vec(1.0, 0.0)`
/// example: `invert(Vec(2.0, 0.0), Circle(Vec(0.0, 0.0), 2.0)) == Vec(2.0, 0.0)`
pub fn invert(p: Vec, across: Circle) -> Vec:
    let d = p - across.c
    let n = d.norm2
    assert n != 0.0, "`invert` needs `p` away from the centre"
    across.c + d * (across.r * across.r / n)

/// The area of the union of the disks `circles`, by Green's theorem over the arcs on the union's
/// boundary.
///
/// Equal circles (within `eps`) count once; zero-radius circles add nothing.
///
/// cost: \(O(n^2 \log n)\) time, \(O(n)\) memory
/// empty: 0.0
/// traps: never
/// example: `circles_union_area([]) == 0.0`
/// example: `close(circles_union_area([Circle(Vec(0.0, 0.0), 1.0)]), PI)`
/// example: `close(circles_union_area([Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)]), 2.0 * PI)`
pub fn circles_union_area(circles: List<Circle>, eps: F64 = 1e-9) -> F64:
    let pi = 3.14159265358979323846
    let raw_n = circles.len
    var cs: List<Circle> = []
    for i in 0..raw_n:
        if circles[i].r <= eps: continue
        var covered = false
        for j in 0..raw_n:
            if i == j or circles[j].r <= eps: continue
            let d = (circles[i].c - circles[j].c).len
            if d + circles[i].r <= circles[j].r + eps:
                let same = d <= eps and abs(circles[i].r - circles[j].r) <= eps
                if not same or j < i:
                    covered = true
                    break
        if not covered: cs.push(circles[i])
    let n = cs.len
    var total = 0.0
    for i in 0..n:
        let ci = cs[i].c
        let ri = cs[i].r
        let seam = Vec(ci.x - ri, ci.y)
        var ivs: List<(lo: F64, p_lo: Vec, hi: F64, p_hi: Vec)> = []
        for j in 0..n:
            if i == j: continue
            let d_vec = cs[j].c - ci
            let d = d_vec.len
            if d >= ri + cs[j].r - eps or d <= eps: continue
            let x = (d * d + ri * ri - cs[j].r * cs[j].r) / (2.0 * d)
            let h = sqrt(max(0.0, ri * ri - x * x))
            if h <= eps: continue
            let base = ci + d_vec * (x / d)
            let off = d_vec.perp * (h / d)
            let p1 = base - off
            let p2 = base + off
            let a1 = atan2(p1.y - ci.y, p1.x - ci.x)
            let a2 = atan2(p2.y - ci.y, p2.x - ci.x)
            if a1 <= a2:
                ivs.push((lo: a1, p_lo: p1, hi: a2, p_hi: p2))
            else:
                ivs.push((lo: a1, p_lo: p1, hi: pi, p_hi: seam))
                ivs.push((lo: -pi, p_lo: seam, hi: a2, p_hi: p2))
        if ivs.is_empty:
            total += pi * ri * ri
            continue
        ivs.sort(by: iv => iv.lo)
        var cur_ang = -pi
        var cur_pt = seam
        for iv in ivs:
            if iv.lo > cur_ang:
                total += 0.5 * (ri * ri * (iv.lo - cur_ang) + vcross(ci, iv.p_lo - cur_pt))
            if iv.hi > cur_ang:
                cur_ang = iv.hi
                cur_pt = iv.p_hi
        if pi > cur_ang:
            total += 0.5 * (ri * ri * (pi - cur_ang) + vcross(ci, seam - cur_pt))
    total

// ---- half-plane intersection

/// \(O(1)\) value type. Directed line through `p` in direction `dir`, keeping the closed left
/// half-plane `{ q | dir.cross(q - p) >= 0 }`.
pub struct HalfPlane(p: Vec, dir: Vec):
    /// The closed half-plane left of the directed line `a -> b`: `HalfPlane(a, b - a)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `a == b`
    /// example: `HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)) == HalfPlane(Vec(0.0, 0.0), Vec(1.0, 0.0))`
    pub fn through(a: Vec, b: Vec) -> HalfPlane:
        assert a != b, "`HalfPlane.through` needs two distinct points"
        HalfPlane(a, b - a)

    /// The half-plane `{ (x, y) | a*x + b*y + c >= 0 }`, as `HalfPlane(p, Vec(b, -a))` with `p` the
    /// line's point nearest the origin, `Vec(-a * c, -b * c) / (a*a + b*b)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `a == 0.0` and `b == 0.0`
    /// example: `HalfPlane.from_line(0.0, 1.0, -2.0) == HalfPlane(Vec(0.0, 2.0), Vec(1.0, 0.0))`
    /// example: `HalfPlane.from_line(1.0, 0.0, 0.0).side(Vec(3.0, 7.0)) == 3.0`
    pub fn from_line(a: F64, b: F64, c: F64) -> HalfPlane:
        let n = a * a + b * b
        assert n != 0.0, "`HalfPlane.from_line` needs `a` or `b` nonzero"
        HalfPlane(Vec(a, b) * (-c / n), Vec(b, -a))

    /// The signed value `dir.cross(q - p)`: positive left of the line (inside), 0 on it, negative
    /// outside. It is `|dir|` times the signed distance.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)).side(Vec(3.0, 2.0)) == 2.0`
    /// example: `HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)).side(Vec(3.0, -2.0)) == -2.0`
    pub fn side(self, q: Vec) -> F64 = dir.cross(q - p)

fn hp_out(
    h: (p: Vec, dir: Vec, ang: F64),
    pt: Vec,
    eps: F64,
) -> Bool = vcross(h.dir, pt - h.p) < -eps

fn hp_cross(a: (p: Vec, dir: Vec, ang: F64), b: (p: Vec, dir: Vec, ang: F64)) -> Vec:
    let den = vcross(a.dir, b.dir)
    a.p + a.dir * (vcross(b.p - a.p, b.dir) / den)

/// The intersection of the closed half-planes `planes`, as a counter-clockwise convex polygon
/// (sort by angle, then a deque sweep).
///
/// With `bound: Some(b)` the box `[-b, b]^2` is added as four more half-planes. A region that is
/// unbounded, empty, or of area at most `eps` gives `[]`. Parallel planes keep the tighter one;
/// vertices within `eps` merge.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]` (unbounded) without `bound`; the box with one
/// order: counter-clockwise, starting at the least `(y, x)`, `y` compared within `eps`; a new
/// `List`, keyed from 0
/// traps: when a plane's `dir` is the zero vector
/// example: `half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)), HalfPlane.through(Vec(1.0, 0.0), Vec(1.0, 1.0)), HalfPlane.through(Vec(1.0, 1.0), Vec(0.0, 1.0)), HalfPlane.through(Vec(0.0, 1.0), Vec(0.0, 0.0))]) == [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(0.0, 1.0)]`
/// example: `half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0))]) == []`
/// example: `half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0))], bound: Some(1.0)) == [Vec(-1.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(-1.0, 1.0)]`
pub fn half_plane_intersect(
    planes: List<HalfPlane>,
    bound: ?F64 = None,
    eps: F64 = 1e-9,
) -> List<Vec>:
    var all = copy(planes)
    if bound != None:
        let b = bound!
        all.push(HalfPlane.through(Vec(-b, -b), Vec(b, -b)))
        all.push(HalfPlane.through(Vec(b, -b), Vec(b, b)))
        all.push(HalfPlane.through(Vec(b, b), Vec(-b, b)))
        all.push(HalfPlane.through(Vec(-b, b), Vec(-b, -b)))
    if all.is_empty: return []
    var hs: List<(p: Vec, dir: Vec, ang: F64)> = []
    for h in all:
        assert h.dir.x != 0.0 or h.dir.y != 0.0, "`half_plane_intersect` needs non-zero directions"
        hs.push((p: h.p, dir: h.dir, ang: atan2(h.dir.y, h.dir.x)))
    hs.sort(by: h => h.ang)
    let tau = 2.0 * pi()
    for i in 0..hs.len:
        let gap = if i + 1 < hs.len: hs[i + 1].ang - hs[i].ang else: hs[0].ang + tau - hs[i].ang
        if gap >= pi() - 1e-12: return []
    var uniq: List<(p: Vec, dir: Vec, ang: F64)> = []
    for h in hs:
        if not uniq.is_empty:
            let last = uniq.last!
            if abs(vcross(last.dir, h.dir)) <= eps and last.dir.dot(h.dir) > 0.0:
                if vcross(last.dir, h.p - last.p) > 0.0: uniq[uniq.len - 1] = h
                continue
        uniq.push(h)
    var dq: List<(p: Vec, dir: Vec, ang: F64)> = []
    var head = 0
    for h in uniq:
        while dq.len - head >= 2 and hp_out(
            h,
            hp_cross(dq.last!, dq[dq.len - 2]),
            eps,
        ): dq.pop()
        while dq.len - head >= 2 and hp_out(h, hp_cross(dq[head], dq[head + 1]), eps): head += 1
        if dq.len - head >= 1 and vcross(dq.last!.dir, h.dir) <= eps: return []
        dq.push(h)
    while dq.len - head >= 3 and hp_out(
        dq[head],
        hp_cross(dq.last!, dq[dq.len - 2]),
        eps,
    ): dq.pop()
    while dq.len - head >= 3 and hp_out(
        dq.last!,
        hp_cross(dq[head], dq[head + 1]),
        eps,
    ): head += 1
    let m = dq.len - head
    if m < 3: return []
    var pts: List<Vec> = []
    for i in 0..m:
        let v = hp_cross(dq[head + i], dq[head + (i + 1) % m])
        if pts.is_empty or pts.last!.dist(v) > eps: pts.push(v)
    while pts.len >= 2 and pts[0].dist(pts.last!) <= eps: pts.pop()
    if pts.len < 3: return []
    let a2 = 0.0 + sum(pts[i].cross(pts[(i + 1) % pts.len]) for i in 0..pts.len)
    if a2 / 2.0 <= eps: return []
    var s = 0
    for i in 1..pts.len:
        let dy = pts[i].y - pts[s].y
        if dy < -eps or (dy <= eps and pts[i].x < pts[s].x): s = i
    [pts[(s + i) % pts.len] for i in 0..pts.len]

/// The signed area of the polygon `poly` (half the shoelace sum): positive counter-clockwise,
/// negative clockwise; 0.0 for fewer than 3 vertices.
///
/// cost: \(O(n)\)
/// empty: 0.0
/// traps: never
/// example: `polygon_area([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(0.0, 2.0)]) == 2.0`
/// example: `polygon_area([Vec(0.0, 0.0), Vec(0.0, 2.0), Vec(2.0, 0.0)]) == -2.0`
pub fn polygon_area(poly: List<Vec>) -> F64:
    let n = poly.len
    if n < 3: return 0.0
    var s = 0.0
    var a = poly[n - 1]
    for b in poly:
        s += a.x * b.y - a.y * b.x
        a = b
    0.5 * s

/// The part of `poly` in the closed half-plane left of the directed line `a -> b`
/// (Sutherland-Hodgman).
///
/// Walking the edges from `poly[0]`, it keeps each vertex with `turn_eps(a, b, v, eps) >= 0`
/// and adds the crossing point of every edge that strictly crosses the line. The result keeps the
/// orientation of `poly`.
///
/// cost: \(O(n)\) time and memory
/// empty: `[]`
/// order: the walk from `poly[0]`; a new `List`, keyed from 0
/// traps: when `a == b`
/// example: `polygon_cut([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], Vec(1.0, 0.0), Vec(1.0, 2.0)) == [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 2.0), Vec(0.0, 2.0)]`
/// example: `polygon_cut([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], Vec(1.0, 2.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(1.0, 2.0)]`
/// example: `polygon_cut([Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0)], Vec(0.0, 5.0), Vec(1.0, 5.0)) == []`
pub fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>:
    let n = poly.len
    if n == 0: return []
    let d = b - a
    var out: List<Vec> = []
    for i in 0..n:
        let cur = poly[i]
        let nxt = poly[(i + 1) % n]
        let sc = vcross(d, cur - a)
        let sn = vcross(d, nxt - a)
        if sc >= -eps: out.push(cur)
        if (sc > eps and sn < -eps) or (sc < -eps and sn > eps):
            let t = sc / (sc - sn)
            out.push(cur + (nxt - cur) * t)
    out

/// The smallest circle containing every point of `pts` (Welzl's algorithm, deterministic: points
/// are processed in input order, with no shuffle).
///
/// The circle is unique; its centre is the midpoint of two points or a circumcentre of three.
/// Without the shuffle the expected \(O(n)\) bound needs input not ordered adversarially; shuffle
/// first with `Rng.shuffle` if it might be.
///
/// cost: \(O(n)\) expected for unadversarial order
/// empty: `Circle(Vec(0.0, 0.0), 0.0)`
/// ties: None; the smallest enclosing circle is unique
/// traps: never
/// example: `min_enclosing_circle([Vec(0.0, 0.0), Vec(2.0, 0.0)]) == Circle(Vec(1.0, 0.0), 1.0)`
/// example: `min_enclosing_circle([Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0), Vec(1.0, 1.0)]) == Circle(Vec(3.0, 4.0), 5.0)`
/// example: `min_enclosing_circle([Vec(2.0, 3.0)]) == Circle(Vec(2.0, 3.0), 0.0)`
pub fn min_enclosing_circle(pts: List<Vec>, eps: F64 = 1e-9) -> Circle:
    let n = pts.len
    if n == 0: return Circle(Vec(0.0, 0.0), 0.0)
    if n == 1: return Circle(pts[0], 0.0)
    var ps = copy(pts)
    var rng = 123456789
    for i in (1..n).rev:
        rng = (rng * 1103515245 + 12345) % 2147483647
        let j = rng % (i + 1)
        let tmp = ps[i]
        ps[i] = ps[j]
        ps[j] = tmp
    var c = ps[0]
    var r = 0.0
    for i in 0..n:
        if vdist(c, ps[i]) > r + eps:
            c = ps[i]
            r = 0.0
            for j in 0..i:
                if vdist(c, ps[j]) > r + eps:
                    c = (ps[i] + ps[j]) * 0.5
                    r = vdist(c, ps[i])
                    for k in 0..j:
                        if vdist(c, ps[k]) > r + eps:
                            if let o = circumcenter(ps[i], ps[j], ps[k], eps:):
                                c = o
                                r = vdist(c, ps[i])
    Circle(c, r)

fn tri_circle_area(a: Vec, b: Vec, r: F64, eps: F64) -> F64:
    let r2 = r * r
    let d = b - a
    let a2 = vdot(d, d)
    if a2 <= eps * eps: return 0.0
    let s = vcross(a, b)
    if abs(s) <= eps * eps: return 0.0
    if a.norm2 <= r2 + eps and b.norm2 <= r2 + eps:
        return 0.5 * s
    let t_mid = -vdot(a, d) / a2
    let foot = a + d * t_mid
    let h2 = foot.norm2
    if h2 >= r2 - eps:
        return 0.5 * r2 * atan2(s, vdot(a, b))
    let dt = sqrt(max(0.0, r2 - h2) / a2)
    let t1 = max(0.0, min(1.0, t_mid - dt))
    let t2 = max(0.0, min(1.0, t_mid + dt))
    let p1 = a + d * t1
    let p2 = a + d * t2
    var res = 0.0
    if t1 > eps: res += 0.5 * r2 * atan2(vcross(a, p1), vdot(a, p1))
    res += 0.5 * vcross(p1, p2)
    if 1.0 - t2 > eps: res += 0.5 * r2 * atan2(vcross(p2, b), vdot(p2, b))
    res

/// The signed area of the intersection of the disk `circle` with the simple polygon `poly`:
/// positive when `poly` is counter-clockwise, negative when clockwise. Summed over the triangles
/// from the centre to each edge.
///
/// cost: \(O(n)\)
/// empty: 0.0
/// traps: never
/// example: `circle_polygon_area(Circle(Vec(0.0, 0.0), 10.0), [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(0.0, 1.0)]) == 1.0`
/// example: `circle_polygon_area(Circle(Vec(0.0, 0.0), 10.0), [Vec(0.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0), Vec(1.0, 0.0)]) == -1.0`
/// example: `close(circle_polygon_area(Circle(Vec(0.0, 0.0), 1.0), [Vec(-2.0, -2.0), Vec(2.0, -2.0), Vec(2.0, 2.0), Vec(-2.0, 2.0)]), PI)`
pub fn circle_polygon_area(circle: Circle, poly: List<Vec>, eps: F64 = 1e-9) -> F64:
    let acc = 0.0 + sum(
        tri_circle_area(
            poly[i] - circle.c,
            poly[(i + 1) % poly.len] - circle.c,
            circle.r,
            eps,
        ) for i in 0..poly.len
    )
    acc

// ---- Shamos-Hoey segment intersection detection

fn sh_seg_cmp(
    a1: Pt,
    b1: Pt,
    id1: Int,
    a2: Pt,
    b2: Pt,
    id2: Int,
    cur_x: Int,
    right_eps: Bool,
) -> Int:
    let dx1 = I128(b1.x - a1.x)
    let dx2 = I128(b2.x - a2.x)
    let lhs = (I128(a1.y) * dx1 + I128(b1.y - a1.y) * I128(cur_x - a1.x)) * dx2
    let rhs = (I128(a2.y) * dx2 + I128(b2.y - a2.y) * I128(cur_x - a2.x)) * dx1
    if lhs != rhs:
        return if lhs < rhs: -1 else: 1
    let c = cross128(b1 - a1, b2 - a2)
    if c != I128(0):
        let lt = if right_eps: c > I128(0) else: c < I128(0)
        return if lt: -1 else: 1
    if id1 < id2: -1 elif id1 > id2: 1 else: 0

fn sh_seg_above_y(a: Pt, b: Pt, cur_x: Int, y: Int, strict: Bool) -> Bool:
    let dx = I128(b.x - a.x)
    let num = I128(a.y) * dx + I128(b.y - a.y) * I128(cur_x - a.x)
    let rhs = I128(y) * dx
    if strict: num > rhs else: num >= rhs

fn sh_merge(
    a: Int,
    b: Int,
    mut left_ch: List<Int>,
    mut right_ch: List<Int>,
    prio: List<Int>,
) -> Int:
    if a == -1: return b
    if b == -1: return a
    if prio[a] > prio[b]:
        let r = sh_merge(right_ch[a], b, mut left_ch, mut right_ch, prio)
        right_ch[a] = r
        a
    else:
        let l = sh_merge(a, left_ch[b], mut left_ch, mut right_ch, prio)
        left_ch[b] = l
        b

fn sh_split_lt(
    u: Int,
    id: Int,
    lo_id: Pt,
    hi_id: Pt,
    cur_x: Int,
    right_eps: Bool,
    mut left_ch: List<Int>,
    mut right_ch: List<Int>,
    lo: List<Pt>,
    hi: List<Pt>,
) -> (Int, Int):
    if u == -1: return (-1, -1)
    if sh_seg_cmp(lo[u], hi[u], u, lo_id, hi_id, id, cur_x, right_eps) < 0:
        let (rl, rr) = sh_split_lt(
            right_ch[u],
            id,
            lo_id,
            hi_id,
            cur_x,
            right_eps,
            mut left_ch,
            mut right_ch,
            lo,
            hi,
        )
        right_ch[u] = rl
        (u, rr)
    else:
        let (ll, lr) = sh_split_lt(
            left_ch[u],
            id,
            lo_id,
            hi_id,
            cur_x,
            right_eps,
            mut left_ch,
            mut right_ch,
            lo,
            hi,
        )
        left_ch[u] = lr
        (ll, u)

fn sh_pop_leftmost(u: Int, mut left_ch: List<Int>, right_ch: List<Int>) -> Int:
    if u == -1: return -1
    if left_ch[u] == -1: return right_ch[u]
    var p = u
    while left_ch[left_ch[p]] != -1: p = left_ch[p]
    left_ch[p] = right_ch[left_ch[p]]
    u

fn sh_leftmost(u0: Int, left_ch: List<Int>) -> Int:
    var u = u0
    if u == -1: return -1
    while left_ch[u] != -1: u = left_ch[u]
    u

fn sh_rightmost(u0: Int, right_ch: List<Int>) -> Int:
    var u = u0
    if u == -1: return -1
    while right_ch[u] != -1: u = right_ch[u]
    u

fn sh_first_above_y(
    u0: Int,
    cur_x: Int,
    y: Int,
    strict: Bool,
    left_ch: List<Int>,
    right_ch: List<Int>,
    lo: List<Pt>,
    hi: List<Pt>,
) -> Int:
    var u = u0
    var best = -1
    while u != -1:
        if sh_seg_above_y(lo[u], hi[u], cur_x, y, strict):
            best = u
            u = left_ch[u]
        else:
            u = right_ch[u]
    best

/// Whether any two segments of `segs` share a point, by a Shamos-Hoey sweep with exact `I128`
/// comparisons (valid for `|x|, |y| <= 1e12`): `Some((i, j))` with `i < j` for a pair that
/// meets, or `None`.
///
/// With `strict: true` only proper interior crossings count, as in `segments_meet`. When several
/// pairs meet, the pair is the first one the sweep finds, not necessarily the least.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `None`
/// ties: the first meeting pair found by the sweep, as `(i, j)` with `i < j`
/// traps: never
/// example: `any_segments_meet([(Pt(0, 0), Pt(2, 2)), (Pt(5, 5), Pt(6, 6)), (Pt(0, 2), Pt(2, 0))]) == Some((0, 2))`
/// example: `any_segments_meet([(Pt(0, 0), Pt(1, 0)), (Pt(0, 1), Pt(1, 1))]) == None`
/// example: `any_segments_meet([(Pt(0, 0), Pt(1, 1)), (Pt(1, 1), Pt(2, 0))], strict: true) == None`
pub fn any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int):
    let n = segs.len
    if n < 2: return None
    var lo: List<Pt> = []
    var hi: List<Pt> = []
    // Event: (x, kind, y_tie, id)
    // strict == false: kind 0 = non-vert insert, 1 = vertical query, 2 = non-vert delete
    // strict == true:  kind 0 = non-vert delete, 1 = vertical query, 2 = non-vert insert
    var events: List<(x: Int, kind: Int, y: Int, id: Int)> = []
    var left_ch: List<Int> = []
    var right_ch: List<Int> = []
    var prio: List<Int> = []
    var rng = 987654321
    for i in 0..n:
        let s = segs[i]
        let (p, q) = if s.a.x < s.b.x or (s.a.x == s.b.x and s.a.y <= s.b.y):
            (s.a, s.b)
        else:
            (s.b, s.a)
        lo.push(p)
        hi.push(q)
        rng = (rng * 1103515245 + 12345) % 2147483647
        left_ch.push(-1)
        right_ch.push(-1)
        prio.push(rng)
        if p.x == q.x:
            events.push((x: p.x, kind: 1, y: p.y, id: i))
        elif strict:
            events.push((x: p.x, kind: 2, y: p.y, id: i))
            events.push((x: q.x, kind: 0, y: q.y, id: i))
        else:
            events.push((x: p.x, kind: 0, y: p.y, id: i))
            events.push((x: q.x, kind: 2, y: q.y, id: i))
    events.sort()

    var root = -1
    var last_vert_x = 0
    var last_vert_hi = 0
    var last_vert_id = -1

    for ev in events:
        let cur_x = ev.x
        let id = ev.id
        if ev.kind == 1:
            if not strict:
                if last_vert_id != -1 and last_vert_x == cur_x and last_vert_hi >= lo[id].y:
                    return Some((min(last_vert_id, id), max(last_vert_id, id)))
                if last_vert_id == -1 or last_vert_x != cur_x or hi[id].y > last_vert_hi:
                    last_vert_x = cur_x
                    last_vert_hi = hi[id].y
                    last_vert_id = id
            let cand = sh_first_above_y(root, cur_x, lo[id].y, strict, left_ch, right_ch, lo, hi)
            if cand != -1 and segments_meet(lo[id], hi[id], lo[cand], hi[cand], strict:):
                return Some((min(id, cand), max(id, cand)))
        elif (not strict and ev.kind == 0) or (strict and ev.kind == 2):
            let lo_id = lo[id]
            let hi_id = hi[id]
            let (l, r) = sh_split_lt(
                root,
                id,
                lo_id,
                hi_id,
                cur_x,
                true,
                mut left_ch,
                mut right_ch,
                lo,
                hi,
            )
            let pred = sh_rightmost(l, right_ch)
            let succ = sh_leftmost(r, left_ch)
            if pred != -1 and segments_meet(lo_id, hi_id, lo[pred], hi[pred], strict:):
                return Some((min(id, pred), max(id, pred)))
            if succ != -1 and segments_meet(lo_id, hi_id, lo[succ], hi[succ], strict:):
                return Some((min(id, succ), max(id, succ)))
            let lm = sh_merge(l, id, mut left_ch, mut right_ch, prio)
            root = sh_merge(lm, r, mut left_ch, mut right_ch, prio)
        else:
            let lo_id = lo[id]
            let hi_id = hi[id]
            let (l, mr) = sh_split_lt(
                root,
                id,
                lo_id,
                hi_id,
                cur_x,
                false,
                mut left_ch,
                mut right_ch,
                lo,
                hi,
            )
            let r = sh_pop_leftmost(mr, mut left_ch, right_ch)
            let pred = sh_rightmost(l, right_ch)
            let succ = sh_leftmost(r, left_ch)
            if pred != -1 and succ != -1 and segments_meet(
                lo[pred],
                hi[pred],
                lo[succ],
                hi[succ],
                strict:,
            ):
                return Some((min(pred, succ), max(pred, succ)))
            root = sh_merge(l, r, mut left_ch, mut right_ch, prio)
    None

/// `any_segments_meet` over a domain-indexed array: `Some((a, b))` with `a` before `b` in key
/// order for a pair of segments that meet, or `None`.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `None`
/// ties: the first meeting pair found by the sweep, `a` before `b` in key order
/// traps: never
/// example: `any_segments_meet_idx([(Pt(0, 0), Pt(1, 1)), (Pt(1, 1), Pt(2, 0))]) == Some((0, 1))`
pub fn any_segments_meet_idx<D>(segs: [D](a: Pt, b: Pt), strict: Bool = false) -> ?(D, D):
    let keys = D.collect()
    if let (i, j) = any_segments_meet([segs[k] for k in keys], strict:):
        return Some((keys[i], keys[j]))
    None

// ---- axis-aligned rectangles, planar subdivisions, and Delaunay / Voronoi

/// \(O(1)\) value type. Half-open integer rectangle `x * y` (accepts cut ranges per decision 124).
pub struct Rect(pub x: Range<Int>, pub y: Range<Int>):
    /// The number of unit cells, `x.len * y.len`; 0 when either range is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: when the product overflows `Int`
    /// example: `Rect(0..3, 0..2).area == 6`
    /// example: `Rect(3..1, 0..2).area == 0`
    @get pub fn area(self) -> Int = max(x.end - x.start, 0) * max(y.end - y.start, 0)

    /// Whether either axis range is empty.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Rect(0..0, 0..5).is_empty == true`
    /// example: `Rect(0..1, 0..1).is_empty == false`
    @get pub fn is_empty(self) -> Bool = x.end <= x.start or y.end <= y.start

    /// Whether `p.x` is in `x` and `p.y` is in `y` (half-open, so the top and right edges are out).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Rect(0..3, 0..2).contains(Pt(2, 1)) == true`
    /// example: `Rect(0..3, 0..2).contains(Pt(3, 1)) == false`
    pub fn contains(
        self,
        p: Pt,
    ) -> Bool = x.start <= p.x and p.x < x.end and y.start <= p.y and p.y < y.end

    /// The common part of two rectangles, or `None` when it is empty (including rectangles that only
    /// touch along an edge).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Rect(0..3, 0..3).intersect(Rect(1..5, 2..4)) == Some(Rect(1..3, 2..3))`
    /// example: `Rect(0..1, 0..1).intersect(Rect(1..2, 0..1)) == None`
    pub fn intersect(self, o: Rect) -> ?Rect:
        let r = Rect(
            max(x.start, o.x.start)..min(x.end, o.x.end),
            max(y.start, o.y.start)..min(y.end, o.y.end),
        )
        if r.is_empty: None else: Some(r)

/// The area of the union of `rects`, by a sweep over x with a segment tree over the compressed y
/// coordinates. Empty rectangles add nothing.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: 0
/// traps: when the area overflows `Int`
/// example: `rects_union_area([Rect(0..2, 0..2), Rect(1..3, 1..3)]) == 7`
/// example: `rects_union_area([Rect(0..2, 0..2), Rect(0..2, 0..2)]) == 4`
/// example: `rects_union_area([]) == 0`
pub fn rects_union_area(rects: List<Rect>) -> Int:
    var ys: List<Int> = []
    var ev: List<(x: Int, k: Int, lo: Int, hi: Int)> = []
    for r in rects:
        if r.is_empty: continue
        ys.push(r.y.start)
        ys.push(r.y.end)
    ys.sort()
    var u: List<Int> = []
    for y in ys:
        if u.is_empty or u.last! != y: u.push(y)
    ys = u
    if ys.len < 2: return 0
    for r in rects:
        if r.is_empty: continue
        let lo = lower_idx(ys, r.y.start)
        let hi = lower_idx(ys, r.y.end)
        ev.push((x: r.x.start, k: 1, lo:, hi:))
        ev.push((x: r.x.end, k: -1, lo:, hi:))
    ev.sort()
    let m = ys.len - 1
    var cnt = List.filled(4 * m, 0)
    var len = List.filled(4 * m, 0)
    var area = 0
    for i in 0..ev.len:
        if i > 0: area += len[1] * (ev[i].x - ev[i - 1].x)
        cover(mut cnt, mut len, ys, 1, 0, m, ev[i].lo, ev[i].hi, ev[i].k)
    area

fn lower_idx(xs: List<Int>, v: Int) -> Int:
    let lo = xs.first_ge(v) ?? xs.len
    lo

fn cover(
    mut cnt: List<Int>,
    mut len: List<Int>,
    ys: List<Int>,
    v: Int,
    l: Int,
    r: Int,
    lo: Int,
    hi: Int,
    k: Int,
):
    if hi <= l or r <= lo: return
    if lo <= l and r <= hi:
        cnt[v] += k
    else:
        let mid = (l + r) / 2
        cover(mut cnt, mut len, ys, 2 * v, l, mid, lo, hi, k)
        cover(mut cnt, mut len, ys, 2 * v + 1, mid, r, lo, hi, k)
    len[v] = if cnt[v] > 0: ys[r] - ys[l] else: if r - l == 1: 0 else: len[2 * v] + len[2 * v + 1]

/// The faces of the planar straight-line graph with vertices `pts` and undirected `edges`, each
/// as its cycle of vertices: bounded faces counter-clockwise, the outer face of each connected
/// component clockwise.
///
/// Each face is traced by turning to the next edge clockwise at every vertex. A bridge edge
/// appears twice in its face; isolated vertices form no face. The graph must be planar (edges
/// meet only at shared endpoints); that is not checked.
///
/// cost: \(O((n + m) \log m)\) time, \(O(n + m)\) memory
/// empty: `[]` when `edges` is empty
/// order: each cycle starts at its least vertex in key order; faces sorted by cycle; a new `List`
/// traps: when an edge joins a vertex to itself, or an edge repeats
/// example: `planar_faces([Pt(0, 0), Pt(1, 0), Pt(0, 1)], [(0, 1), (1, 2), (2, 0)]) == [[0, 1, 2], [0, 2, 1]]`
/// example: `planar_faces([Pt(0, 0), Pt(1, 0)], [(0, 1)]) == [[0, 1]]`
pub fn planar_faces<V>(pts: [V]Pt, edges: List<(V, V)>) -> List<List<V>>:
    with lint(unproven_list_index: allow)
    let keys = pts.keys.collect()
    let walk = planar_walk([pts[k] for k in keys], [(a.ord, b.ord) for (a, b) in edges])
    [[keys[i] for i in f] for f in walk.faces]

struct PlanarWalk:
    var faces: List<List<Int>>
    off: List<Int>
    face_of: List<Int>
    half: List<(Int, Int, Int)>

    fn at(self, u: Int, v: Int) -> Int:
        with lint(unproven_list_index: allow)
        let lo = (0..half.len).first_true(mid => (half[mid].0, half[mid].1) >= (u, v)) ?? half.len
        half[lo].2

    fn side(self, u: Int, v: Int) -> Int:
        with lint(unproven_list_index: allow)
        return face_of[off[u] + self.at(u, v)]

fn planar_walk(ps: List<Pt>, edges: List<(Int, Int)>) -> PlanarWalk:
    with lint(unproven_list_index: allow)
    let n = ps.len
    var adj: List<List<Int>> = [[] for _ in 0..n]
    var norm: List<(Int, Int)> = []
    for (u, v) in edges:
        assert u != v, "`planar_faces` needs no edge from a vertex to itself"
        norm.push((min(u, v), max(u, v)))
        adj[u].push(v)
        adj[v].push(u)
    norm.sort()
    for i in 1..norm.len: assert norm[i] != norm[i - 1], "`planar_faces` needs no repeated edge"
    var off = List.filled(n + 1, 0)
    for u in 0..n:
        let o = ps[u]
        adj[u].sort(by: v => ByAngle(ps[v] - o))
        off[u + 1] = off[u] + adj[u].len
    var half: List<(Int, Int, Int)> = []
    for u in 0..n:
        for j in 0..adj[u].len: half.push((u, adj[u][j], j))
    half.sort()
    var walk = PlanarWalk([], off.clone(), List.filled(off[n], 0), half)
    var done = List.filled(off[n], false)
    var raw: List<List<Int>> = []
    for u in 0..n:
        for j in 0..adj[u].len:
            if done[off[u] + j]: continue
            var cyc: List<Int> = []
            var (a, k) = (u, j)
            while not done[off[a] + k]:
                done[off[a] + k] = true
                walk.face_of[off[a] + k] = raw.len
                cyc.push(a)
                let b = adj[a][k]
                let deg = adj[b].len
                k = (walk.at(b, a) + deg - 1) % deg
                a = b
            raw.push(least_rotation(cyc))
    var order = [f for f in 0..raw.len]
    order.sort(by: f => raw[f])
    var rank = List.filled(raw.len, 0)
    for i in 0..order.len: rank[order[i]] = i
    for h in 0..walk.face_of.len: walk.face_of[h] = rank[walk.face_of[h]]
    walk.faces = [raw[f] for f in order]
    walk

fn least_rotation(cyc: List<Int>) -> List<Int>:
    // `<` on lists is lexicographic; `best min= rot` would take the cellwise minimum.
    with lint(unproven_list_index: allow, chmin_chmax_if: allow)
    let m = cyc.len
    var best: List<Int> = copy(cyc)
    for s in 1..m:
        let rot = [cyc[(s + i) % m] for i in 0..m]
        if rot < best: best = rot
    best

/// For each query point, the index into `planar_faces(pts, edges)` of the bounded face that
/// contains it, or `None` when it lies on an edge or vertex or in the unbounded region (offline
/// sweep).
///
/// cost: \(O((n + m + q) \log(n + m))\) time, \(O(n + m + q)\) memory
/// empty: `[]` when `queries` is empty
/// order: one answer per query, in query order; a new `List`, keyed from 0
/// traps: as `planar_faces`
/// example: `point_location_planar([Pt(0, 0), Pt(4, 0), Pt(0, 4)], [(0, 1), (1, 2), (2, 0)], [Pt(1, 1), Pt(0, 0), Pt(5, 5)]) == [Some(0), None, None]`
pub fn point_location_planar<V>(pts: [V]Pt, edges: List<(V, V)>, queries: List<Pt>) -> List<?Int>:
    let keys = pts.keys.collect()
    locate_planar([pts[k] for k in keys], [(a.ord, b.ord) for (a, b) in edges], queries)

struct SweepSeg(a: Pt, b: Pt, id: Int)

impl Ord for SweepSeg:
    fn lt(s, t):
        if s.id == t.id: return false
        var c = 0
        if s.a.x <= t.a.x:
            c = turn(s.a, s.b, t.a)
            if c == 0: c = turn(s.a, s.b, t.b)
        else:
            c = -turn(t.a, t.b, s.a)
            if c == 0: c = -turn(t.a, t.b, s.b)
        if c == 0: s.id < t.id else: c > 0

fn locate_planar(ps: List<Pt>, edges: List<(Int, Int)>, queries: List<Pt>) -> List<?Int>:
    with lint(unproven_list_index: allow)
    let walk = planar_walk(ps, edges)
    let nf = walk.faces.len
    var outer = List.filled(nf, false)
    var probes = copy(queries)
    var probed: List<Int> = []
    for f in 0..nf:
        let cyc = walk.faces[f]
        var area = I128(0)
        for i in 0..cyc.len: area += ps[cyc[i]].cross128(ps[cyc[(i + 1) % cyc.len]])
        if area > I128(0): continue
        outer[f] = true
        var low = cyc[0]
        for v in cyc:
            if (ps[v].y, ps[v].x) < (ps[low].y, ps[low].x): low = v
        probes.push(ps[low])
        probed.push(f)
    let below = sweep_below(ps, edges, walk, probes)
    var parent: List<?Int> = [None for _ in 0..nf]
    for i in 0..probed.len: parent[probed[i]] = below[queries.len + i].0
    var verts: List<(Int, Int)> = [(p.x, p.y) for p in ps]
    verts.sort()
    var upright: List<(Int, Int, Int)> = [
        (
            ps[u].x,
            min(ps[u].y, ps[v].y),
            max(ps[u].y, ps[v].y),
        ) for (u, v) in edges if ps[u].x == ps[v].x
    ]
    upright.sort()
    var out: List<?Int> = []
    for i in 0..queries.len:
        let q = queries[i]
        let vi = verts.first_ge((q.x, q.y)) ?? verts.len
        let ui = (0..upright.len).first_true(mid => (upright[mid].0, upright[mid].1) > (q.x, q.y)) ?? upright.len
        let on_vertex = vi < verts.len and verts[vi] == (q.x, q.y)
        let on_upright = ui > 0 and upright[ui - 1].0 == q.x and q.y <= upright[ui - 1].2
        if on_vertex or on_upright or below[i].1:
            out.push(None)
            continue
        var f = below[i].0
        while f != None and outer[f!]: f = parent[f!]
        out.push(f)
    out

fn sweep_below(
    ps: List<Pt>,
    edges: List<(Int, Int)>,
    walk: PlanarWalk,
    probes: List<Pt>,
) -> List<(?Int, Bool)>:
    with lint(unproven_list_index: allow)
    var segs: List<(Int, Int)> = []
    for (u, v) in edges:
        if ps[u].x < ps[v].x: segs.push((u, v))
        elif ps[v].x < ps[u].x: segs.push((v, u))
    let m = segs.len
    var by_start = [i for i in 0..m]
    by_start.sort(by: i => ps[segs[i].0].x)
    var by_end = [i for i in 0..m]
    by_end.sort(by: i => ps[segs[i].1].x)
    var by_x = [i for i in 0..probes.len]
    by_x.sort(by: i => probes[i].x)
    var live = List.filled(m, false)
    var active = SortedSet<SweepSeg>.new()
    var out: List<(?Int, Bool)> = [(None, false) for _ in 0..probes.len]
    var si = 0
    var ei = 0
    for qi in by_x:
        let q = probes[qi]
        while ei < m and ps[segs[by_end[ei]].1].x <= q.x:
            let e = by_end[ei]
            if live[e]:
                let _ = active.remove(SweepSeg(ps[segs[e].0], ps[segs[e].1], e))
                live[e] = false
            ei += 1
        while si < m and ps[segs[by_start[si]].0].x <= q.x:
            let e = by_start[si]
            if ps[segs[e].1].x > q.x:
                let _ = active.add(SweepSeg(ps[segs[e].0], ps[segs[e].1], e))
                live[e] = true
            si += 1
        let lo = active.last_true(s => turn(s.a, s.b, q) > 0)
        let hi = active.first_true(s => turn(s.a, s.b, q) <= 0)
        let on = hi != None and turn(hi!.a, hi!.b, q) == 0
        out[qi] = (if lo == None: None else: Some(walk.side(segs[lo!.id].0, segs[lo!.id].1)), on)
    out

/// The Delaunay triangulation of distinct integer points (`|coords| <= 1e9`), exact in `I128`,
/// as counter-clockwise handle triples.
///
/// Where four or more points are cocircular the triangulation is not unique; the algorithm picks
/// one of the valid ones. Fewer than 3 points, or all collinear, give `[]`.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: `[]`
/// order: each triple rotated to start at its least handle in key order; triples sorted; a new `List`
/// traps: when two points are equal
/// example: `delaunay([Pt(0, 0), Pt(4, 0), Pt(0, 4)]) == [(0, 1, 2)]`
/// example: `delaunay([Pt(0, 0), Pt(4, 0), Pt(0, 4), Pt(1, 1)]) == [(0, 1, 3), (0, 3, 2), (1, 2, 3)]`
/// example: `delaunay([Pt(0, 0), Pt(1, 1), Pt(2, 2)]) == []`
pub fn delaunay<D>(pts: [D]Pt) -> List<(D, D, D)>:
    with lint(unproven_list_index: allow)
    let keys = pts.keys.collect()
    [(keys[a], keys[b], keys[c]) for (a, b, c) in delaunay_idx([pts[k] for k in keys])]

fn delaunay_idx(ps: List<Pt>) -> List<(Int, Int, Int)>:
    with lint(unproven_list_index: allow)
    let n = ps.len
    var order = [i for i in 0..n]
    order.sort(by: i => (ps[i].x, ps[i].y))
    let s = [ps[i] for i in order]
    for i in 1..n: assert s[i] != s[i - 1], "`delaunay` needs distinct points"
    if n < 3 or (2..n).all(i => cross128_3(s[0], s[1], s[i]) == I128(0)): return []
    var qs = DelQuads([], [], [], [], s)
    var e = qs.rec(0, n).0
    while qs.turn(qs.dest(qs.o[e]), qs.dest(e), qs.p[e]) < 0: e = qs.o[e]
    var queue = [e]
    let _ = qs.face(e, mut queue)
    var out: List<(Int, Int, Int)> = []
    var qi = 0
    while qi < queue.len:
        e = queue[qi]
        qi += 1
        if qs.mark[e]: continue
        let f = qs.face(e, mut queue)
        let (a, b, c) = (order[f[0]], order[f[1]], order[f[2]])
        out.push(if a < b and a < c: (a, b, c) else: if b < c: (b, c, a) else: (c, a, b))
    out.sort()
    out

struct DelQuads:
    rot: List<Int>
    o: List<Int>
    p: List<Int>
    mark: List<Bool>
    s: List<Pt>

    fn r(self, e: Int) -> Int:
        with lint(unproven_list_index: allow)
        return rot[rot[e]]
    fn dest(self, e: Int) -> Int:
        with lint(unproven_list_index: allow)
        return p[rot[rot[e]]]
    fn prev(self, e: Int) -> Int:
        with lint(unproven_list_index: allow)
        return rot[o[rot[e]]]
    fn next(self, e: Int) -> Int = self.prev(self.r(e))
    fn turn(self, a: Int, b: Int, c: Int) -> Int:
        with lint(unproven_list_index: allow)
        let k = cross128_3(s[a], s[b], s[c])
        if k > I128(0): 1 else: if k < I128(0): -1 else: 0

    fn in_circle(self, q: Int, a: Int, b: Int, c: Int) -> Bool:
        with lint(unproven_list_index: allow)
        let q2 = s[q].norm2_128
        let da = s[a].norm2_128 - q2
        let db = s[b].norm2_128 - q2
        let dc = s[c].norm2_128 - q2
        cross128_3(s[q], s[a], s[b]) * dc + cross128_3(s[q], s[b], s[c]) * da + cross128_3(
            s[q],
            s[c],
            s[a],
        ) * db > I128(0)

    fn edge(mut self, a: Int, b: Int) -> Int:
        with lint(unproven_list_index: allow)
        let e = rot.len
        for i in 0..4:
            rot.push(e + (i + 1) % 4)
            o.push(0)
            p.push(-1)
            mark.push(false)
        for i in 0..4:
            let q = e + (i + 1) % 4
            o[q] = if i % 2 == 1: q else: e + (i + 3) % 4
        p[e] = a
        p[e + 2] = b
        e

    fn splice(mut self, a: Int, b: Int):
        with lint(unproven_list_index: allow)
        let x = rot[o[a]]
        let y = rot[o[b]]
        let t = o[x]
        o[x] = o[y]
        o[y] = t
        let u = o[a]
        o[a] = o[b]
        o[b] = u

    fn connect(mut self, a: Int, b: Int) -> Int:
        let q = self.edge(self.dest(a), p[b])
        self.splice(q, self.next(a))
        self.splice(self.r(q), b)
        q

    fn cut(mut self, e: Int):
        self.splice(e, self.prev(e))
        self.splice(self.r(e), self.prev(self.r(e)))

    fn above(self, e: Int, base: Int) -> Bool = self.turn(
        self.dest(e),
        self.dest(base),
        p[base],
    ) > 0

    fn rec(mut self, lo: Int, hi: Int) -> (Int, Int):
        let m = hi - lo
        if m <= 3:
            let a = self.edge(lo, lo + 1)
            let b = self.edge(lo + 1, hi - 1)
            if m == 2: return (a, self.r(a))
            self.splice(self.r(a), b)
            let side = self.turn(lo, lo + 1, lo + 2)
            if side == 0: return (a, self.r(b))
            let c = self.connect(b, a)
            return if side < 0: (self.r(c), c) else: (a, self.r(b))
        let half = m / 2
        var (ra, a) = self.rec(lo, hi - half)
        var (b, rb) = self.rec(hi - half, hi)
        while true:
            if self.turn(p[b], self.dest(a), p[a]) < 0: a = self.next(a)
            elif self.turn(p[a], self.dest(b), p[b]) > 0: b = o[self.r(b)]
            else: break
        var base = self.connect(self.r(b), a)
        if p[a] == p[ra]: ra = self.r(base)
        if p[b] == p[rb]: rb = base
        while true:
            var lc = o[self.r(base)]
            if self.above(lc, base):
                while self.in_circle(self.dest(o[lc]), self.dest(base), p[base], self.dest(lc)):
                    let t = o[lc]
                    self.cut(lc)
                    lc = t
            var rc = self.prev(base)
            if self.above(rc, base):
                while self.in_circle(
                    self.dest(self.prev(rc)),
                    self.dest(base),
                    p[base],
                    self.dest(rc),
                ):
                    let t = self.prev(rc)
                    self.cut(rc)
                    rc = t
            let vl = self.above(lc, base)
            let vr = self.above(rc, base)
            if not vl and not vr: break
            if not vl or (vr and self.in_circle(self.dest(rc), p[rc], self.dest(lc), p[lc])):
                base = self.connect(rc, self.r(base))
            else:
                base = self.connect(self.r(base), self.r(lc))
        (ra, rb)

    fn face(mut self, e: Int, mut queue: List<Int>) -> List<Int>:
        with lint(unproven_list_index: allow)
        var f: List<Int> = []
        var c = e
        while true:
            mark[c] = true
            f.push(p[c])
            queue.push(self.r(c))
            c = self.next(c)
            if c == e: break
        f

/// The Voronoi cell of each point, clipped to the box `[-bound, bound]^2`, as a counter-clockwise
/// polygon (from the Delaunay dual, then half-plane clipping).
///
/// The result keeps the domain of `pts`. A cell that misses the box is `[]`; a single point's
/// cell is the whole box.
///
/// cost: \(O(n \log n)\) time, \(O(n)\) memory
/// empty: an empty array
/// order: each cell counter-clockwise, starting at its least `(y, x)` corner
/// traps: when two points are equal or `bound <= 0.0`
/// example: `voronoi_cells([Pt(-1, 0), Pt(1, 0)], 2.0)[0] == [Vec(-2.0, -2.0), Vec(0.0, -2.0), Vec(0.0, 2.0), Vec(-2.0, 2.0)]`
/// example: `voronoi_cells([Pt(-1, 0), Pt(1, 0)], 2.0)[1] == [Vec(0.0, -2.0), Vec(2.0, -2.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]`
/// example: `voronoi_cells([Pt(0, 0)], 1.0)[0] == [Vec(-1.0, -1.0), Vec(1.0, -1.0), Vec(1.0, 1.0), Vec(-1.0, 1.0)]`
pub fn voronoi_cells<D>(pts: [D]Pt, bound: F64) -> [D]List<Vec>:
    with lint(unproven_list_index: allow)
    assert bound > 0.0, "`voronoi_cells` needs `bound > 0.0`"
    let keys = D.collect()
    let cells = voronoi_idx([pts[k] for k in keys], bound)
    var out: [D]List<Vec> = []
    for i in 0..keys.len: out[keys[i]] = cells[i]
    out

fn voronoi_idx(ps: List<Pt>, bound: F64) -> List<List<Vec>>:
    with lint(unproven_list_index: allow)
    let n = ps.len
    var near: List<List<Int>> = [[] for _ in 0..n]
    let tris = delaunay_idx(ps)
    for (a, b, c) in tris:
        for (u, v) in [(a, b), (b, c), (c, a)]:
            near[u].push(v)
            near[v].push(u)
    if tris.is_empty and n > 1:
        var order = [i for i in 0..n]
        order.sort(by: i => (ps[i].x, ps[i].y))
        for i in 1..n:
            near[order[i - 1]].push(order[i])
            near[order[i]].push(order[i - 1])
    var cells: List<List<Vec>> = []
    for i in 0..n:
        let p = Vec(F64(ps[i].x), F64(ps[i].y))
        var planes: List<HalfPlane> = []
        for j in near[i]:
            let q = Vec(F64(ps[j].x), F64(ps[j].y))
            let d = q - p
            planes.push(HalfPlane((p + q) * 0.5, Vec(-d.y, d.x)))
        cells.push(half_plane_intersect(planes, bound: Some(bound)))
    cells

// ---- 3D exact & floating-point geometry

/// \(O(1)\) value type. A 3D point or vector with exact integer coordinates.
pub struct Pt3(pub x: Int, pub y: Int, pub z: Int):
    /// The squared length `x*x + y*y + z*z`, in `Int`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(1, 2, 2).norm2 == 9`
    @get pub fn norm2(self) -> Int = x * x + y * y + z * z

    /// The exact squared length, computed in `I128`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `I128`
    /// example: `Pt3(1, 2, 2).norm2_128 == 9`
    @get pub fn norm2_128(self) -> I128 = I128(x) * I128(x) + I128(y) * I128(y) + I128(z) * I128(z)

    /// The Euclidean length as `F64`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt3(1, 2, 2).len == 3.0`
    @get pub fn len(self) -> F64 = sqrt(F64(self.norm2_128))

    /// `self` as a floating-point `Vec3`: the explicit way across.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt3(1, 2, 2).to_vec3 == Vec3(1.0, 2.0, 2.0)`
    @get pub fn to_vec3(self) -> Vec3 = Vec3(F64(x), F64(y), F64(z))

    /// The dot product `x * o.x + y * o.y + z * o.z`, in `Int`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result or a product overflows `Int`
    /// example: `Pt3(1, 2, 3).dot(Pt3(4, 5, 6)) == 32`
    pub fn dot(self, o: Pt3) -> Int = x * o.x + y * o.y + z * o.z

    /// The exact dot product, computed in `I128`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the sum overflows `I128`
    /// example: `Pt3(1, 2, 3).dot128(Pt3(4, 5, 6)) == 32`
    pub fn dot128(
        self,
        o: Pt3,
    ) -> I128 = I128(x) * I128(o.x) + I128(y) * I128(o.y) + I128(z) * I128(o.z)

    /// The cross product `Pt3(y*o.z - z*o.y, z*o.x - x*o.z, x*o.y - y*o.x)`, perpendicular to both by
    /// the right-hand rule.
    ///
    /// cost: \(O(1)\)
    /// traps: when a component overflows `Int`
    /// example: `Pt3(1, 0, 0).cross(Pt3(0, 1, 0)) == Pt3(0, 0, 1)`
    /// example: `Pt3(1, 2, 3).cross(Pt3(4, 5, 6)) == Pt3(-3, 6, -3)`
    pub fn cross(self, o: Pt3) -> Pt3 = Pt3(y * o.z - z * o.y, z * o.x - x * o.z, x * o.y - y * o.x)

    /// The squared Euclidean distance from `self` to `o`, in `Int`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(0, 0, 0).dist2(Pt3(1, 2, 2)) == 9`
    pub fn dist2(self, o: Pt3) -> Int = (o - self).norm2

    /// The Euclidean distance from `self` to `o`, as `F64`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pt3(0, 0, 0).dist(Pt3(1, 2, 2)) == 3.0`
    pub fn dist(self, o: Pt3) -> F64 = (o - self).len

    /// The Manhattan (L1) distance `|x - o.x| + |y - o.y| + |z - o.z|`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(1, 1, 1).manhattan(Pt3(2, -1, 4)) == 6`
    pub fn manhattan(self, o: Pt3) -> Int = abs(o.x - x) + abs(o.y - y) + abs(o.z - z)

    /// The Chebyshev (L-infinity) distance `max(|x - o.x|, |y - o.y|, |z - o.z|)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(1, 1, 1).chebyshev(Pt3(2, -1, 4)) == 3`
    pub fn chebyshev(self, o: Pt3) -> Int = max(max(abs(o.x - x), abs(o.y - y)), abs(o.z - z))

impl Add for Pt3:
    fn add(a, b) = Pt3(a.x + b.x, a.y + b.y, a.z + b.z)

impl Sub for Pt3:
    fn sub(a, b) = Pt3(a.x - b.x, a.y - b.y, a.z - b.z)

impl Neg for Pt3:
    fn neg(a) = Pt3(-a.x, -a.y, -a.z)

impl Mul<Int> for Pt3:
    fn mul(a, k) = Pt3(a.x * k, a.y * k, a.z * k)

impl Show for Pt3:
    fn show(a) = "{a.x} {a.y} {a.z}"

fn face_side128(a: Pt3, b: Pt3, c: Pt3, p: Pt3) -> I128:
    let ux = I128(b.x - a.x)
    let uy = I128(b.y - a.y)
    let uz = I128(b.z - a.z)
    let vx = I128(c.x - a.x)
    let vy = I128(c.y - a.y)
    let vz = I128(c.z - a.z)
    let nx = uy * vz - uz * vy
    let ny = uz * vx - ux * vz
    let nz = ux * vy - uy * vx
    nx * I128(p.x - a.x) + ny * I128(p.y - a.y) + nz * I128(p.z - a.z)

fn col3(a: Pt3, b: Pt3, c: Pt3) -> Bool:
    let (ux, uy, uz) = (I128(b.x) - I128(a.x), I128(b.y) - I128(a.y), I128(b.z) - I128(a.z))
    let (vx, vy, vz) = (I128(c.x) - I128(a.x), I128(c.y) - I128(a.y), I128(c.z) - I128(a.z))
    uy * vz == uz * vy and uz * vx == ux * vz and ux * vy == uy * vx

/// The 3D convex hull of `pts` as triangular faces of indices, each counter-clockwise seen from
/// outside (so its normal points out), exact in `I128` (valid for `|x|, |y|, |z| <= 1e12`).
///
/// Only strict corners appear; a flat face with more than 3 corners is split as a fan from its
/// least index. Of equal points the smallest index stands for them. Fewer than 4 non-coplanar
/// points give `[]`.
///
/// cost: \(O(n^2)\) time, \(O(n)\) memory
/// empty: `[]`
/// order: each face rotated to start at its least index; faces sorted; a new `List`, keyed from 0
/// traps: never
/// example: `convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]`
/// example: `convex_hull_3d([Pt3(0, 0, 0), Pt3(4, 0, 0), Pt3(0, 4, 0), Pt3(0, 0, 4), Pt3(1, 1, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]`
/// example: `convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0)]) == []`
pub fn convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)>:
    let n = pts.len
    var order = [i for i in 0..n]
    order.sort(by: i => (pts[i].x, pts[i].y, pts[i].z, i))
    var ids: List<Int> = [order[t] for t in 0..n if t == 0 or pts[order[t]] != pts[order[t - 1]]]
    ids.sort()
    if ids.len < 4: return []
    let i0 = ids[0]
    let i1 = ids[1]
    let i2 = first(i for i in ids if i != i0 and i != i1 and not col3(pts[i0], pts[i1], pts[i])) ?? -1
    if i2 == -1: return []
    let i3 = first(i for i in ids if face_side128(pts[i0], pts[i1], pts[i2], pts[i]) != I128(0)) ?? -1
    if i3 == -1: return []
    var faces: List<(Int, Int, Int)> = if face_side128(
        pts[i0],
        pts[i1],
        pts[i2],
        pts[i3],
    ) > I128(0):
        [(i0, i2, i1), (i0, i1, i3), (i1, i2, i3), (i2, i0, i3)]
    else:
        [(i0, i1, i2), (i0, i3, i1), (i1, i3, i2), (i2, i3, i0)]
    var bucket: List<List<Int>> = [[] for _ in 0..n]
    for i in ids:
        if i == i0 or i == i1 or i == i2 or i == i3: continue
        var vis = [face_side128(pts[f.0], pts[f.1], pts[f.2], pts[i]) > I128(0) for f in faces]
        if not vis.any(v => v): continue
        var touched: List<Int> = []
        for k in 0..faces.len:
            if vis[k]:
                let (a, b, c) = faces[k]
                for (u, v) in [(a, b), (b, c), (c, a)]:
                    bucket[u].push(v)
                    touched.push(u)
        var kept: List<(Int, Int, Int)> = [faces[k] for k in 0..faces.len if not vis[k]]
        for k in 0..faces.len:
            if vis[k]:
                let (a, b, c) = faces[k]
                for (u, v) in [(a, b), (b, c), (c, a)]:
                    if not bucket[v].any(w => w == u): kept.push((u, v, i))
        for u in touched: bucket[u].clear()
        faces = kept
    hull3_facets(pts, faces)

fn hull3_facets(pts: List<Pt3>, faces: List<(Int, Int, Int)>) -> List<(Int, Int, Int)>:
    let n = pts.len
    let m = faces.len
    var by_edge: List<List<(v: Int, f: Int)>> = [[] for _ in 0..n]
    for k in 0..m:
        let (a, b, c) = faces[k]
        for (u, v) in [(a, b), (b, c), (c, a)]: by_edge[u].push((v:, f: k))
    var up = [k for k in 0..m]
    fn find(mut up: List<Int>, x: Int) -> Int:
        var r = x
        while up[r] != r: r = up[r]
        var y = x
        while up[y] != r:
            let nx = up[y]
            up[y] = r
            y = nx
        r
    for k in 0..m:
        let (a, b, c) = faces[k]
        for (u, v) in [(a, b), (b, c), (c, a)]:
            for e in by_edge[v]:
                if e.v == u:
                    let g = faces[e.f]
                    if face_side128(pts[a], pts[b], pts[c], pts[g.0]) == I128(0) and face_side128(
                        pts[a],
                        pts[b],
                        pts[c],
                        pts[g.1],
                    ) == I128(0) and face_side128(
                        pts[a],
                        pts[b],
                        pts[c],
                        pts[g.2],
                    ) == I128(0):
                        let ra = find(mut up, k)
                        let rb = find(mut up, e.f)
                        if ra != rb: up[ra] = rb
    var members: List<List<Int>> = [[] for _ in 0..m]
    for k in 0..m:
        let (a, b, c) = faces[k]
        let r = find(mut up, k)
        members[r].push(a)
        members[r].push(b)
        members[r].push(c)
    var out: List<(Int, Int, Int)> = []
    for k in 0..m:
        if members[k].is_empty: continue
        let (a, b, c) = faces[k]
        let pa = pts[a]
        let pb = pts[b]
        let pc = pts[c]
        let ux = I128(pb.x - pa.x)
        let uy = I128(pb.y - pa.y)
        let uz = I128(pb.z - pa.z)
        let vx = I128(pc.x - pa.x)
        let vy = I128(pc.y - pa.y)
        let vz = I128(pc.z - pa.z)
        let nx = uy * vz - uz * vy
        let ny = uz * vx - ux * vz
        let nz = ux * vy - uy * vx
        let ax = abs(nx)
        let ay = abs(ny)
        let az = abs(nz)
        let axis = if az >= ax and az >= ay: 2 else: if ax >= ay: 0 else: 1
        let pos = if axis == 0: nx > I128(0) else: if axis == 1: ny > I128(0) else: nz > I128(0)
        var vs = copy(members[k])
        vs.sort()
        var uniq: List<Int> = []
        for v in vs:
            if uniq.is_empty or uniq.last! != v: uniq.push(v)
        let proj = (v: Int) => if axis == 0: Pt(pts[v].y, pts[v].z) else: if axis == 1: Pt(
            pts[v].z,
            pts[v].x,
        ) else: Pt(
            pts[v].x,
            pts[v].y,
        )
        var hull = hull2_strict(uniq, proj)
        if not pos: hull = hull.reversed()
        var s = 0
        for i in 1..hull.len:
            if hull[i] < hull[s]: s = i
        let ring = [hull[(s + i) % hull.len] for i in 0..hull.len]
        for i in 1..ring.len - 1: out.push((ring[0], ring[i], ring[i + 1]))
    out.sort()
    out

fn hull2_strict(vs: List<Int>, proj: (Int) -> Pt) -> List<Int>:
    var order = copy(vs)
    order.sort(by: v => (proj(v).x, proj(v).y))
    var h: List<Int> = []
    for pass in 0..2:
        let start = h.len
        for t in 0..order.len:
            let v = if pass == 0: order[t] else: order[order.len - 1 - t]
            while h.len >= start + 2 and cross128(
                proj(h.last!) - proj(h[h.len - 2]),
                proj(v) - proj(h[h.len - 2]),
            ) <= I128(0): h.pop()
            h.push(v)
        h.pop()
    h

/// `convex_hull_3d` over a domain-indexed array, with faces of handles; the same orientation,
/// order and tie rules, in key order.
///
/// cost: \(O(n^2)\) time, \(O(n)\) memory
/// empty: `[]`
/// order: each face rotated to start at its least handle; faces sorted; a new `List`
/// traps: never
/// example: `convex_hull_3d_idx([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]`
pub fn convex_hull_3d_idx<D>(pts: [D]Pt3) -> List<(D, D, D)>:
    let keys = D.collect()
    [(keys[a], keys[b], keys[c]) for (a, b, c) in convex_hull_3d([pts[k] for k in keys])]

/// Six times the signed volume of the closed triangulated surface `faces` over `pts`: the sum of
/// `pts[a].dot128(pts[b].cross(pts[c]))` over faces, computed in `I128`.
///
/// Positive when faces are counter-clockwise seen from outside, as `convex_hull_3d` returns them.
///
/// cost: \(O(\lvert \mathit{faces} \rvert)\)
/// empty: 0
/// traps: when a face index is out of range
/// example: `polyhedron_volume6([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)], [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]) == 1`
/// example: `polyhedron_volume6([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)], [(0, 3, 1), (0, 1, 2), (0, 2, 3), (1, 3, 2)]) == -1`
pub fn polyhedron_volume6(pts: List<Pt3>, faces: List<(Int, Int, Int)>) -> I128:
    var v = I128(0)
    for (a, b, c) in faces:
        let p = pts[a]
        let q = pts[b]
        let r = pts[c]
        v += I128(p.x) * (I128(q.y) * I128(r.z) - I128(q.z) * I128(r.y)) + I128(p.y) * (
            I128(q.z) * I128(r.x) - I128(q.x) * I128(r.z)
        ) + I128(p.z) * (
            I128(q.x) * I128(r.y) - I128(q.y) * I128(r.x)
        )
    v

/// \(O(1)\) value type. A 3D floating-point point or vector.
pub struct Vec3(pub x: F64, pub y: F64, pub z: F64):
    /// The squared length `x*x + y*y + z*z`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 2.0).norm2 == 9.0`
    @get pub fn norm2(self) -> F64 = x * x + y * y + z * z

    /// The Euclidean length.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 2.0).len == 3.0`
    @get pub fn len(self) -> F64 = sqrt(x * x + y * y + z * z)

    /// The unit vector in the direction of `self`: each coordinate divided by `self.len`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `self.len == 0.0`
    /// example: `Vec3(0.0, 0.0, 2.0).unit == Vec3(0.0, 0.0, 1.0)`
    @get pub fn unit(self) -> Vec3:
        let l = self.len
        assert l != 0.0, "`unit` needs a nonzero vector"
        Vec3(x / l, y / l, z / l)

    /// The dot product `x * o.x + y * o.y + z * o.z`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 3.0).dot(Vec3(4.0, 5.0, 6.0)) == 32.0`
    pub fn dot(self, o: Vec3) -> F64 = x * o.x + y * o.y + z * o.z

    /// The cross product, by the right-hand rule, as for `Pt3.cross`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(1.0, 0.0, 0.0).cross(Vec3(0.0, 1.0, 0.0)) == Vec3(0.0, 0.0, 1.0)`
    pub fn cross(self, o: Vec3) -> Vec3 = Vec3(
        y * o.z - z * o.y,
        z * o.x - x * o.z,
        x * o.y - y * o.x,
    )

    /// The squared Euclidean distance from `self` to `o`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(0.0, 0.0, 0.0).dist2(Vec3(1.0, 2.0, 2.0)) == 9.0`
    pub fn dist2(self, o: Vec3) -> F64 = (o - self).norm2

    /// The Euclidean distance from `self` to `o`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(0.0, 0.0, 0.0).dist(Vec3(1.0, 2.0, 2.0)) == 3.0`
    pub fn dist(self, o: Vec3) -> F64 = (o - self).len

        /// `self` rotated by `rad` radians counter-clockwise about the unit vector `axis` (seen with
    /// `axis` pointing at the viewer), by Rodrigues' formula
    /// `v cos t + axis.cross(v) sin t + axis * axis.dot(v) * (1 - cos t)`.
    ///
    /// `axis` must be a unit vector; that is not checked.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 3.0).rot(Vec3(0.0, 0.0, 1.0), 0.0) == Vec3(1.0, 2.0, 3.0)`
    /// example: `let v = Vec3(1.0, 0.0, 0.0).rot(Vec3(0.0, 0.0, 1.0), PI / 2.0); close(v.x, 0.0) and close(v.y, 1.0) and close(v.z, 0.0)`
    pub fn rot(self, axis: Vec3, rad: F64) -> Vec3:
        let (c, s) = (cos(rad), sin(rad))
        self * c + axis.cross(self) * s + axis * (axis.dot(self) * (1.0 - c))

impl Add for Vec3:
    fn add(a, b) = Vec3(a.x + b.x, a.y + b.y, a.z + b.z)

impl Sub for Vec3:
    fn sub(a, b) = Vec3(a.x - b.x, a.y - b.y, a.z - b.z)

impl Neg for Vec3:
    fn neg(a) = Vec3(-a.x, -a.y, -a.z)

impl Mul<F64> for Vec3:
    fn mul(a, k) = Vec3(a.x * k, a.y * k, a.z * k)

impl Div<F64> for Vec3:
    fn div(a, k) = Vec3(a.x / k, a.y / k, a.z / k)

/// \(O(1)\) value type. A 3D plane `{ p | n.dot(p) == d }` with unit normal `n`.
pub struct Plane3(pub n: Vec3, pub d: F64):
    /// The plane through `a`, `b` and `c`: `n = (b - a).cross(c - a).unit`, `d = n.dot(a)`; `None`
    /// when `(b - a).cross(c - a).len <= eps` (collinear).
    ///
    /// The normal follows the right-hand rule, so `a -> b -> c` runs counter-clockwise seen from the
    /// side `n` points to.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Plane3.through(Vec3(0.0, 0.0, 1.0), Vec3(1.0, 0.0, 1.0), Vec3(0.0, 1.0, 1.0)) == Some(Plane3(Vec3(0.0, 0.0, 1.0), 1.0))`
    /// example: `Plane3.through(Vec3(0.0, 0.0, 0.0), Vec3(1.0, 1.0, 1.0), Vec3(2.0, 2.0, 2.0)) == None`
    pub fn through(a: Vec3, b: Vec3, c: Vec3, eps: F64 = 1e-9) -> ?Plane3:
        let m = (b - a).cross(c - a)
        if m.len <= eps: return None
        let n = m.unit
        Some(Plane3(n, n.dot(a)))

    /// The signed distance `n.dot(p) - d`: positive on the side `n` points to.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Plane3(Vec3(0.0, 0.0, 1.0), 1.0).dist(Vec3(5.0, 5.0, 3.0)) == 2.0`
    /// example: `Plane3(Vec3(0.0, 0.0, 1.0), 1.0).dist(Vec3(5.0, 5.0, 0.0)) == -1.0`
    pub fn dist(self, p: Vec3) -> F64 = n.dot(p) - d

    /// The orthogonal projection of `p` onto the plane: `p - n * self.dist(p)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Plane3(Vec3(0.0, 0.0, 1.0), 1.0).project(Vec3(5.0, 5.0, 3.0)) == Vec3(5.0, 5.0, 1.0)`
    pub fn project(self, p: Vec3) -> Vec3 = p - n * self.dist(p)

/// The point where the infinite line through `a` and `b` meets `plane`, or `None` when
/// `|plane.n.dot(b - a)| <= eps` (parallel, including lying in the plane, or `a == b`).
///
/// The point is `a + (b - a) * t` with `t = (plane.d - plane.n.dot(a)) / plane.n.dot(b - a)`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `line_plane_intersect(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 2.0), Plane3(Vec3(0.0, 0.0, 1.0), 1.0)) == Some(Vec3(0.0, 0.0, 1.0))`
/// example: `line_plane_intersect(Vec3(0.0, 0.0, 0.0), Vec3(1.0, 0.0, 0.0), Plane3(Vec3(0.0, 0.0, 1.0), 1.0)) == None`
pub fn line_plane_intersect(a: Vec3, b: Vec3, plane: Plane3, eps: F64 = 1e-9) -> ?Vec3:
    let den = plane.n.dot(b - a)
    if abs(den) <= eps: return None
    Some(a + (b - a) * ((plane.d - plane.n.dot(a)) / den))

/// The great-circle distance between the directions of `a` and `b` on a sphere of radius `r`:
/// `r * atan2(a.cross(b).len, a.dot(b))`. Lengths of `a` and `b` do not matter; 0.0 when either
/// is the zero vector.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `sphere_dist(Vec3(1.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0)) == PI / 2.0`
/// example: `sphere_dist(Vec3(1.0, 0.0, 0.0), Vec3(-3.0, 0.0, 0.0), r: 2.0) == 2.0 * PI`
pub fn sphere_dist(a: Vec3, b: Vec3, r: F64 = 1.0) -> F64 = r * atan2(a.cross(b).len, a.dot(b))

/// The dot product `a.x * b.x + a.y * b.y`, the free form of `a.dot(b)`.
///
/// cost: \(O(1)\)
/// traps: overflow when a product leaves `Int`; use `dot128` style `I128` maths for coordinates past about 3e9
/// example: `dot(Pt(1, 2), Pt(3, 4)) == 11`
pub fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y

/// The cross product `a.x * b.y - a.y * b.x`, the free form of `a.cross(b)`: positive when
/// `b` is counter-clockwise from `a`.
///
/// cost: \(O(1)\)
/// traps: overflow when a product leaves `Int`
/// example: `cross(Pt(1, 0), Pt(0, 1)) == 1 and cross(Pt(0, 1), Pt(1, 0)) == -1`
pub fn cross(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x

// ---- tests

fn dot128(a: Pt, b: Pt) -> I128 = I128(a.x) * I128(b.x) + I128(a.y) * I128(b.y)
fn cross128(a: Pt, b: Pt) -> I128 = I128(a.x) * I128(b.y) - I128(a.y) * I128(b.x)
fn dist2(a: Pt, b: Pt) -> Int = (a - b).norm2
fn dist2_128(a: Pt, b: Pt) -> I128:
    let dx = I128(a.x) - I128(b.x)
    let dy = I128(a.y) - I128(b.y)
    dx * dx + dy * dy
fn dist(a: Pt, b: Pt) -> F64 = (a - b).len
fn vdot(a: Vec, b: Vec) -> F64 = a.x * b.x + a.y * b.y
fn vcross(a: Vec, b: Vec) -> F64 = a.x * b.y - a.y * b.x
fn vdist(a: Vec, b: Vec) -> F64 = (a - b).len
fn dot3(a: Pt3, b: Pt3) -> Int = a.x * b.x + a.y * b.y + a.z * b.z
fn cross3(a: Pt3, b: Pt3) -> Pt3 = Pt3(
    a.y * b.z - a.z * b.y,
    a.z * b.x - a.x * b.z,
    a.x * b.y - a.y * b.x,
)
fn dist2_3(a: Pt3, b: Pt3) -> Int = (a - b).norm2

fn random_points(n: Int, span: Int) -> List<Pt> = [
    Pt(rand(-span, span), rand(-span, span)) for _ in 0..n
]

@test
fn vectors_traits_and_large_coords():
    let (a, b) = (Pt(3, 4), Pt(-1, 2))
    assert a + b == Pt(2, 6) and a - b == Pt(4, 2) and -a == Pt(-3, -4)
    assert a * 3 == Pt(9, 12)
    assert dot(a, b) == 5 and cross(a, b) == 10 and a.norm2 == 25 and close(a.len, 5.0)
    assert a.perp == Pt(-4, 3) and a.perp_cw == Pt(4, -3) and dot(a, a.perp) == 0
    assert Pt(6, -9).unit_dir() == Pt(2, -3) and Pt(0, 0).unit_dir() == Pt(0, 0)
    assert same_dir(Pt(2, 4), Pt(3, 6)) and not same_dir(Pt(2, 4), Pt(-2, -4))
    assert "{a}" == "3 4"
    // Ord and Map/Set support
    var pts = [Pt(2, 1), Pt(1, 5), Pt(1, 2)]
    pts.sort()
    assert pts == [Pt(1, 2), Pt(1, 5), Pt(2, 1)]
    var seen: Map<Pt, Int> = []
    seen[Pt(1, 2)] = 7
    assert seen[Pt(1, 2)] == 7
    // I128 orientation works at 1e18 without overflow
    let big = 1_000_000_000_000_000_000
    assert turn(Pt(0, 0), Pt(big, 0), Pt(big, 1)) == 1
    assert turn(Pt(0, 0), Pt(big, 0), Pt(big, -1)) == -1
    assert turn(Pt(-big, -big), Pt(0, 0), Pt(big, big)) == 0
    assert on_segment(Pt(0, 0), Pt(-big, -big), Pt(big, big), strict: true)

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
    dirs.sort(by: ByAngle)
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
    let base = Pt(10, -5)
    var shifted = [base + d for d in dirs]
    shuffle(mut shifted)
    shifted.sort(by: p => ByAngle(p - base))
    assert shifted == [base + d for d in dirs]

@test
fn segments_touching_crossing_and_strict():
    let o = Pt(0, 0)
    assert segments_meet(o, Pt(4, 4), Pt(0, 4), Pt(4, 0))
    assert segments_meet(o, Pt(4, 4), Pt(0, 4), Pt(4, 0), strict: true)
    assert segments_meet(o, Pt(2, 2), Pt(2, 2), Pt(5, 0))
    assert not segments_meet(o, Pt(2, 2), Pt(2, 2), Pt(5, 0), strict: true)
    assert segments_meet(o, Pt(4, 0), Pt(2, 0), Pt(6, 0))
    assert not segments_meet(o, Pt(4, 0), Pt(2, 0), Pt(6, 0), strict: true)
    assert not segments_meet(o, Pt(1, 0), Pt(2, 0), Pt(3, 0))
    assert not segments_meet(o, Pt(4, 4), Pt(1, 0), Pt(5, 3))
    assert segments_meet(o, o, o, o) and not segments_meet(o, o, Pt(1, 1), Pt(1, 1))
    assert on_segment(Pt(0, 0), Pt(0, 0), Pt(2, 2))
    assert not on_segment(Pt(0, 0), Pt(0, 0), Pt(2, 2), strict: true)
    assert on_segment(Pt(1, 1), Pt(0, 0), Pt(2, 2), strict: true)
    for _ in 0..200:
        let p = random_points(4, 3)
        let m = segments_meet(p[0], p[1], p[2], p[3])
        assert m == segments_meet(p[2], p[3], p[0], p[1]) and m == segments_meet(
            p[1],
            p[0],
            p[3],
            p[2],
        )
        if p[0] != p[1] and p[2] != p[3] and cross(p[1] - p[0], p[3] - p[2]) != 0:
            let den = cross(p[1] - p[0], p[3] - p[2])
            let t = cross(p[2] - p[0], p[3] - p[2])
            let u = cross(p[2] - p[0], p[1] - p[0])
            let within = if den > 0: 0 <= t <= den and 0 <= u <= den else: den <= t <= 0 and den <= u <= 0
            let strict_within = if den > 0: 0 < t < den and 0 < u < den else: den < t < 0 and den < u < 0
            assert m == within
            assert segments_meet(p[0], p[1], p[2], p[3], strict: true) == strict_within

@test
fn convex_hull_and_domain_indices():
    // empty, single, duplicates, all collinear
    assert convex_hull([]) == []
    assert convex_hull([Pt(3, 4)]) == [Pt(3, 4)]
    assert convex_hull([Pt(2, 2), Pt(2, 2)]) == [Pt(2, 2)]
    let line = [Pt(0, 0), Pt(1, 1), Pt(2, 2), Pt(3, 3)]
    assert convex_hull(line) == [Pt(0, 0), Pt(3, 3)]
    assert convex_hull(line, collinear: true) == line
    // square with interior and edge points
    let sq = [Pt(0, 0), Pt(2, 0), Pt(1, 0), Pt(2, 2), Pt(1, 1), Pt(0, 2), Pt(0, 1)]
    assert convex_hull(sq) == [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
    assert convex_hull(sq, collinear: true) == [
        Pt(0, 0),
        Pt(1, 0),
        Pt(2, 0),
        Pt(2, 2),
        Pt(0, 2),
        Pt(0, 1),
    ]
    // random stress: every input point lies inside or on the hull, all turns > 0
    for _ in 0..80:
        let pts = random_points(18, 6)
        let h = convex_hull(pts)
        if h.len >= 3:
            assert area2(h) > 0
            for i in 0..h.len:
                assert turn(h[i], h[(i + 1) % h.len], h[(i + 2) % h.len]) == 1
            for p in pts:
                assert locate(h, p) != Location.Outside
                assert locate_convex(h, p) == locate(h, p)
    // domain-indexed hull matches value hull
    domain P = 1..=12
    let arr = [P] _ => Pt(rand(-8, 8), rand(-8, 8))
    let idx = convex_hull_idx(arr)
    assert [arr[i] for i in idx] == convex_hull([arr[i] for i in P])

@test
fn minkowski_sum_and_upper_match_brute():
    for _ in 0..40:
        let a = convex_hull(random_points(10, 5))
        let b = convex_hull(random_points(10, 5))
        if a.len == 0 or b.len == 0: continue
        let got = minkowski_sum(a, b)
        let brute = convex_hull([p + q for p in a for q in b])
        assert area2(got) == area2(brute)
        assert convex_hull(got) == brute
        // upper hull Minkowski sum
        var sa = copy(a)
        var sb = copy(b)
        sa.sort()
        sb.sort()
        let ua = upper_hull(sa)
        let ub = upper_hull(sb)
        let mu = minkowski_upper(ua, ub)
        var all_sums = [p + q for p in ua for q in ub]
        all_sums.sort()
        assert mu == upper_hull(all_sums)

@test
fn area_locate_diameter_and_extreme():
    let poly = [Pt(0, 0), Pt(4, 0), Pt(5, 2), Pt(2, 4), Pt(-1, 2)]
    assert area2(poly) == 32 and area2(poly.rev) == -32
    assert area2_128(poly) == I128(32)
    for x in -2..=6:
        for y in -1..=5:
            let p = Pt(x, y)
            let turns = [turn(poly[i], poly[(i + 1) % poly.len], p) for i in 0..poly.len]
            let want = if turns.min()! > 0: Location.Inside elif turns.min()! == 0: Location.Boundary else: Location.Outside
            assert locate(poly, p) == want
            assert locate(poly.rev, p) == want
            assert locate_convex(poly, p) == want
    // concave L shape
    let l = [Pt(0, 0), Pt(4, 0), Pt(4, 2), Pt(2, 2), Pt(2, 4), Pt(0, 4)]
    assert area2(l) == 24
    assert locate(l, Pt(1, 3)) == Location.Inside and locate(
        l,
        Pt(3, 3),
    ) == Location.Outside and locate(
        l,
        Pt(2, 3),
    ) == Location.Boundary
    assert locate(l, Pt(3, 1)) == Location.Inside and locate(
        l,
        Pt(5, 2),
    ) == Location.Outside and locate(
        l,
        Pt(-1, 0),
    ) == Location.Outside
    // diameter2 and hull_extreme vs brute force on random hulls
    for _ in 0..60:
        let pts = random_points(20, 10)
        let h = convex_hull(pts)
        var want_d2 = 0
        for i in 0..pts.len:
            for j in i + 1..pts.len:
                want_d2 max= dist2(pts[i], pts[j])
        assert diameter2(h) == want_d2
        if h.len > 0:
            for _ in 0..10:
                let dir = Pt(rand(-5, 5), rand(-5, 5))
                let idx = hull_extreme(h, dir)
                let best_dot = h.map(p => dot128(p, dir)).max()!
                assert dot128(h[idx], dir) == best_dot

@test
fn closest_pair_matches_brute():
    for _ in 0..50:
        let n = rand(2, 30)
        let pts = random_points(n, 20)
        var want = Int.MAX
        for i in 0..n:
            for j in i + 1..n:
                want min= dist2(pts[i], pts[j])
        assert closest_pair2(pts) == want
    // domain-indexed closest pair
    domain P = 1..=25
    let arr = [P] _ => Pt(rand(-15, 15), rand(-15, 15))
    let res = closest_pair(arr)
    assert res.a != res.b and dist2(arr[res.a], arr[res.b]) == res.dist2
    var want = Int.MAX
    for i in P:
        for j in P.after(i):
            want min= dist2(arr[i], arr[j])
    assert res.dist2 == want

@test
fn float_vec_circles_and_half_planes():
    let a = Vec(0.0, 0.0)
    let b = Vec(4.0, 0.0)
    let p = Vec(1.0, 3.0)
    let proj = project(p, a, b)
    assert close(proj.x, 1.0) and close(proj.y, 0.0)
    let r = reflect(p, a, b)
    assert close(r.x, 1.0) and close(r.y, -3.0)
    let r2 = reflect(r, a, b)
    assert close(r2.x, p.x) and close(r2.y, p.y)
    assert close(line_dist(p, a, b), 3.0)
    assert close(seg_dist(Vec(6.0, 0.0), a, b), 2.0)
    // line intersection
    let inter = line_intersect(Vec(0.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0), Vec(2.0, 0.0))!
    assert close(inter.x, 1.0) and close(inter.y, 1.0)
    assert line_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == None
    // circumcenter equidistant from all 3 vertices
    let cc = circumcenter(Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(0.0, 3.0))!
    assert close(cc.x, 2.0) and close(cc.y, 1.5)
    assert close(vdist(cc, Vec(0.0, 0.0)), vdist(cc, Vec(4.0, 0.0)))
    assert close(vdist(cc, Vec(0.0, 0.0)), vdist(cc, Vec(0.0, 3.0)))
    // circle-line and circle-circle intersections
    let cl = circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 3.0), Vec(10.0, 3.0))
    assert cl.len == 2
    for q in cl:
        assert close(q.len, 5.0) and close(q.y, 3.0)
    let cc_pts = circle_intersect(Circle(Vec(0.0, 0.0), 5.0), Circle(Vec(6.0, 0.0), 5.0))
    assert cc_pts.len == 2
    for q in cc_pts:
        assert close(vdist(q, Vec(0.0, 0.0)), 5.0) and close(vdist(q, Vec(6.0, 0.0)), 5.0)
    // half-plane intersection: unit square [0, 2] x [0, 2] cut by x + y <= 3
    let planes = [
        HalfPlane(Vec(0.0, 0.0), Vec(1.0, 0.0)),
        HalfPlane(Vec(2.0, 0.0), Vec(0.0, 1.0)),
        HalfPlane(Vec(2.0, 2.0), Vec(-1.0, 0.0)),
        HalfPlane(Vec(0.0, 2.0), Vec(0.0, -1.0)),
        HalfPlane(Vec(3.0, 0.0), Vec(-1.0, 1.0)),
    ]
    let poly = half_plane_intersect(planes)
    assert poly.len == 5
    let area = 0.0 + sum(vcross(poly[i], poly[(i + 1) % poly.len]) for i in 0..poly.len)
    assert close(area * 0.5, 3.5)
    // empty intersection
    let empty_hp = [
        HalfPlane(Vec(0.0, 0.0), Vec(1.0, 0.0)),
        HalfPlane(Vec(0.0, -1.0), Vec(-1.0, 0.0)),
        HalfPlane(Vec(0.0, 0.0), Vec(0.0, 1.0)),
    ]
    assert half_plane_intersect(empty_hp).len == 0
    // random half-plane stress: bounding box + random cuts always satisfy every plane
    for _ in 0..40:
        var hps = [
            HalfPlane(Vec(-10.0, -10.0), Vec(1.0, 0.0)),
            HalfPlane(Vec(10.0, -10.0), Vec(0.0, 1.0)),
            HalfPlane(Vec(10.0, 10.0), Vec(-1.0, 0.0)),
            HalfPlane(Vec(-10.0, 10.0), Vec(0.0, -1.0)),
        ]
        for _ in 0..8:
            let px = F64(rand(-4, 4))
            let py = F64(rand(-4, 4))
            var dx = F64(rand(-5, 5))
            let dy = F64(rand(-5, 5))
            if dx == 0.0 and dy == 0.0: dx = 1.0
            hps.push(HalfPlane(Vec(px, py), Vec(dx, dy)))
        let res = half_plane_intersect(hps)
        if res.len >= 3:
            for v in res:
                for hp in hps:
                    assert vcross(hp.dir, v - hp.p) >= -1e-6 * hp.dir.len
            let a2 = 0.0 + sum(vcross(res[i], res[(i + 1) % res.len]) for i in 0..res.len)
            assert a2 >= -1e-6

@test
fn hull_extreme_and_locate_convex_edge_cases():
    // degenerate 0/1/2-point hulls
    assert locate_convex([], Pt(0, 0)) == Location.Outside
    assert locate_convex([Pt(1, 2)], Pt(1, 2)) == Location.Boundary
    assert locate_convex([Pt(1, 2)], Pt(1, 3)) == Location.Outside
    assert locate_convex([Pt(0, 0), Pt(4, 0)], Pt(2, 0)) == Location.Boundary
    assert locate_convex([Pt(0, 0), Pt(4, 0)], Pt(2, 1)) == Location.Outside
    let r90 = Vec(3.0, 4.0).rot(by: Vec(0.0, 1.0))
    assert close(r90.x, -4.0) and close(r90.y, 3.0)
    // hull_extreme on collinear: true hulls
    for _ in 0..40:
        let pts = random_points(20, 4)
        let h = convex_hull(pts, collinear: true)
        if h.len > 0:
            for _ in 0..10:
                let dir = Pt(rand(-4, 4), rand(-4, 4))
                let idx = hull_extreme(h, dir)
                let best_dot = h.map(p => dot128(p, dir)).max()!
                assert dot128(h[idx], dir) == best_dot

@test
fn hull_tangents_matches_brute():
    for _ in 0..50:
        let h = convex_hull(random_points(24, 8))
        if h.len < 3: continue
        for _ in 0..15:
            let p = Pt(rand(-15, 15), rand(-15, 15))
            if locate_convex(h, p) != Location.Outside: continue
            let t = hull_tangents(h, p)
            for q in h:
                assert turn(p, h[t.left], q) <= 0
                assert turn(p, h[t.right], q) >= 0

@test
fn polygon_area_and_cut():
    let sq = [Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(4.0, 4.0), Vec(0.0, 4.0)]
    assert close(polygon_area(sq), 16.0)
    // Cut square by diagonal (0, 0) -> (4, 4): keeps upper-left triangle (area 8)
    let tri = polygon_cut(sq, Vec(0.0, 0.0), Vec(4.0, 4.0))
    assert tri.len == 3 and close(polygon_area(tri), 8.0)
    // Cut again by x <= 2 (directed line (2, 0) -> (2, 4))
    let trap = polygon_cut(tri, Vec(2.0, 0.0), Vec(2.0, 4.0))
    assert close(polygon_area(trap), 6.0)
    // Cut completely away
    assert polygon_cut(sq, Vec(0.0, 5.0), Vec(4.0, 5.0)).is_empty

@test
fn min_enclosing_circle_and_circle_polygon_area():
    assert min_enclosing_circle([]).r == 0.0
    assert close(min_enclosing_circle([Vec(3.0, 4.0)]).r, 0.0)
    let sq = [Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]
    let mc = min_enclosing_circle(sq)
    assert close(mc.c.x, 1.0) and close(mc.c.y, 1.0) and close(mc.r, sqrt(2.0))
    // Randomized stress against brute-force over all pairs & triples
    for _ in 0..25:
        let pts = [Vec(F64(rand(-10, 10)), F64(rand(-10, 10))) for _ in 0..10]
        let got = min_enclosing_circle(pts)
        for p in pts:
            assert vdist(got.c, p) <= got.r + 1e-6
        var best_r = F64(1e18)
        for i in 0..pts.len:
            for j in i + 1..pts.len:
                let c2 = (pts[i] + pts[j]) * 0.5
                let r2 = vdist(c2, pts[i])
                if pts.all(q => vdist(c2, q) <= r2 + 1e-7): best_r min= r2
                for k in j + 1..pts.len:
                    if let c3 = circumcenter(pts[i], pts[j], pts[k]):
                        let r3 = vdist(c3, pts[i])
                        if pts.all(q => vdist(c3, q) <= r3 + 1e-7): best_r min= r3
        if best_r < F64(1e17):
            assert close(got.r, best_r, eps: 1e-5)
    // Circle-polygon intersection area
    let pi = 4.0 * atan2(1.0, 1.0)
    // Disk radius 0.5 strictly inside [0,2]^2 -> area = pi * 0.25
    assert close(circle_polygon_area(Circle(Vec(1.0, 1.0), 0.5), sq), pi * 0.25, eps: 1e-6)
    // Square strictly inside disk radius 5 -> area = 4.0
    assert close(circle_polygon_area(Circle(Vec(1.0, 1.0), 5.0), sq), 4.0, eps: 1e-6)
    // Quarter disk at corner (0, 0) with radius 1 -> area = pi / 4
    assert close(circle_polygon_area(Circle(Vec(0.0, 0.0), 1.0), sq), pi * 0.25, eps: 1e-6)
    // Disk centered at (1, 0) with radius 1 -> half disk = pi / 2
    assert close(circle_polygon_area(Circle(Vec(1.0, 0.0), 1.0), sq), pi * 0.5, eps: 1e-6)

@test
fn arg_exact_angular_sort_without_floats():
    let big = 1_000_000_000_000_000_000
    var pts = [Pt(big, 2), Pt(big, 1), Pt(-1, 0), Pt(0, -1), Pt(1, 0), Pt(0, 1)]
    pts.sort(by: arg)
    assert pts == [Pt(1, 0), Pt(big, 1), Pt(big, 2), Pt(0, 1), Pt(-1, 0), Pt(0, -1)]
    let base = Pt(10, 20)
    var around = [Pt(9, 20), Pt(10, 19), Pt(11, 20), Pt(10, 21)]
    around.sort(by: p => arg(p - base))
    assert around == [Pt(11, 20), Pt(10, 21), Pt(9, 20), Pt(10, 19)]

@test
fn shamos_hoey_matches_brute():
    assert any_segments_meet([]) == None
    assert any_segments_meet([(a: Pt(0, 0), b: Pt(1, 1))]) == None
    // Simple square: touches at corners (strict: false), no proper crossing (strict: true)
    let square = [
        (a: Pt(0, 0), b: Pt(2, 0)),
        (a: Pt(2, 0), b: Pt(2, 2)),
        (a: Pt(2, 2), b: Pt(0, 2)),
        (a: Pt(0, 2), b: Pt(0, 0)),
    ]
    assert any_segments_meet(square) != None
    assert any_segments_meet(square, strict: true) == None
    // Vertical-horizontal strict crossing
    let cross_vh = [(a: Pt(1, 0), b: Pt(1, 4)), (a: Pt(0, 2), b: Pt(3, 2))]
    assert any_segments_meet(cross_vh, strict: true) != None
    // Randomized stress against O(n^2) brute force for both strict modes
    for _ in 0..80:
        let m = rand(2, 16)
        var segs: List<(a: Pt, b: Pt)> = []
        for _ in 0..m:
            let a = Pt(rand(-6, 6), rand(-6, 6))
            let b = Pt(rand(-6, 6), rand(-6, 6))
            segs.push((a:, b:))
        for strict_mode in [false, true]:
            var brute = false
            for i in 0..m:
                for j in i + 1..m:
                    if segments_meet(
                        segs[i].a,
                        segs[i].b,
                        segs[j].a,
                        segs[j].b,
                        strict: strict_mode,
                    ):
                        brute = true
            let got = any_segments_meet(segs, strict: strict_mode)
            assert (got != None) == brute
            if let (i, j) = got:
                assert i != j and segments_meet(
                    segs[i].a,
                    segs[i].b,
                    segs[j].a,
                    segs[j].b,
                    strict: strict_mode,
                )

@test
fn pt3_and_convex_hull_3d():
    let (a, b) = (Pt3(1, 2, 3), Pt3(4, -5, 6))
    assert a + b == Pt3(5, -3, 9) and a - b == Pt3(-3, 7, -3) and -a == Pt3(-1, -2, -3)
    assert a * 2 == Pt3(2, 4, 6) and a.norm2 == 14 and dot3(a, b) == 12
    assert cross3(Pt3(1, 0, 0), Pt3(0, 1, 0)) == Pt3(0, 0, 1)
    assert dist2_3(Pt3(1, 2, 3), Pt3(4, 6, 3)) == 25
    assert "{a}" == "1 2 3"
    // Degenerate / coplanar
    assert convex_hull_3d([]).is_empty
    assert convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(1, 1, 0)]).is_empty
    // Tetrahedron (4 faces) and octahedron (8 faces)
    let tet = [Pt3(0, 0, 0), Pt3(2, 0, 0), Pt3(0, 2, 0), Pt3(0, 0, 2)]
    assert convex_hull_3d(tet).len == 4
    let oct = [
        Pt3(1, 0, 0),
        Pt3(-1, 0, 0),
        Pt3(0, 1, 0),
        Pt3(0, -1, 0),
        Pt3(0, 0, 1),
        Pt3(0, 0, -1),
    ]
    assert convex_hull_3d(oct).len == 8
    // Randomized 3D point clouds: check outward normals & closed manifold
    for _ in 0..20:
        let pts = [Pt3(rand(-50, 50), rand(-50, 50), rand(-50, 50)) for _ in 0..18]
        let faces = convex_hull_3d(pts)
        assert faces.len >= 4
        var edges: Map<(Int, Int), Int> = []
        for (u, v, w) in faces:
            assert not col3(pts[u], pts[v], pts[w])
            for p in pts:
                assert face_side128(pts[u], pts[v], pts[w], p) <= I128(0)
            edges[(u, v)] = (edges.get((u, v)) ?? 0) + 1
            edges[(v, w)] = (edges.get((v, w)) ?? 0) + 1
            edges[(w, u)] = (edges.get((w, u)) ?? 0) + 1
        for ((u, v), cnt) in edges:
            assert cnt == 1 and (edges.get((v, u)) ?? 0) == 1

@test
fn test_float_eps_guards():
    // Degenerate and near-degenerate segments use eps rather than bare == 0.0.
    assert close(seg_dist(Vec(3.0, 4.0), Vec(0.0, 0.0), Vec(1e-12, 0.0)), 5.0)
    // Large-radius tangency: linear eps stays valid when r^2 ~ 1e12.
    let cl = circle_line_intersect(
        Circle(Vec(0.0, 0.0), 1000000.0),
        Vec(-1.0, 1000000.0 + 1e-11),
        Vec(1.0, 1000000.0 + 1e-11),
    )
    assert cl.len == 1 and close(cl[0].y, 1000000.0, eps: 1e-6)
    let cc = circle_intersect(
        Circle(Vec(0.0, 0.0), 1000000.0),
        Circle(Vec(2000000.0 + 1e-11, 0.0), 1000000.0),
    )
    assert cc.len == 1 and close(cc[0].x, 1000000.0, eps: 1e-6)

@test
fn test_min_bounding_box_against_brute():
    assert close(min_bounding_box([]).area, 0.0)
    assert close(min_bounding_box([Pt(3, 4)]).perimeter, 0.0)
    let seg = min_bounding_box([Pt(0, 0), Pt(3, 4)])
    assert close(seg.area, 0.0) and close(seg.perimeter, 10.0)
    // Rotated unit square (vertices at (+/-1, 0), (0, +/-1)): area = 2, perim = 4*sqrt(2).
    let diamond = convex_hull([Pt(1, 0), Pt(0, 1), Pt(-1, 0), Pt(0, -1)])
    let db = min_bounding_box(diamond)
    assert close(db.area, 2.0, eps: 1e-9)
    assert close(db.perimeter, 4.0 * sqrt(2.0), eps: 1e-9)
    // Randomized hulls vs O(n^2) brute force over all edge orientations.
    for _ in 0..40:
        let pts = [Pt(rand(-1000, 1000), rand(-1000, 1000)) for _ in 0..25]
        let h = convex_hull(pts)
        let got = min_bounding_box(h)
        let n = h.len
        if n <= 2: continue
        var brute_area = 1e300
        var brute_perim = 1e300
        for i in 0..n:
            let e = h[(i + 1) % n] - h[i]
            let elen = sqrt(F64(dot128(e, e)))
            var min_d = dot128(e, h[0])
            var max_d = min_d
            var max_c = I128(0)
            for j in 0..n:
                let d = dot128(e, h[j])
                min_d min= d
                max_d max= d
                let c = cross128(e, h[j] - h[i])
                max_c max= c
            let w = F64(max_d - min_d) / elen
            let ht = F64(max_c) / elen
            brute_area min= w * ht
            brute_perim min= 2.0 * (w + ht)
        assert close(got.area, brute_area, eps: 1e-6 * max(1.0, brute_area))
        assert close(got.perimeter, brute_perim, eps: 1e-6 * max(1.0, brute_perim))

@test
fn test_circle_tangents_and_circle_circle_tangents():
    // Point inside, on, and outside circle.
    assert circle_tangents(Circle(Vec(0.0, 0.0), 5.0), Vec(1.0, 1.0)).is_empty
    let on = circle_tangents(Circle(Vec(0.0, 0.0), 5.0), Vec(3.0, 4.0))
    assert on.len == 1 and close(on[0].x, 3.0) and close(on[0].y, 4.0)
    // Randomized point-circle tangent laws.
    for _ in 0..40:
        let c = Vec(F64(rand(-50, 50)), F64(rand(-50, 50)))
        let r = F64(rand(1, 30))
        let p = Vec(F64(rand(-80, 80)), F64(rand(-80, 80)))
        let ts = circle_tangents(Circle(c, r), p)
        let d = (p - c).len
        if d < r - 1e-6: assert ts.is_empty
        elif d > r + 1e-6:
            assert ts.len == 2
            for t in ts:
                assert close((t - c).len, r, eps: 1e-6)
                assert close(vdot(t - c, t - p), 0.0, eps: 1e-5)
    // Circle-circle tangents: disjoint, externally tangent, intersecting, internally tangent, nested.
    let c1 = Vec(0.0, 0.0)
    assert circle_circle_tangents(Circle(c1, 2.0), Circle(Vec(6.0, 0.0), 2.0)).len == 2
    assert circle_circle_tangents(Circle(c1, 2.0), Circle(Vec(6.0, 0.0), 2.0), inner: true).len == 2
    assert circle_circle_tangents(Circle(c1, 2.0), Circle(Vec(4.0, 0.0), 2.0), inner: true).len == 1
    assert circle_circle_tangents(Circle(c1, 2.0), Circle(Vec(3.0, 0.0), 2.0), inner: true).is_empty
    assert circle_circle_tangents(Circle(c1, 5.0), Circle(Vec(3.0, 0.0), 2.0)).len == 1
    assert circle_circle_tangents(Circle(c1, 5.0), Circle(Vec(1.0, 0.0), 2.0)).is_empty
    // Randomized circle-circle tangent orthogonality & tangency laws.
    for _ in 0..40:
        let a = Vec(F64(rand(-40, 40)), F64(rand(-40, 40)))
        let b = Vec(F64(rand(-40, 40)), F64(rand(-40, 40)))
        let r1 = F64(rand(1, 25))
        let r2 = F64(rand(1, 25))
        for inner in [false, true]:
            let segs = circle_circle_tangents(Circle(a, r1), Circle(b, r2), inner:)
            for (p1, p2) in segs:
                assert close((p1 - a).len, r1, eps: 1e-6)
                assert close((p2 - b).len, r2, eps: 1e-6)
                let dir = p2 - p1
                assert close(vdot(p1 - a, dir), 0.0, eps: 1e-4)
                assert close(vdot(p2 - b, dir), 0.0, eps: 1e-4)

@test
fn test_circles_union_area_against_analytic_and_grid():
    let pi = 3.14159265358979323846
    assert close(circles_union_area([]), 0.0)
    // Single circle, duplicates, and nested circle.
    assert close(circles_union_area([Circle(Vec(0.0, 0.0), 2.0)]), 4.0 * pi, eps: 1e-7)
    assert close(
        circles_union_area([Circle(Vec(0.0, 0.0), 2.0), Circle(Vec(0.0, 0.0), 2.0)]),
        4.0 * pi,
        eps: 1e-7,
    )
    assert close(
        circles_union_area([Circle(Vec(0.0, 0.0), 3.0), Circle(Vec(1.0, 0.0), 1.0)]),
        9.0 * pi,
        eps: 1e-7,
    )
    // Two unit circles separated by 1: 2*pi - (2*pi/3 - sqrt(3)/2) = 4*pi/3 + sqrt(3)/2.
    let two_overlap = circles_union_area([Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(1.0, 0.0), 1.0)])
    assert close(two_overlap, 4.0 * pi / 3.0 + sqrt(3.0) * 0.5, eps: 1e-7)
    // Circle completely covered by three surrounding circles (None individually contains it).
    let covered = circles_union_area(
        [
            Circle(Vec(0.0, 0.0), 1.0),
            Circle(Vec(1.0, 0.0), 2.0),
            Circle(Vec(-0.5, 0.86602540378), 2.0),
            Circle(Vec(-0.5, -0.86602540378), 2.0),
        ]
    )
    let without_center = circles_union_area(
        [
            Circle(Vec(1.0, 0.0), 2.0),
            Circle(Vec(-0.5, 0.86602540378), 2.0),
            Circle(Vec(-0.5, -0.86602540378), 2.0),
        ]
    )
    assert close(covered, without_center, eps: 1e-6)
    // Randomized multi-circle union vs fine Riemann grid integration.
    let cs = [
        Circle(Vec(-1.2, 0.3), 1.8),
        Circle(Vec(0.9, -0.4), 1.5),
        Circle(Vec(0.1, 1.1), 1.4),
        Circle(Vec(-0.3, -1.0), 1.1),
        Circle(Vec(2.5, 1.8), 0.7),
    ]
    let exact = circles_union_area(cs)
    let lo = -3.5
    let hi = 3.5
    let steps = 160
    let step = (hi - lo) / F64(steps)
    var hits = 0
    for ix in 0..steps:
        let x = lo + (F64(ix) + 0.5) * step
        for iy in 0..steps:
            let y = lo + (F64(iy) + 0.5) * step
            var inside = false
            for ci in cs:
                let dx = x - ci.c.x
                let dy = y - ci.c.y
                if dx * dx + dy * dy <= ci.r * ci.r:
                    inside = true
                    break
            if inside: hits += 1
    let grid_area = F64(hits) * step * step
    assert close(exact, grid_area, eps: 0.05)

@test
fn planar_faces_cover_each_side_once_and_satisfy_euler():
    for _ in 0..20:
        let k = rand(1, 4)
        let n = k * k
        domain G = 0..n
        let pts = [G] g => Pt(g.ord % k, g.ord / k)
        var edges: List<(Int, Int)> = []
        for i in 0..n:
            if i % k + 1 < k and rand(0, 2) > 0: edges.push((i, i + 1))
            if i + k < n and rand(0, 2) > 0: edges.push((i + k, i))
        let faces = [
            [g.ord for g in f] for f in planar_faces(pts, [(G(u), G(v)) for (u, v) in edges])
        ]
        assert faces.map(f => f.len).sum() == 2 * edges.len
        var sides: List<(Int, Int)> = []
        for f in faces:
            assert f[0] == f.min()
            for i in 0..f.len: sides.push((f[i], f[(i + 1) % f.len]))
        sides.sort()
        var want: List<(Int, Int)> = []
        for (u, v) in edges:
            want.push((u, v))
            want.push((v, u))
        want.sort()
        assert sides == want
        var parent = [i for i in 0..k * k]
        fn root(p: List<Int>, x: Int) -> Int = if p[x] == x: x else: root(p, p[x])
        for (u, v) in edges: parent[root(parent, u)] = root(parent, v)
        var used = List.filled(k * k, false)
        for (u, v) in edges:
            used[u] = true
            used[v] = true
        let verts = used.count(true)
        let comps = [i for i in 0..k * k if used[i] and root(parent, i) == i].len
        assert faces.len == edges.len - verts + 2 * comps

@test
fn delaunay_is_a_ccw_empty_circle_triangulation_of_the_hull():
    for _ in 0..60:
        var seen: List<Pt> = []
        for _ in 0..rand(1, 30):
            let q = Pt(rand(-4, 4), rand(-4, 4))
            if not seen.contains(q): seen.push(q)
        let tris = delaunay(seen)
        let hull = convex_hull(seen)
        if hull.len < 3:
            assert tris.is_empty
            continue
        var total = 0
        var used = List.filled(seen.len, false)
        for (a, b, c) in tris:
            assert a < b and a < c
            let k = (seen[b] - seen[a]).cross128(seen[c] - seen[a])
            assert k > I128(0)
            total += area2([seen[a], seen[b], seen[c]])
            used[a] = true
            used[b] = true
            used[c] = true
            for q in seen:
                let q2 = q.norm2_128
                let inside = (seen[a] - q).cross128(seen[b] - q) * (seen[c].norm2_128 - q2) + (seen[b] - q).cross128(seen[c] - q) * (seen[a].norm2_128 - q2) + (seen[c] - q).cross128(seen[a] - q) * (seen[b].norm2_128 - q2)
                assert inside <= I128(0)
        assert total == area2(hull)
        assert used.all(u => u)
        assert tris == tris.sorted()

@test
fn voronoi_cells_tile_the_box_with_points_nearest_their_site():
    for _ in 0..40:
        var seen: List<Pt> = []
        for _ in 0..rand(1, 25):
            let q = Pt(rand(-5, 5), rand(-5, 5))
            if not seen.contains(q): seen.push(q)
        let cells = voronoi_cells(seen, 6.0)
        var total = 0.0
        for i in 0..seen.len:
            let cell = cells[i]
            assert cell.len >= 3
            var a = 0.0
            for k in 0..cell.len:
                let u = cell[k]
                let w = cell[(k + 1) % cell.len]
                a += u.x * w.y - u.y * w.x
                let own = (u - Vec(F64(seen[i].x), F64(seen[i].y))).norm2
                for q in seen: assert own <= (u - Vec(F64(q.x), F64(q.y))).norm2 + 1e-6
            assert a > 0.0
            total += a / 2.0
        assert abs(total - 144.0) < 1e-6

@test
fn point_location_planar_matches_the_least_enclosing_face():
    for _ in 0..30:
        let k = rand(2, 5)
        let n = k * k
        domain G = 0..n
        let pts = [G] g => Pt(2 * (g.ord % k), 2 * (g.ord / k))
        var edges: List<(Int, Int)> = []
        for i in 0..n:
            let (x, y) = (i % k, i / k)
            if x + 1 < k and rand(0, 2) > 0: edges.push((i, i + 1))
            if y + 1 < k and rand(0, 2) > 0: edges.push((i + k, i))
            if x + 1 < k and y + 1 < k:
                let pick = rand(0, 2)
                if pick == 1: edges.push((i, i + k + 1))
                if pick == 2: edges.push((i + 1, i + k))
        let keyed = [(G(u), G(v)) for (u, v) in edges]
        let faces = [[g.ord for g in f] for f in planar_faces(pts, keyed)]
        let qs = [Pt(rand(-1, 2 * k - 1), rand(-1, 2 * k - 1)) for _ in 0..30]
        let got = point_location_planar(pts, keyed, qs)
        let ps = [pts[G(i)] for i in 0..n]
        for qi in 0..qs.len:
            let q = qs[qi]
            var on = ps.contains(q)
            for (u, v) in edges:
                let (a, b) = (ps[u], ps[v])
                if (b - a).cross128(q - a) == I128(0) and min(a.x, b.x) <= q.x and q.x <= max(
                    a.x,
                    b.x,
                ) and min(
                    a.y,
                    b.y,
                ) <= q.y and q.y <= max(
                    a.y,
                    b.y,
                ): on = true
            var want: ?Int = None
            var best = 0
            if not on:
                for f in 0..faces.len:
                    let cyc = faces[f]
                    var area = 0
                    var inside = false
                    for i in 0..cyc.len:
                        let a = ps[cyc[i]]
                        let b = ps[cyc[(i + 1) % cyc.len]]
                        area += a.x * b.y - a.y * b.x
                        if (a.y <= q.y) != (b.y <= q.y):
                            let c = (b - a).cross128(q - a)
                            if (c > I128(0)) == (b.y > a.y): inside = not inside
                    if area > 0 and inside and (want == None or area < best):
                        want = Some(f)
                        best = area
            assert got[qi] == want
