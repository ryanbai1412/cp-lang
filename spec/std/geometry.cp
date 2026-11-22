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

fn igcd(a0: Int, b0: Int) -> Int = todo

// ---- exact integer points

/// O(1) value type. A 2D point or vector with exact integer coordinates.
pub struct Pt(x: Int, y: Int):
    /// The squared length `x*x + y*y`, in `Int`.
    ///
    /// Fits for `|x|, |y| <= 2e9`; use `norm2_128` beyond.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt(3, 4).norm2 == 25`
    /// example: `Pt(0, 0).norm2 == 0`
    pub @get fn norm2(self) -> Int = todo

    /// The exact squared length `x*x + y*y`, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `I128` (only for coordinates near `Int.MIN`)
    /// example: `Pt(3, 4).norm2_128 == 25`
    /// example: `Pt(3_000_000_000, 4_000_000_000).norm2_128 == 25_000_000_000_000_000_000`
    pub @get fn norm2_128(self) -> I128 = todo

    /// The Euclidean length `sqrt(x*x + y*y)` as `F64` (`hypot`, so no intermediate overflow).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt(3, 4).len == 5.0`
    /// example: `Pt(0, 0).len == 0.0`
    pub @get fn len(self) -> F64 = todo

    /// `self` rotated 90 degrees counter-clockwise: `Pt(-y, x)`.
    ///
    /// cost: O(1)
    /// traps: when `y` is `Int.MIN`
    /// example: `Pt(1, 2).perp == Pt(-2, 1)`
    pub @get fn perp(self) -> Pt = todo

    /// `self` rotated 90 degrees clockwise: `Pt(y, -x)`.
    ///
    /// cost: O(1)
    /// traps: when `x` is `Int.MIN`
    /// example: `Pt(1, 2).perp_cw == Pt(2, -1)`
    pub @get fn perp_cw(self) -> Pt = todo

    /// The polar angle `atan2(y, x)` in `(-pi, pi]`; 0 for the zero vector.
    ///
    /// For exact angular comparisons use `arg` or `angle_less` instead.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt(1, 0).angle == 0.0`
    /// example: `Pt(0, -1).angle < 0.0`
    /// example: `Pt(-1, 0).angle > 3.14`
    /// example: `Pt(0, 0).angle == 0.0`
    pub @get fn angle(self) -> F64 = todo

    /// The reduced integer direction `(x / g, y / g)` with `g = gcd(|x|, |y|)`; `Pt(0, 0)` for the
    /// zero vector.
    ///
    /// Two nonzero vectors point the same way exactly when their `unit_dir`s are equal, so it is a
    /// hashable direction key.
    ///
    /// cost: O(log max(|x|, |y|))
    /// traps: never
    /// example: `Pt(4, -6).unit_dir() == Pt(2, -3)`
    /// example: `Pt(0, 5).unit_dir() == Pt(0, 1)`
    /// example: `Pt(0, 0).unit_dir() == Pt(0, 0)`
    pub fn unit_dir(self) -> Pt = todo

    /// `self` as a floating-point `Vec(F64(x), F64(y))`: the explicit way across.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt(1, 2).to_vec == Vec(1.0, 2.0)`
    pub @get fn to_vec(self) -> Vec = todo

    /// The dot product `x * o.x + y * o.y`, in `Int`.
    ///
    /// cost: O(1)
    /// traps: when the result or a product overflows `Int`
    /// example: `Pt(1, 2).dot(Pt(3, 4)) == 11`
    /// example: `Pt(1, 0).dot(Pt(0, 1)) == 0`
    pub fn dot(self, o: Pt) -> Int = todo

    /// The exact dot product `x * o.x + y * o.y`, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the sum overflows `I128` (only when every coordinate is near `Int.MIN`)
    /// example: `Pt(1, 2).dot128(Pt(3, 4)) == 11`
    pub fn dot128(self, o: Pt) -> I128 = todo

    /// The 2D cross product `x * o.y - y * o.x`, in `Int`: positive when `o` is counter-clockwise
    /// from `self`, negative when clockwise, 0 when parallel.
    ///
    /// cost: O(1)
    /// traps: when the result or a product overflows `Int`
    /// example: `Pt(1, 0).cross(Pt(0, 1)) == 1`
    /// example: `Pt(1, 2).cross(Pt(3, 4)) == -2`
    /// example: `Pt(1, 2).cross(Pt(2, 4)) == 0`
    pub fn cross(self, o: Pt) -> Int = todo

    /// The exact 2D cross product `x * o.y - y * o.x`, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the difference overflows `I128` (only for coordinates near `Int.MIN`)
    /// example: `Pt(1, 2).cross128(Pt(3, 4)) == -2`
    pub fn cross128(self, o: Pt) -> I128 = todo

    /// The squared Euclidean distance from `self` to `o`, in `Int`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt(0, 0).dist2(Pt(3, 4)) == 25`
    /// example: `Pt(1, 1).dist2(Pt(1, 1)) == 0`
    pub fn dist2(self, o: Pt) -> Int = todo

    /// The exact squared Euclidean distance from `self` to `o`, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `I128` (coordinates beyond about `4e18` apart)
    /// example: `Pt(0, 0).dist2_128(Pt(3, 4)) == 25`
    pub fn dist2_128(self, o: Pt) -> I128 = todo

    /// The Euclidean distance from `self` to `o`, as `F64`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt(0, 0).dist(Pt(3, 4)) == 5.0`
    pub fn dist(self, o: Pt) -> F64 = todo

    /// The Manhattan (L1) distance `|x - o.x| + |y - o.y|`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt(1, 1).manhattan(Pt(4, -3)) == 7`
    pub fn manhattan(self, o: Pt) -> Int = todo

    /// The Chebyshev (L-infinity) distance `max(|x - o.x|, |y - o.y|)`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt(1, 1).chebyshev(Pt(4, -3)) == 4`
    pub fn chebyshev(self, o: Pt) -> Int = todo

impl Add for Pt:
    fn add(a, b) = todo

impl Sub for Pt:
    fn sub(a, b) = todo

impl Neg for Pt:
    fn neg(a) = todo

impl Mul<Int> for Pt:
    fn mul(a, k) = todo

impl Show for Pt:
    fn show(a) = todo

fn cross128_3(a: Pt, b: Pt, c: Pt) -> I128 = todo

fn dot128_3(a: Pt, b: Pt, c: Pt) -> I128 = todo

/// Which way the path `a -> b -> c` turns: 1 left (counter-clockwise), -1 right, 0 straight.
///
/// The sign of `(b - a).cross(c - b)`, computed exactly in `I128`, so it is exact for
/// coordinates up to `1e18`.
///
/// cost: O(1)
/// traps: never
/// example: `turn(Pt(0, 0), Pt(1, 0), Pt(1, 1)) == 1`
/// example: `turn(Pt(0, 0), Pt(1, 0), Pt(1, -1)) == -1`
/// example: `turn(Pt(0, 0), Pt(1, 0), Pt(2, 0)) == 0`
pub fn turn(a: Pt, b: Pt, c: Pt) -> Int = todo

/// Whether `a` and `b` are both nonzero and point the same way: parallel with a positive dot
/// product, exact in `I128`.
///
/// cost: O(1)
/// traps: never
/// example: `same_dir(Pt(1, 2), Pt(2, 4)) == true`
/// example: `same_dir(Pt(1, 2), Pt(-1, -2)) == false`
/// example: `same_dir(Pt(0, 0), Pt(0, 0)) == false`
pub fn same_dir(a: Pt, b: Pt) -> Bool = todo

// ---- angular sort

/// 0 for the zero vector, 1 for directions in `[0, pi)`, 2 for `[pi, 2*pi)`.
fn half(a: Pt) -> Int = todo

/// Whether `a` comes before `b` counter-clockwise, starting from the ray `from` (default `+x`).
///
/// The order is exact in `I128`: the zero vector comes first, then directions by angle in
/// `[0, 2*pi)` measured from `from`. Vectors in the same direction compare equal, so neither is
/// less than the other.
///
/// cost: O(1)
/// traps: when `from` is the zero vector
/// example: `angle_less(Pt(1, 0), Pt(0, 1)) == true`
/// example: `angle_less(Pt(0, -1), Pt(-1, 0)) == false`
/// example: `angle_less(Pt(0, 0), Pt(1, 0)) == true`
/// example: `angle_less(Pt(2, 0), Pt(1, 0)) == false`
/// example: `angle_less(Pt(-1, 0), Pt(1, 0), from: Pt(0, 1)) == true`
pub fn angle_less(a: Pt, b: Pt, from: Pt = Pt(1, 0)) -> Bool = todo

/// O(1) comparison key. Counter-clockwise by direction, starting from the positive x axis.
/// Exact in `I128`; vectors in the same direction compare equal, zero comes first.
pub struct ByAngle(p: Pt)

impl Ord for ByAngle:
    fn lt(a, b) = todo

/// O(1) comparison key. Counter-clockwise by direction starting from ray `base`.
/// Exact in `I128`; vectors in the same direction compare equal, zero comes first.
pub struct ByAngleFrom(base: Pt, p: Pt)

impl Ord for ByAngleFrom:
    fn lt(a, b) = todo

/// An exact, float-free angular sort key for `p`: `ByAngle(p)`, ordered as `angle_less` from `+x`.
///
/// Use `sort(mut pts, by: arg)`, or `sort(mut pts, by: p => arg(p - base))` around another
/// centre. Points in the same direction tie, so a stable sort keeps their input order.
///
/// cost: O(1)
/// traps: never
/// example: `var pts = [Pt(0, 1), Pt(-1, 0), Pt(1, 0)]; sort(mut pts, by: arg); pts == [Pt(1, 0), Pt(0, 1), Pt(-1, 0)]`
/// example: `arg(Pt(1, 1)) == arg(Pt(2, 2))`
pub fn arg(p: Pt) -> ByAngle = todo

// ---- segments

/// Whether `p` lies on the closed segment from `a` to `b`, exact in `I128`.
///
/// With `strict: true` the endpoints do not count. A degenerate segment `a == b` contains only
/// `a`, and nothing when strict.
///
/// cost: O(1)
/// traps: never
/// example: `on_segment(Pt(1, 1), Pt(0, 0), Pt(2, 2)) == true`
/// example: `on_segment(Pt(0, 0), Pt(0, 0), Pt(2, 2), strict: true) == false`
/// example: `on_segment(Pt(3, 3), Pt(0, 0), Pt(2, 2)) == false`
pub fn on_segment(p: Pt, a: Pt, b: Pt, strict: Bool = false) -> Bool = todo

/// Whether the closed segments `a b` and `c d` share a point, exact in `I128`.
///
/// With `strict: true` they must cross properly, at one point interior to both: touching at an
/// endpoint and collinear overlap do not count.
///
/// cost: O(1)
/// traps: never
/// example: `segments_meet(Pt(0, 0), Pt(2, 2), Pt(0, 2), Pt(2, 0)) == true`
/// example: `segments_meet(Pt(0, 0), Pt(1, 1), Pt(1, 1), Pt(2, 0)) == true`
/// example: `segments_meet(Pt(0, 0), Pt(1, 1), Pt(1, 1), Pt(2, 0), strict: true) == false`
/// example: `segments_meet(Pt(0, 0), Pt(1, 0), Pt(0, 1), Pt(1, 1)) == false`
pub fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool = todo

// ---- convex hull and Minkowski sum

/// The lower convex hull chain, left to right, of points sorted by `(x, y)` (Andrew's monotone
/// chain).
///
/// Equal points count once. With `collinear: true` points on the chain's straight stretches are
/// kept; otherwise only the corners. One point gives itself.
///
/// cost: O(n) time and memory
/// empty: `[]`
/// order: left to right, by `(x, y)`; a new `List`, keyed from 0
/// ties: equal points count once; with `collinear: false` the chain keeps only the two ends of a straight stretch
/// traps: when `sorted_pts` is not sorted by `(x, y)`
/// example: `lower_hull([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [Pt(0, 0), Pt(1, -1), Pt(2, 0)]`
/// example: `lower_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0)]) == [Pt(0, 0), Pt(2, 0)]`
/// example: `lower_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0)], collinear: true) == [Pt(0, 0), Pt(1, 0), Pt(2, 0)]`
pub fn lower_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt> = todo

/// The upper convex hull chain, left to right, of points sorted by `(x, y)`; the mirror of
/// `lower_hull`, with the same rules for duplicates and `collinear`.
///
/// cost: O(n) time and memory
/// empty: `[]`
/// order: left to right, by `(x, y)`; a new `List`, keyed from 0
/// ties: equal points count once; with `collinear: false` the chain keeps only the two ends of a straight stretch
/// traps: when `sorted_pts` is not sorted by `(x, y)`
/// example: `upper_hull([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [Pt(0, 0), Pt(1, 1), Pt(2, 0)]`
pub fn upper_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt> = todo

/// The convex hull of `pts`, counter-clockwise from the least `(x, y)` point.
///
/// Equal points count once. With `collinear: true` boundary points on edges are kept, each once;
/// otherwise only the corners. When all points are collinear the hull is the two ends (or, with
/// `collinear: true`, every point in order from the least); one distinct point gives `[p]`.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]`
/// order: counter-clockwise, starting at the least `(x, y)`; a new `List`, keyed from 0
/// traps: never
/// example: `convex_hull([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2), Pt(1, 1)]) == [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]`
/// example: `convex_hull([Pt(0, 0), Pt(1, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], collinear: true) == [Pt(0, 0), Pt(1, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]`
/// example: `convex_hull([Pt(2, 2), Pt(1, 1), Pt(0, 0)]) == [Pt(0, 0), Pt(2, 2)]`
/// example: `convex_hull([Pt(5, 5), Pt(5, 5)]) == [Pt(5, 5)]`
pub fn convex_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt> = todo

/// The convex hull of `pts` as domain positions `D` instead of copied points; otherwise as
/// `convex_hull`.
///
/// Of several positions with equal coordinates, the smallest in key order stands for them.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]`
/// order: counter-clockwise, starting at the least `(x, y)`; a new `List` of keys of `pts`
/// traps: never
/// example: `convex_hull_idx([Pt(1, 1), Pt(0, 0), Pt(2, 0), Pt(0, 0)]) == [1, 2, 0]`
pub fn convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D> = todo

/// The lower hull chain of `(x, y)`-sorted `pts` as domain positions, left to right; otherwise
/// as `lower_hull`, and equal points keep the smallest position.
///
/// cost: O(n) time and memory
/// empty: `[]`
/// order: left to right; a new `List` of keys of `pts`
/// ties: equal points keep the smallest position
/// traps: when `pts` is not sorted by `(x, y)` in key order
/// example: `lower_hull_idx([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [0, 1, 3]`
pub fn lower_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D> = todo

/// The upper hull chain of `(x, y)`-sorted `pts` as domain positions, left to right; otherwise
/// as `upper_hull`, and equal points keep the smallest position.
///
/// cost: O(n) time and memory
/// empty: `[]`
/// order: left to right; a new `List` of keys of `pts`
/// ties: equal points keep the smallest position
/// traps: when `pts` is not sorted by `(x, y)` in key order
/// example: `upper_hull_idx([Pt(0, 0), Pt(1, -1), Pt(1, 1), Pt(2, 0)]) == [0, 2, 3]`
pub fn upper_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D> = todo

/// O(n) memory. Incremental 2D convex hull of integer points backed by upper and lower chains.
pub struct OnlineHull:
    /// An empty online convex hull.
    ///
    /// The hull keeps strict corners only: points on an edge or inside are absorbed.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `OnlineHull.new().is_empty == true`
    pub fn new() -> OnlineHull = todo

    /// Insert `p` and report whether the hull changed (whether `p` is now a corner).
    ///
    /// A point inside, on the boundary, or equal to a corner returns false and changes nothing.
    ///
    /// cost: O(log n) amortized
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)) == true`
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.add(Pt(1, 1)) == false`
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(0, 0)) == false`
    pub fn add(mut self, p: Pt) -> Bool = todo

    /// Where `p` is relative to the current hull: `Inside`, `Boundary` or `Outside`.
    ///
    /// An empty hull has everything `Outside`; a one-point or segment hull has only its points on
    /// the `Boundary`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.locate(Pt(1, 1)) == Location.Inside`
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.locate(Pt(2, 0)) == Location.Boundary`
    /// example: `OnlineHull.new().locate(Pt(0, 0)) == Location.Outside`
    pub fn locate(self, p: Pt) -> Location = todo

    /// The hull corner maximizing `v.dot128(dir)`, or `None` when the hull is empty.
    ///
    /// Ties go to the corner that comes first in `vertices()` order.
    ///
    /// cost: O(log n)
    /// ties: the first in `vertices()` order
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.extreme(Pt(1, 1)) == Some(Pt(4, 0))`
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.extreme(Pt(0, 1)) == Some(Pt(0, 4))`
    /// example: `OnlineHull.new().extreme(Pt(1, 0)) == None`
    pub fn extreme(self, dir: Pt) -> ?Pt = todo

    /// Twice the area of the current hull; 0 with fewer than 3 non-collinear points.
    ///
    /// cost: O(1)
    /// traps: when the value overflows `Int`
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.area2 == 16`
    /// example: `OnlineHull.new().area2 == 0`
    pub @get fn area2(self) -> Int = todo

    /// The number of hull corners.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.add(Pt(1, 1)); h.len == 3`
    pub @get fn len(self) -> Int = todo

    /// Whether no point has been added since creation or the last `clear`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `OnlineHull.new().is_empty == true`
    /// example: `var h = OnlineHull.new(); h.add(Pt(1, 1)); h.is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// The current hull corners, counter-clockwise from the least `(x, y)`.
    ///
    /// cost: O(n) time and memory
    /// order: counter-clockwise, starting at the least `(x, y)`; a new `List`, keyed from 0
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 4)); h.add(Pt(4, 0)); h.add(Pt(0, 0)); h.vertices() == [Pt(0, 0), Pt(4, 0), Pt(0, 4)]`
    pub fn vertices(self) -> List<Pt> = todo

    /// The current hull corners, yielded in `vertices()` order without building a list.
    ///
    /// cost: O(n)
    /// order: counter-clockwise, starting at the least `(x, y)`
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(0, 4)); h.add(Pt(4, 0)); h.add(Pt(0, 0)); h.iter().collect() == [Pt(0, 0), Pt(4, 0), Pt(0, 4)]`
    pub fn iter(self) -> yields Pt = todo

    /// Remove every point; the hull is then empty.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `var h = OnlineHull.new(); h.add(Pt(1, 1)); h.clear(); h.is_empty`
    pub fn clear(mut self) = todo

fn min_yx_idx(poly: List<Pt>) -> Int = todo

/// The Minkowski sum `{p + q}` of two non-empty counter-clockwise convex polygons, by merging
/// edges.
///
/// The result is convex, counter-clockwise from its least `(y, x)` corner, with collinear
/// points dropped. Inputs may start at any corner.
///
/// cost: O(|a| + |b|) time and memory
/// empty: traps
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// ties: the start is the least `(y, x)`, which is unique
/// traps: when `a` or `b` is empty
/// example: `minkowski_sum([Pt(0, 0), Pt(1, 0), Pt(1, 1), Pt(0, 1)], [Pt(0, 0), Pt(1, 0), Pt(0, 1)]) == [Pt(0, 0), Pt(2, 0), Pt(2, 1), Pt(1, 2), Pt(0, 2)]`
/// example: `minkowski_sum([Pt(1, 1)], [Pt(0, 0), Pt(1, 0), Pt(0, 1)]) == [Pt(1, 1), Pt(2, 1), Pt(1, 2)]`
pub fn minkowski_sum(a: List<Pt>, b: List<Pt>) -> List<Pt> = todo

/// The upper hull of the Minkowski sum of two non-empty left-to-right upper hulls.
///
/// The result is again a left-to-right upper hull with collinear points dropped.
///
/// cost: O(|a| + |b|) time and memory
/// empty: traps
/// order: left to right; a new `List`, keyed from 0
/// ties: collinear sums are dropped, keeping the two ends
/// traps: when `a` or `b` is empty
/// example: `minkowski_upper([Pt(0, 0), Pt(1, 1), Pt(2, 0)], [Pt(0, 0), Pt(1, 0)]) == [Pt(0, 0), Pt(1, 1), Pt(2, 1), Pt(3, 0)]`
pub fn minkowski_upper(a: List<Pt>, b: List<Pt>) -> List<Pt> = todo

/// The lower hull of the Minkowski sum of two non-empty left-to-right lower hulls; the mirror of
/// `minkowski_upper`.
///
/// cost: O(|a| + |b|) time and memory
/// empty: traps
/// order: left to right; a new `List`, keyed from 0
/// ties: collinear sums are dropped, keeping the two ends
/// traps: when `a` or `b` is empty
/// example: `minkowski_lower([Pt(0, 0), Pt(1, -1), Pt(2, 0)], [Pt(0, 0), Pt(1, 0)]) == [Pt(0, 0), Pt(1, -1), Pt(2, -1), Pt(3, 0)]`
pub fn minkowski_lower(a: List<Pt>, b: List<Pt>) -> List<Pt> = todo

/// Whether two non-empty counter-clockwise convex polygons share at least one point, boundary
/// included. Exact in `I128`.
///
/// cost: O(|a| + |b|)
/// empty: traps
/// traps: when `a` or `b` is empty
/// example: `convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]) == true`
/// example: `convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(2, 2), Pt(3, 2), Pt(3, 3)]) == true`
/// example: `convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(5, 5), Pt(6, 5), Pt(6, 6)]) == false`
pub fn convex_polygons_overlap(a: List<Pt>, b: List<Pt>) -> Bool = todo

/// The intersection of two counter-clockwise convex polygons, as a counter-clockwise polygon.
///
/// Vertices within `eps` of each other merge. An intersection with area at most `eps` (disjoint,
/// or touching at a point or along an edge) is `[]`. The result starts at its least `(y, x)`
/// corner.
///
/// cost: O(|a| + |b|) time and memory
/// empty: `[]` when either input is empty
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// traps: never
/// example: `convex_polygon_intersect([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], [Vec(1.0, 1.0), Vec(3.0, 1.0), Vec(3.0, 3.0), Vec(1.0, 3.0)]) == [Vec(1.0, 1.0), Vec(2.0, 1.0), Vec(2.0, 2.0), Vec(1.0, 2.0)]`
/// example: `convex_polygon_intersect([Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0)], [Vec(5.0, 5.0), Vec(6.0, 5.0), Vec(5.0, 6.0)]) == []`
pub fn convex_polygon_intersect(
    a: List<Vec>,
    b: List<Vec>,
    eps: F64 = 1e-9,
) -> List<Vec> = todo

// ---- polygons and rotating calipers

/// Twice the signed area of the polygon `poly` (shoelace): positive when the vertices run
/// counter-clockwise, negative when clockwise; 0 for fewer than 3 vertices.
///
/// cost: O(n)
/// empty: 0
/// traps: when the sum overflows `Int`
/// example: `area2([Pt(0, 0), Pt(2, 0), Pt(0, 2)]) == 4`
/// example: `area2([Pt(0, 0), Pt(0, 2), Pt(2, 0)]) == -4`
/// example: `area2([Pt(0, 0), Pt(1, 1)]) == 0`
pub fn area2(poly: List<Pt>) -> Int = todo

/// Twice the signed area of `poly`, as `area2`, accumulated exactly in `I128`.
///
/// cost: O(n)
/// empty: 0
/// traps: when the sum overflows `I128`
/// example: `area2_128([Pt(0, 0), Pt(2, 0), Pt(0, 2)]) == 4`
pub fn area2_128(poly: List<Pt>) -> I128 = todo

/// O(1) enum. Position of a point relative to a polygon or disk.
pub enum Location = Inside | Boundary | Outside

/// Where `p` is relative to the simple polygon `poly`, in either orientation: `Inside`,
/// `Boundary` or `Outside`. Exact in `I128`.
///
/// Boundary is tested first (on any edge, including vertices); otherwise ray crossing decides.
/// An empty polygon has everything `Outside`.
///
/// cost: O(n)
/// empty: `Location.Outside`
/// traps: never
/// example: `locate([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(1, 1)) == Location.Inside`
/// example: `locate([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(4, 2)) == Location.Boundary`
/// example: `locate([Pt(0, 0), Pt(0, 4), Pt(4, 4), Pt(4, 0)], Pt(5, 1)) == Location.Outside`
pub fn locate(poly: List<Pt>, p: Pt) -> Location = todo

/// Where `p` is relative to the counter-clockwise convex polygon `hull`: `Inside`, `Boundary`
/// or `Outside`, by binary search on the fan from `hull[0]`. Exact in `I128`.
///
/// A hull of one or two points has only its points (or segment) on the `Boundary`; an empty hull
/// has everything `Outside`.
///
/// cost: O(log n)
/// empty: `Location.Outside`
/// traps: never
/// example: `locate_convex([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(2, 2)) == Location.Inside`
/// example: `locate_convex([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(0, 0)) == Location.Boundary`
/// example: `locate_convex([Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)], Pt(-1, 2)) == Location.Outside`
pub fn locate_convex(hull: List<Pt>, p: Pt) -> Location = todo

/// O(n). Squared diameter of a CCW convex polygon by rotating calipers.
pub fn diameter2(hull: List<Pt>) -> Int = todo

/// O(n). Minimum-area and minimum-perimeter enclosing rectangle of a CCW convex
/// polygon `hull` via exact-`I128` rotating calipers.
pub fn min_bounding_box(hull: List<Pt>) -> (area: F64, perimeter: F64) = todo

fn better_dir(a: Pt, b: Pt, dir: Pt) -> Bool = todo

/// O(log n). Index of a vertex of a non-empty CCW convex polygon maximizing
/// `hull[i].dot128(dir)`.
pub fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int = todo

fn tangent_cmp(p: Pt, a: Pt, b: Pt, sign: Int) -> Bool = todo

fn find_tangent(hull: List<Pt>, p: Pt, sign: Int) -> Int = todo

/// O(log n). Indices `(left, right)` of the two tangent vertices from a strictly
/// exterior point `p` to a strictly convex CCW polygon `hull` (`hull.len >= 3`).
/// Every vertex `q` satisfies `turn(p, hull[left], q) <= 0` and
/// `turn(p, hull[right], q) >= 0`.
pub fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int) = todo

// ---- closest pair

fn cp_solve(
    mut ord: List<(x: Int, y: Int, id: Int)>,
    mut buf: List<(x: Int, y: Int, id: Int)>,
    l: Int,
    r: Int,
) -> (a: Int, b: Int, dist2: Int) = todo

/// O(n log n), O(n) memory. Minimum squared Euclidean distance between any two distinct
/// points in `pts`. Requires `pts.len >= 2`.
pub fn closest_pair2(pts: List<Pt>) -> Int = todo

/// O(n log n), O(n) memory. Closest pair of distinct positions in a domain-indexed point array.
/// Requires at least 2 points.
pub fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int) = todo

/// O(n). Minimum width (distance between parallel supporting lines) of a CCW convex hull.
pub fn hull_width(hull: List<Pt>) -> F64 = todo

/// O(log n). Indices of the two edges of a CCW convex hull crossed by line `a b`, or `None`.
pub fn hull_line_intersect(hull: List<Pt>, a: Pt, b: Pt) -> ?(Int, Int) = todo

/// O(n log n), O(n) memory. Farthest pair of distinct handles in `pts` and their squared distance.
pub fn farthest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int) = todo

/// O(n log n), O(n) memory. O(n) candidate edges guaranteed to contain the Manhattan (L1) MST of `pts`.
pub fn manhattan_mst<D>(pts: [D]Pt) -> List<(u: D, v: D, w: Int)> = todo

/// O(n). Centroid (center of mass) of a non-degenerate simple polygon.
pub fn polygon_centroid(poly: List<Vec>) -> Vec = todo

/// O(n log n), O(n) memory. Kernel of a CCW simple polygon (points seeing the entire interior).
pub fn polygon_kernel(poly: List<Vec>, eps: F64 = 1e-9) -> List<Vec> = todo

/// O(n log n), O(n) memory. Triangulate a CCW simple polygon into vertex index triples.
pub fn polygon_triangulate(poly: List<Pt>) -> List<(Int, Int, Int)> = todo

/// O(n log max_coord). Number of strictly interior lattice points of a simple lattice polygon (Pick's theorem).
pub fn pick_interior(poly: List<Pt>) -> Int = todo

/// O(n log max_coord). Number of boundary lattice points of a simple lattice polygon.
pub fn pick_boundary(poly: List<Pt>) -> Int = todo

// ---- careful floating-point geometry

/// O(1). Circle constant pi (canonical home: `std.math.PI`; re-exported here).
pub const PI: F64 = 3.14159265358979323846

/// O(1). True when `|a - b| <= eps`.
pub fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool = todo

/// O(1). Tolerance-aware sign: 1 if `x > eps`, -1 if `x < -eps`, else 0.
pub fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int = todo

/// O(1) value type. A 2D floating-point point or vector. Kept distinct from `Pt` so exact
/// integer predicates never silently mix with approximate arithmetic.
pub struct Vec(x: F64, y: F64):
    /// O(1). Squared Euclidean length.
    pub @get fn norm2(self) -> F64 = todo

    /// O(1). Euclidean length.
    pub @get fn len(self) -> F64 = todo

    /// O(1). Rotated 90 degrees counter-clockwise.
    pub @get fn perp(self) -> Vec = todo

    /// O(1). Rotated 90 degrees clockwise.
    pub @get fn perp_cw(self) -> Vec = todo

    /// O(1). Polar angle in `(-pi, pi]`.
    pub @get fn angle(self) -> F64 = todo

    /// O(1). Unit vector `(cos(rad), sin(rad))`.
    pub fn from_angle(rad: F64) -> Vec = todo

    /// O(1). Unit vector in the same direction. Asserts nonzero length.
    pub @get fn unit(self) -> Vec = todo

    /// O(1). Rotate by unit direction vector `by` (complex multiply `self * by`).
    pub fn rot(self, by: Vec) -> Vec = todo

    /// O(1). Dot product of `self` and `o`.
    pub fn dot(self, o: Vec) -> F64 = todo

    /// O(1). 2D cross product of `self` and `o`.
    pub fn cross(self, o: Vec) -> F64 = todo

    /// O(1). Squared Euclidean distance from `self` to `o`.
    pub fn dist2(self, o: Vec) -> F64 = todo

    /// O(1). Euclidean distance from `self` to `o`.
    pub fn dist(self, o: Vec) -> F64 = todo

impl Add for Vec:
    fn add(a, b) = todo

impl Sub for Vec:
    fn sub(a, b) = todo

impl Neg for Vec:
    fn neg(a) = todo

impl Mul<F64> for Vec:
    fn mul(a, k) = todo

impl Div<F64> for Vec:
    fn div(a, k) = todo

/// O(1). Tolerance-aware orientation of `a -> b -> c`: 1 left, -1 right, 0 straight.
pub fn turn_eps(
    a: Vec,
    b: Vec,
    c: Vec,
    eps: F64 = 1e-9,
) -> Int = todo

/// O(1). Signed angle from `a` to `b` in `(-pi, pi]`.
pub fn angle_between(a: Vec, b: Vec) -> F64 = todo

/// O(1). Orthogonal projection of `p` onto the line through `a` and `b`.
pub fn project(p: Vec, a: Vec, b: Vec) -> Vec = todo

/// O(1). Reflection of `p` across the line through `a` and `b`.
pub fn reflect(p: Vec, a: Vec, b: Vec) -> Vec = todo

/// O(1). Perpendicular distance from `p` to the infinite line through `a` and `b`.
pub fn line_dist(p: Vec, a: Vec, b: Vec) -> F64 = todo

/// O(1). Shortest distance from `p` to the closed segment `a b`.
pub fn seg_dist(p: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> F64 = todo

/// O(1). Intersection of the infinite lines `a b` and `c d`, or `None` when parallel.
pub fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec = todo

/// O(1). Intersection of closed segments `a b` and `c d`: 0 disjoint, 1 crossing/touching,
/// 2 collinear overlap endpoints.
pub fn segment_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// O(1). First intersection of ray `(o, dir)` with closed segment `a b`, or `None`.
pub fn ray_hits_segment(o: Vec, dir: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> ?Vec = todo

/// O(1). Circumcenter of triangle `(a, b, c)`, or `None` when collinear.
pub fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec = todo

/// O(1) value type. A 2D circle with center `c` and non-negative radius `r`.
pub struct Circle(c: Vec, r: F64):
    /// O(1). Unique circumcircle through `a`, `b`, `c`, or `None` if collinear.
    pub fn through(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle = todo

    /// O(1). Disk area `pi * r^2`.
    pub @get fn area(self) -> F64 = todo

    /// O(1). Circumference `2 * pi * r`.
    pub @get fn circumference(self) -> F64 = todo

    /// O(1). Point location relative to the disk.
    pub fn locate(self, p: Vec, eps: F64 = 1e-9) -> Location = todo

/// O(1). Inscribed circle of triangle `a b c`, or `None` if collinear.
pub fn incenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle = todo

/// O(1). Intersection points (0, 1, or 2) of `circle` with the infinite line through `a` and `b`.
pub fn circle_line_intersect(circle: Circle, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// O(1). Intersection points (0, 1, or 2) of two circles `a` and `b`.
pub fn circle_intersect(a: Circle, b: Circle, eps: F64 = 1e-9) -> List<Vec> = todo

/// O(1). Tangent points (0, 1, or 2) from `p` to `circle`.
pub fn circle_tangents(circle: Circle, p: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// O(1). Common tangent segments `(p1, p2)` between circles `a` and `b`,
/// where `p1` lies on `a` and `p2` lies on `b` (outer tangents by default,
/// or inner tangents when `inner: true`).
pub fn circle_circle_tangents(
    a: Circle,
    b: Circle,
    inner: Bool = false,
    eps: F64 = 1e-9,
) -> List<(Vec, Vec)> = todo

/// O(1). Area of the intersection of two disks `a` and `b`.
pub fn circle_circle_area(a: Circle, b: Circle, eps: F64 = 1e-9) -> F64 = todo

/// O(1). Circle inversion of point `p` across `across`.
pub fn invert(p: Vec, across: Circle) -> Vec = todo

/// O(n^2 log n), O(n) memory. Total area of the union of disks via Green's theorem arc sweep.
pub fn circles_union_area(circles: List<Circle>, eps: F64 = 1e-9) -> F64 = todo

// ---- half-plane intersection

/// O(1) value type. Directed line through `p` in direction `dir`, keeping the closed left
/// half-plane `{ q | dir.cross(q - p) >= 0 }`.
pub struct HalfPlane(p: Vec, dir: Vec):
    /// O(1). Half-plane keeping the closed left side of directed line `a -> b`.
    pub fn through(a: Vec, b: Vec) -> HalfPlane = todo

    /// O(1). Half-plane keeping `{ (x, y) | a*x + b*y + c >= 0 }`.
    pub fn from_line(a: F64, b: F64, c: F64) -> HalfPlane = todo

    /// O(1). Signed orientation value `dir.cross(q - p)`; `>= 0` inside.
    pub fn side(self, q: Vec) -> F64 = todo

fn hp_out(
    h: (p: Vec, dir: Vec, ang: F64),
    pt: Vec,
    eps: F64,
) -> Bool = todo

fn hp_cross(a: (p: Vec, dir: Vec, ang: F64), b: (p: Vec, dir: Vec, ang: F64)) -> Vec = todo

/// O(n log n), O(n) memory. Counter-clockwise polygon of the bounded intersection of `planes`
/// (with optional `[-bound, bound]^2` clipping box), or `[]` when empty or unbounded.
pub fn half_plane_intersect(
    planes: List<HalfPlane>,
    bound: ?F64 = None,
    eps: F64 = 1e-9,
) -> List<Vec> = todo

/// O(n). Signed area of a floating-point polygon (positive for CCW, negative for CW).
pub fn polygon_area(poly: List<Vec>) -> F64 = todo

/// O(n), O(n) memory. Clip `poly` to the closed left half-plane of directed line `a -> b`.
pub fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// O(n) expected. Smallest enclosing circle of `pts` via deterministic Welzl's algorithm.
/// Returns `Circle(Vec(0.0, 0.0), 0.0)` for empty input.
pub fn min_enclosing_circle(pts: List<Vec>, eps: F64 = 1e-9) -> Circle = todo

fn tri_circle_area(a: Vec, b: Vec, r: F64, eps: F64) -> F64 = todo

/// O(n). Signed area of intersection of disk `circle` with polygon `poly`.
pub fn circle_polygon_area(circle: Circle, poly: List<Vec>, eps: F64 = 1e-9) -> F64 = todo

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
) -> Int = todo

fn sh_seg_above_y(a: Pt, b: Pt, cur_x: Int, y: Int, strict: Bool) -> Bool = todo

fn sh_merge(
    a: Int,
    b: Int,
    mut left_ch: List<Int>,
    mut right_ch: List<Int>,
    prio: List<Int>,
) -> Int = todo

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
) -> (Int, Int) = todo

fn sh_pop_leftmost(u: Int, mut left_ch: List<Int>, right_ch: List<Int>) -> Int = todo

fn sh_leftmost(u0: Int, left_ch: List<Int>) -> Int = todo

fn sh_rightmost(u0: Int, right_ch: List<Int>) -> Int = todo

fn sh_first_above_y(
    u0: Int,
    cur_x: Int,
    y: Int,
    strict: Bool,
    left_ch: List<Int>,
    right_ch: List<Int>,
    lo: List<Pt>,
    hi: List<Pt>,
) -> Int = todo

/// O(n log n), O(n) memory. Detect whether any two segments in `segs` intersect via
/// Shamos-Hoey sweep line with exact `I128` comparisons (valid for `|x|, |y| <= 1e12`).
/// Returns an intersecting index pair `Some((i, j))` or `None`. Set `strict: true` to
/// require a proper interior crossing.
pub fn any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int) = todo

/// O(n log n), O(n) memory. Domain-indexed Shamos-Hoey sweep; returns a pair of
/// intersecting segment handles, or `None`.
pub fn any_segments_meet_idx<D>(segs: [D](a: Pt, b: Pt), strict: Bool = false) -> ?(D, D) = todo

// ---- axis-aligned rectangles, planar subdivisions, and Delaunay / Voronoi

/// O(1) value type. Half-open integer rectangle `x * y` (accepts cut ranges per decision 124).
pub struct Rect(x: Range<Int>, y: Range<Int>):
    /// O(1). Area of the rectangle (0 if empty).
    pub @get fn area(self) -> Int = todo

    /// O(1). True when either axis range is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(1). True when `p.x` is in `x` and `p.y` is in `y`.
    pub fn contains(self, p: Pt) -> Bool = todo

    /// O(1). Non-empty intersection of two rectangles, or `None`.
    pub fn intersect(self, o: Rect) -> ?Rect = todo

/// O(n log n), O(n) memory. Total area of the union of axis-aligned integer rectangles via sweep line.
pub fn rects_union_area(rects: List<Rect>) -> Int = todo

/// O((n + m) log m), O(n + m) memory. Faces (CCW interior cycles and CW outer cycle) of a planar
/// straight-line graph.
pub fn planar_faces<V>(pts: [V]Pt, edges: List<(V, V)>) -> List<List<V>> = todo

/// O((n + m + q) log(n + m)), O(n + m + q) memory. Offline sweep-line point location in a planar
/// subdivision; returns face index or `None`.
pub fn point_location_planar<V>(
    pts: [V]Pt,
    edges: List<(V, V)>,
    queries: List<Pt>,
) -> List<?Int> = todo

/// O(n log n), O(n) memory. Exact Delaunay triangulation of distinct integer points
/// (`|coords| <= 1e9`) as CCW handle triples.
pub fn delaunay<D>(pts: [D]Pt) -> List<(D, D, D)> = todo

/// O(n log n), O(n) memory. Voronoi cells of `pts` clipped to `[-bound, bound]^2` via the Delaunay dual.
pub fn voronoi_cells<D>(pts: [D]Pt, bound: F64) -> [D]List<Vec> = todo

// ---- 3D exact & floating-point geometry

/// O(1) value type. A 3D point or vector with exact integer coordinates.
pub struct Pt3(x: Int, y: Int, z: Int):
    /// O(1). The squared Euclidean length (`x*x + y*y + z*z`).
    pub @get fn norm2(self) -> Int = todo

    /// O(1). The exact 128-bit squared Euclidean length.
    pub @get fn norm2_128(self) -> I128 = todo

    /// O(1). The Euclidean length as `F64`.
    pub @get fn len(self) -> F64 = todo

    /// O(1). Explicit conversion to 3D floating-point `Vec3`.
    pub @get fn to_vec3(self) -> Vec3 = todo

    /// O(1). 3D dot product of `self` and `o`.
    pub fn dot(self, o: Pt3) -> Int = todo

    /// O(1). Exact 128-bit 3D dot product of `self` and `o`.
    pub fn dot128(self, o: Pt3) -> I128 = todo

    /// O(1). 3D cross product of `self` and `o`.
    pub fn cross(self, o: Pt3) -> Pt3 = todo

    /// O(1). Squared 3D Euclidean distance from `self` to `o`.
    pub fn dist2(self, o: Pt3) -> Int = todo

    /// O(1). 3D Euclidean distance from `self` to `o` as `F64`.
    pub fn dist(self, o: Pt3) -> F64 = todo

    /// O(1). 3D Manhattan (L1) distance `|x - o.x| + |y - o.y| + |z - o.z|`.
    pub fn manhattan(self, o: Pt3) -> Int = todo

    /// O(1). 3D Chebyshev (L-infinity) distance `max(|x - o.x|, |y - o.y|, |z - o.z|)`.
    pub fn chebyshev(self, o: Pt3) -> Int = todo

impl Add for Pt3:
    fn add(a, b) = todo

impl Sub for Pt3:
    fn sub(a, b) = todo

impl Neg for Pt3:
    fn neg(a) = todo

impl Mul<Int> for Pt3:
    fn mul(a, k) = todo

impl Show for Pt3:
    fn show(a) = todo

fn face_side128(a: Pt3, b: Pt3, c: Pt3, p: Pt3) -> I128 = todo

fn col3(a: Pt3, b: Pt3, c: Pt3) -> Bool = todo

/// O(n^2), O(n) memory. Exact 3D convex hull of `pts` via incremental hull in `I128`
/// (valid for `|x|, |y|, |z| <= 1e12`). Returns outward-oriented triangular faces `(a, b, c)`.
pub fn convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)> = todo

/// O(n^2), O(n) memory. Domain-indexed 3D convex hull returning outward-oriented
/// triangular faces of `D` handles.
pub fn convex_hull_3d_idx<D>(pts: [D]Pt3) -> List<(D, D, D)> = todo

/// O(|faces|). Six times the signed volume of a closed triangulated polyhedron.
pub fn polyhedron_volume6(pts: List<Pt3>, faces: List<(Int, Int, Int)>) -> I128 = todo

/// O(1) value type. A 3D floating-point point or vector.
pub struct Vec3(x: F64, y: F64, z: F64):
    /// O(1). Squared Euclidean length.
    pub @get fn norm2(self) -> F64 = todo

    /// O(1). Euclidean length.
    pub @get fn len(self) -> F64 = todo

    /// O(1). Unit vector in the same direction (traps if `len == 0`).
    pub @get fn unit(self) -> Vec3 = todo

    /// O(1). 3D dot product of `self` and `o`.
    pub fn dot(self, o: Vec3) -> F64 = todo

    /// O(1). 3D cross product of `self` and `o`.
    pub fn cross(self, o: Vec3) -> Vec3 = todo

    /// O(1). Squared 3D Euclidean distance from `self` to `o`.
    pub fn dist2(self, o: Vec3) -> F64 = todo

    /// O(1). 3D Euclidean distance from `self` to `o`.
    pub fn dist(self, o: Vec3) -> F64 = todo

    /// O(1). Rodrigues rotation around unit `axis` by `rad` radians.
    pub fn rot(self, axis: Vec3, rad: F64) -> Vec3 = todo

impl Add for Vec3:
    fn add(a, b) = todo

impl Sub for Vec3:
    fn sub(a, b) = todo

impl Neg for Vec3:
    fn neg(a) = todo

impl Mul<F64> for Vec3:
    fn mul(a, k) = todo

impl Div<F64> for Vec3:
    fn div(a, k) = todo

/// O(1) value type. A 3D plane `{ p | n.dot(p) == d }` with unit normal `n`.
pub struct Plane3(n: Vec3, d: F64):
    /// O(1). Plane through non-collinear points `a`, `b`, `c`, or `None`.
    pub fn through(a: Vec3, b: Vec3, c: Vec3, eps: F64 = 1e-9) -> ?Plane3 = todo

    /// O(1). Signed distance from `p` to the plane.
    pub fn dist(self, p: Vec3) -> F64 = todo

    /// O(1). Orthogonal projection of `p` onto the plane.
    pub fn project(self, p: Vec3) -> Vec3 = todo

/// O(1). Intersection of line `a b` with `plane`, or `None` if parallel.
pub fn line_plane_intersect(a: Vec3, b: Vec3, plane: Plane3, eps: F64 = 1e-9) -> ?Vec3 = todo

/// O(1). Great-circle distance between directions `a` and `b` on a sphere of radius `r`.
pub fn sphere_dist(a: Vec3, b: Vec3, r: F64 = 1.0) -> F64 = todo

// ---- tests

fn dot(a: Pt, b: Pt) -> Int = todo
fn cross(a: Pt, b: Pt) -> Int = todo
fn dot128(a: Pt, b: Pt) -> I128 = todo
fn cross128(a: Pt, b: Pt) -> I128 = todo
fn dist2(a: Pt, b: Pt) -> Int = todo
fn dist2_128(a: Pt, b: Pt) -> I128 = todo
fn dist(a: Pt, b: Pt) -> F64 = todo
fn vdot(a: Vec, b: Vec) -> F64 = todo
fn vcross(a: Vec, b: Vec) -> F64 = todo
fn vdist(a: Vec, b: Vec) -> F64 = todo
fn dot3(a: Pt3, b: Pt3) -> Int = todo
fn cross3(a: Pt3, b: Pt3) -> Pt3 = todo
fn dist2_3(a: Pt3, b: Pt3) -> Int = todo

fn random_points(n: Int, span: Int) -> List<Pt> = todo

@test
fn vectors_traits_and_large_coords() = todo

@test
fn angle_order_goes_once_around() = todo

@test
fn segments_touching_crossing_and_strict() = todo

@test
fn convex_hull_and_domain_indices() = todo

@test
fn minkowski_sum_and_upper_match_brute() = todo

@test
fn area_locate_diameter_and_extreme() = todo

@test
fn closest_pair_matches_brute() = todo

@test
fn float_vec_circles_and_half_planes() = todo

@test
fn hull_extreme_and_locate_convex_edge_cases() = todo

@test
fn hull_tangents_matches_brute() = todo

@test
fn polygon_area_and_cut() = todo

@test
fn min_enclosing_circle_and_circle_polygon_area() = todo

@test
fn arg_exact_angular_sort_without_floats() = todo

@test
fn shamos_hoey_matches_brute() = todo

@test
fn pt3_and_convex_hull_3d() = todo

@test
fn test_float_eps_guards() = todo

@test
fn test_min_bounding_box_against_brute() = todo

@test
fn test_circle_tangents_and_circle_circle_tangents() = todo

@test
fn test_circles_union_area_against_analytic_and_grid() = todo
