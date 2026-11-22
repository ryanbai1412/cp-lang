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
    @get pub fn norm2(self) -> Int = todo

    /// The exact squared length `x*x + y*y`, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `I128` (only for coordinates near `Int.MIN`)
    /// example: `Pt(3, 4).norm2_128 == 25`
    /// example: `Pt(3_000_000_000, 4_000_000_000).norm2_128 == 25_000_000_000_000_000_000`
    @get pub fn norm2_128(self) -> I128 = todo

    /// The Euclidean length `sqrt(x*x + y*y)` as `F64` (`hypot`, so no intermediate overflow).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt(3, 4).len == 5.0`
    /// example: `Pt(0, 0).len == 0.0`
    @get pub fn len(self) -> F64 = todo

    /// `self` rotated 90 degrees counter-clockwise: `Pt(-y, x)`.
    ///
    /// cost: O(1)
    /// traps: when `y` is `Int.MIN`
    /// example: `Pt(1, 2).perp == Pt(-2, 1)`
    @get pub fn perp(self) -> Pt = todo

    /// `self` rotated 90 degrees clockwise: `Pt(y, -x)`.
    ///
    /// cost: O(1)
    /// traps: when `x` is `Int.MIN`
    /// example: `Pt(1, 2).perp_cw == Pt(2, -1)`
    @get pub fn perp_cw(self) -> Pt = todo

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
    @get pub fn angle(self) -> F64 = todo

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
    @get pub fn to_vec(self) -> Vec = todo

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
    /// example: `OnlineHull().is_empty == true`
    pub fn new() -> OnlineHull = todo

    /// Insert `p` and report whether the hull changed (whether `p` is now a corner).
    ///
    /// A point inside, on the boundary, or equal to a corner returns false and changes nothing.
    ///
    /// cost: O(log n) amortized
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)) == true`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.add(Pt(1, 1)) == false`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(0, 0)) == false`
    pub fn add(mut self, p: Pt) -> Bool = todo

    /// Where `p` is relative to the current hull: `Inside`, `Boundary` or `Outside`.
    ///
    /// An empty hull has everything `Outside`; a one-point or segment hull has only its points on
    /// the `Boundary`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.locate(Pt(1, 1)) == Location.Inside`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.locate(Pt(2, 0)) == Location.Boundary`
    /// example: `OnlineHull().locate(Pt(0, 0)) == Location.Outside`
    pub fn locate(self, p: Pt) -> Location = todo

    /// The hull corner maximizing `v.dot128(dir)`, or `None` when the hull is empty.
    ///
    /// Ties go to the corner that comes first in `vertices()` order.
    ///
    /// cost: O(log n)
    /// ties: the first in `vertices()` order
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.extreme(Pt(1, 1)) == Some(Pt(4, 0))`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.extreme(Pt(0, 1)) == Some(Pt(0, 4))`
    /// example: `OnlineHull().extreme(Pt(1, 0)) == None`
    pub fn extreme(self, dir: Pt) -> ?Pt = todo

    /// Twice the area of the current hull; 0 with fewer than 3 non-collinear points.
    ///
    /// cost: O(1)
    /// traps: when the value overflows `Int`
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.area2 == 16`
    /// example: `OnlineHull().area2 == 0`
    @get pub fn area2(self) -> Int = todo

    /// The number of hull corners.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 0)); h.add(Pt(4, 0)); h.add(Pt(0, 4)); h.add(Pt(1, 1)); h.len == 3`
    @get pub fn len(self) -> Int = todo

    /// Whether no point has been added since creation or the last `clear`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `OnlineHull().is_empty == true`
    /// example: `var h = OnlineHull(); h.add(Pt(1, 1)); h.is_empty == false`
    @get pub fn is_empty(self) -> Bool = todo

    /// The current hull corners, counter-clockwise from the least `(x, y)`.
    ///
    /// cost: O(n) time and memory
    /// order: counter-clockwise, starting at the least `(x, y)`; a new `List`, keyed from 0
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 4)); h.add(Pt(4, 0)); h.add(Pt(0, 0)); h.vertices() == [Pt(0, 0), Pt(4, 0), Pt(0, 4)]`
    pub fn vertices(self) -> List<Pt> = todo

    /// The current hull corners, yielded in `vertices()` order without building a list.
    ///
    /// cost: O(n)
    /// order: counter-clockwise, starting at the least `(x, y)`
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(0, 4)); h.add(Pt(4, 0)); h.add(Pt(0, 0)); h.iter().collect() == [Pt(0, 0), Pt(4, 0), Pt(0, 4)]`
    pub fn iter(self) -> yields Pt = todo

    /// Remove every point; the hull is then empty.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `var h = OnlineHull(); h.add(Pt(1, 1)); h.clear(); h.is_empty`
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

/// The squared diameter of the counter-clockwise convex polygon `hull`: the largest
/// `dist2` between two vertices, by rotating calipers.
///
/// cost: O(n)
/// empty: 0 (also for one point)
/// traps: when the result overflows `Int`
/// example: `diameter2([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 8`
/// example: `diameter2([Pt(0, 0), Pt(3, 4)]) == 25`
pub fn diameter2(hull: List<Pt>) -> Int = todo

/// The least area and the least perimeter of a rectangle enclosing the counter-clockwise convex
/// polygon `hull`, each minimized on its own (by rotating calipers, exact in `I128` until the
/// final division).
///
/// One side of an optimal rectangle lies along a hull edge, so only those `n` orientations are
/// tried. Fewer than 3 vertices give a degenerate rectangle (area 0, perimeter twice the length).
///
/// cost: O(n)
/// empty: `(0.0, 0.0)`
/// ties: none; both results are values, not a chosen rectangle
/// traps: never
/// example: `min_bounding_box([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == (4.0, 8.0)`
/// example: `min_bounding_box([Pt(0, 0), Pt(3, 0)]) == (0.0, 6.0)`
pub fn min_bounding_box(hull: List<Pt>) -> (area: F64, perimeter: F64) = todo

fn better_dir(a: Pt, b: Pt, dir: Pt) -> Bool = todo

/// The index of a vertex of the non-empty counter-clockwise convex polygon `hull` maximizing
/// `hull[i].dot128(dir)`, by binary search.
///
/// cost: O(log n)
/// empty: traps
/// ties: the smallest index
/// traps: when `hull` is empty
/// example: `hull_extreme([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(1, 1)) == 2`
/// example: `hull_extreme([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(1, 0)) == 1`
pub fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int = todo

fn tangent_cmp(p: Pt, a: Pt, b: Pt, sign: Int) -> Bool = todo

fn find_tangent(hull: List<Pt>, p: Pt, sign: Int) -> Int = todo

/// The indices `(left, right)` of the tangent vertices from a strictly exterior point `p` to a
/// strictly convex counter-clockwise polygon `hull` (`hull.len >= 3`).
///
/// Every vertex `q` satisfies `turn(p, hull[left], q) <= 0` and `turn(p, hull[right], q) >= 0`:
/// seen from `p`, the whole polygon lies right of the ray to `left` and left of the ray to
/// `right`.
///
/// cost: O(log n)
/// ties: when two vertices qualify (`p` on an edge's line), the one nearer to `p`
/// empty: traps (needs `hull.len >= 3`)
/// traps: when `hull.len < 3` or `p` is not strictly outside
/// example: `hull_tangents([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(4, 1)) == (1, 2)`
pub fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int) = todo

// ---- closest pair

fn cp_solve(
    mut ord: List<(x: Int, y: Int, id: Int)>,
    mut buf: List<(x: Int, y: Int, id: Int)>,
    l: Int,
    r: Int,
) -> (a: Int, b: Int, dist2: Int) = todo

/// The least squared distance between two points of `pts` at distinct positions (0 when two are
/// equal), by divide and conquer.
///
/// cost: O(n log n) time, O(n) memory
/// empty: traps
/// traps: when `pts.len < 2`, or a distance overflows `Int`
/// example: `closest_pair2([Pt(0, 0), Pt(5, 5), Pt(1, 1), Pt(9, 9)]) == 2`
/// example: `closest_pair2([Pt(3, 3), Pt(3, 3)]) == 0`
pub fn closest_pair2(pts: List<Pt>) -> Int = todo

/// The closest pair of distinct positions `a` before `b` in key order, with their squared
/// distance.
///
/// cost: O(n log n) time, O(n) memory
/// empty: traps
/// ties: the pair `(a, b)` that comes first in key order, `a` then `b`
/// traps: when `pts` has fewer than 2 points, or a distance overflows `Int`
/// example: `closest_pair([Pt(0, 0), Pt(5, 5), Pt(1, 1)]) == (0, 2, 2)`
/// example: `closest_pair([Pt(0, 0), Pt(1, 0), Pt(2, 0)]) == (0, 1, 1)`
pub fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int) = todo

/// The least width of the counter-clockwise convex polygon `hull`: the smallest distance between
/// two parallel supporting lines, by rotating calipers. Fewer than 3 vertices give 0.0.
///
/// cost: O(n)
/// empty: 0.0
/// traps: never
/// example: `hull_width([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 2.0`
/// example: `hull_width([Pt(0, 0), Pt(4, 0), Pt(4, 1), Pt(0, 1)]) == 1.0`
pub fn hull_width(hull: List<Pt>) -> F64 = todo

/// The two edges of the counter-clockwise convex polygon `hull` that the line through `a` and
/// `b` crosses, as `(i, j)` with `i < j`, or `None` if the line misses the interior.
///
/// Edge `i` runs from `hull[i]` to `hull[(i + 1) % n]`. A line that only touches the polygon (at a
/// vertex, or along an edge) returns `None`.
///
/// cost: O(log n)
/// empty: `None`
/// traps: when `a == b`
/// example: `hull_line_intersect([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(-1, 1), Pt(3, 1)) == Some((1, 3))`
/// example: `hull_line_intersect([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], Pt(-1, 5), Pt(3, 5)) == None`
pub fn hull_line_intersect(hull: List<Pt>, a: Pt, b: Pt) -> ?(Int, Int) = todo

/// The farthest pair of distinct positions `a` before `b` in key order, with their squared
/// distance (convex hull, then rotating calipers).
///
/// cost: O(n log n) time, O(n) memory
/// empty: traps
/// ties: the pair `(a, b)` that comes first in key order, `a` then `b`
/// traps: when `pts` has fewer than 2 points, or a distance overflows `Int`
/// example: `farthest_pair([Pt(0, 0), Pt(1, 1), Pt(3, 0)]) == (0, 2, 9)`
pub fn farthest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int) = todo

/// At most `4n` candidate edges `(u, v, w)` that contain a minimum spanning tree of `pts` under
/// the Manhattan distance `w = pts[u].manhattan(pts[v])`.
///
/// For each of the 8 octants of each point, the nearest point in that octant (sweep with a
/// Fenwick tree), each unordered pair once with `u` before `v` in key order. Run Kruskal on the
/// result for the MST itself.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]` (also for one point)
/// order: by `(w, u, v)` ascending; a new `List`, keyed from 0
/// traps: when a distance overflows `Int`
/// example: `manhattan_mst([Pt(0, 0), Pt(1, 1), Pt(5, 5)]).any(e => e.u == 0 and e.v == 1 and e.w == 2)`
/// example: `manhattan_mst([Pt(0, 0), Pt(2, 0)]) == [(0, 1, 2)]`
pub fn manhattan_mst<D>(pts: [D]Pt) -> List<(u: D, v: D, w: Int)> = todo

/// The centroid (centre of mass) of the region inside the simple polygon `poly`, either
/// orientation.
///
/// cost: O(n)
/// empty: traps
/// traps: when the polygon has zero area
/// example: `polygon_centroid([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]) == Vec(1.0, 1.0)`
pub fn polygon_centroid(poly: List<Vec>) -> Vec = todo

/// The kernel of the counter-clockwise simple polygon `poly`: the points that see the whole
/// interior, as a counter-clockwise convex polygon (half-plane intersection of the edges).
///
/// A kernel with area at most `eps` is `[]`. The result starts at its least `(y, x)` corner.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]`
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// traps: never
/// example: `polygon_kernel([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]) == [Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]`
pub fn polygon_kernel(poly: List<Vec>, eps: F64 = 1e-9) -> List<Vec> = todo

/// A triangulation of the counter-clockwise simple polygon `poly` into `n - 2` triangles of
/// vertex indices, each listed counter-clockwise.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]` (also for fewer than 3 vertices)
/// order: deterministic for a given input; each triple counter-clockwise
/// traps: when `poly` is not counter-clockwise
/// example: `polygon_triangulate([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]).len == 2`
/// example: `polygon_triangulate([Pt(0, 0), Pt(1, 0), Pt(0, 1)]).len == 1`
pub fn polygon_triangulate(poly: List<Pt>) -> List<(Int, Int, Int)> = todo

/// The number of lattice points strictly inside the simple lattice polygon `poly`, either
/// orientation, by Pick's theorem: `I = (|area2| - B) / 2 + 1` with `B = pick_boundary(poly)`.
///
/// cost: O(n log max_coord)
/// empty: traps
/// traps: when `poly` has fewer than 3 vertices, or the area overflows `Int`
/// example: `pick_interior([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 1`
/// example: `pick_interior([Pt(0, 0), Pt(4, 0), Pt(0, 4)]) == 3`
pub fn pick_interior(poly: List<Pt>) -> Int = todo

/// The number of lattice points on the boundary of the lattice polygon `poly`: the sum of
/// `gcd(|dx|, |dy|)` over its edges, including the closing edge.
///
/// cost: O(n log max_coord)
/// empty: 0
/// traps: when the sum overflows `Int`
/// example: `pick_boundary([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 8`
/// example: `pick_boundary([Pt(0, 0), Pt(4, 0), Pt(0, 4)]) == 12`
pub fn pick_boundary(poly: List<Pt>) -> Int = todo

// ---- careful floating-point geometry

/// O(1). Circle constant pi (canonical home: `std.math.PI`; re-exported here).
pub const PI: F64 = 3.14159265358979323846

/// Whether `|a - b| <= eps`: an absolute tolerance, the same test every `eps` in this module uses.
///
/// cost: O(1)
/// traps: never
/// example: `close(0.1 + 0.2, 0.3) == true`
/// example: `close(1.0, 1.1) == false`
/// example: `close(1.0, 1.25, eps: 0.25) == true`
pub fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool = todo

/// The sign of `x` with tolerance: 1 if `x > eps`, -1 if `x < -eps`, else 0.
///
/// cost: O(1)
/// traps: never
/// example: `sign_eps(2.0) == 1`
/// example: `sign_eps(-0.5) == -1`
/// example: `sign_eps(1e-12) == 0`
pub fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int = todo

/// O(1) value type. A 2D floating-point point or vector. Kept distinct from `Pt` so exact
/// integer predicates never silently mix with approximate arithmetic.
pub struct Vec(x: F64, y: F64):
    /// The squared length `x*x + y*y`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(3.0, 4.0).norm2 == 25.0`
    @get pub fn norm2(self) -> F64 = todo

    /// The Euclidean length (`hypot(x, y)`, so no intermediate overflow).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(3.0, 4.0).len == 5.0`
    @get pub fn len(self) -> F64 = todo

    /// `self` rotated 90 degrees counter-clockwise: `Vec(-y, x)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(1.0, 2.0).perp == Vec(-2.0, 1.0)`
    @get pub fn perp(self) -> Vec = todo

    /// `self` rotated 90 degrees clockwise: `Vec(y, -x)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(1.0, 2.0).perp_cw == Vec(2.0, -1.0)`
    @get pub fn perp_cw(self) -> Vec = todo

    /// The polar angle `atan2(y, x)` in `(-pi, pi]`; a `-0.0` `y` counts as `0.0`, so the negative x
    /// axis gives `PI`, and the zero vector gives 0.0.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(1.0, 0.0).angle == 0.0`
    /// example: `Vec(-1.0, 0.0).angle == PI`
    /// example: `Vec(0.0, 1.0).angle == PI / 2.0`
    @get pub fn angle(self) -> F64 = todo

    /// The unit vector `Vec(cos(rad), sin(rad))`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec.from_angle(0.0) == Vec(1.0, 0.0)`
    /// example: `close(Vec.from_angle(PI / 2.0).y, 1.0)`
    pub fn from_angle(rad: F64) -> Vec = todo

    /// The unit vector in the direction of `self`: each coordinate divided by `self.len`.
    ///
    /// cost: O(1)
    /// traps: when `self.len == 0.0`
    /// example: `Vec(3.0, 4.0).unit == Vec(0.6, 0.8)`
    @get pub fn unit(self) -> Vec = todo

    /// `self` rotated by the direction `by`, as the complex product `self * by`:
    /// `Vec(x * by.x - y * by.y, x * by.y + y * by.x)`. A unit `by` rotates without scaling; get
    /// one from `Vec.from_angle`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(1.0, 2.0).rot(Vec(0.0, 1.0)) == Vec(-2.0, 1.0)`
    pub fn rot(self, by: Vec) -> Vec = todo

    /// The dot product `x * o.x + y * o.y`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(1.0, 2.0).dot(Vec(3.0, 4.0)) == 11.0`
    pub fn dot(self, o: Vec) -> F64 = todo

    /// The 2D cross product `x * o.y - y * o.x`: positive when `o` is counter-clockwise from `self`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(1.0, 2.0).cross(Vec(3.0, 4.0)) == -2.0`
    pub fn cross(self, o: Vec) -> F64 = todo

    /// The squared Euclidean distance from `self` to `o`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(0.0, 0.0).dist2(Vec(3.0, 4.0)) == 25.0`
    pub fn dist2(self, o: Vec) -> F64 = todo

    /// The Euclidean distance from `self` to `o`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec(0.0, 0.0).dist(Vec(3.0, 4.0)) == 5.0`
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

/// Which way the path `a -> b -> c` turns, with tolerance: `sign_eps((b - a).cross(c - a), eps)`,
/// so 1 left, -1 right, 0 straight within `eps`.
///
/// `eps` bounds the raw cross product (twice the triangle's area), not an angle.
///
/// cost: O(1)
/// traps: never
/// example: `turn_eps(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0)) == 1`
/// example: `turn_eps(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, -1.0)) == -1`
/// example: `turn_eps(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(2.0, 1e-12)) == 0`
pub fn turn_eps(
    a: Vec,
    b: Vec,
    c: Vec,
    eps: F64 = 1e-9,
) -> Int = todo

/// The signed angle from `a` to `b`, `atan2(a.cross(b), a.dot(b))`, in `(-pi, pi]`: positive when
/// `b` is counter-clockwise from `a`. 0.0 when either is the zero vector.
///
/// cost: O(1)
/// traps: never
/// example: `angle_between(Vec(1.0, 0.0), Vec(0.0, 1.0)) == PI / 2.0`
/// example: `angle_between(Vec(1.0, 0.0), Vec(0.0, -1.0)) == -PI / 2.0`
/// example: `angle_between(Vec(1.0, 0.0), Vec(-1.0, 0.0)) == PI`
pub fn angle_between(a: Vec, b: Vec) -> F64 = todo

/// The orthogonal projection of `p` onto the line through `a` and `b`:
/// `a + (b - a) * ((p - a).dot(b - a) / (b - a).norm2)`.
///
/// cost: O(1)
/// traps: when `a == b`
/// example: `project(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == Vec(1.0, 0.0)`
pub fn project(p: Vec, a: Vec, b: Vec) -> Vec = todo

/// The mirror image of `p` across the line through `a` and `b`: `project(p, a, b) * 2.0 - p`.
///
/// cost: O(1)
/// traps: when `a == b`
/// example: `reflect(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == Vec(1.0, -1.0)`
pub fn reflect(p: Vec, a: Vec, b: Vec) -> Vec = todo

/// The perpendicular distance from `p` to the infinite line through `a` and `b`:
/// `|(b - a).cross(p - a)| / (b - a).len`.
///
/// cost: O(1)
/// traps: when `a == b`
/// example: `line_dist(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 1.0`
/// example: `line_dist(Vec(5.0, 3.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 3.0`
pub fn line_dist(p: Vec, a: Vec, b: Vec) -> F64 = todo

/// The shortest distance from `p` to the closed segment `a b`: to the projection when it falls
/// inside the segment, else to the nearer endpoint. A segment shorter than `eps` counts as the
/// point `a`.
///
/// cost: O(1)
/// traps: never
/// example: `seg_dist(Vec(1.0, 1.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 1.0`
/// example: `seg_dist(Vec(-3.0, 4.0), Vec(0.0, 0.0), Vec(2.0, 0.0)) == 5.0`
/// example: `seg_dist(Vec(3.0, 4.0), Vec(0.0, 0.0), Vec(0.0, 0.0)) == 5.0`
pub fn seg_dist(p: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> F64 = todo

/// The intersection of the infinite lines `a b` and `c d`, or `None` when
/// `|(b - a).cross(d - c)| <= eps` (parallel or coincident).
///
/// The point is `a + (b - a) * t` with `t = (c - a).cross(d - c) / (b - a).cross(d - c)`.
///
/// cost: O(1)
/// traps: never
/// example: `line_intersect(Vec(0.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0), Vec(2.0, 0.0)) == Some(Vec(1.0, 1.0))`
/// example: `line_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == None`
pub fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec = todo

/// The common points of the closed segments `a b` and `c d`: `[]` when disjoint, one point when
/// they cross or touch, and the two ends of the shared piece when they overlap collinearly.
///
/// cost: O(1)
/// empty: `[]` when disjoint
/// order: by `(x, y)` ascending; a new `List`, keyed from 0
/// traps: never
/// example: `segment_intersect(Vec(0.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0), Vec(2.0, 0.0)) == [Vec(1.0, 1.0)]`
/// example: `segment_intersect(Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(3.0, 0.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0), Vec(2.0, 0.0)]`
/// example: `segment_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == []`
pub fn segment_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// The first point where the ray from `o` in direction `dir` meets the closed segment `a b`, or
/// `None`.
///
/// When the ray runs along the segment, the hit is the segment point nearest `o` (`o` itself if
/// it lies on the segment).
///
/// cost: O(1)
/// traps: when `dir` is the zero vector
/// example: `ray_hits_segment(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(2.0, -1.0), Vec(2.0, 1.0)) == Some(Vec(2.0, 0.0))`
/// example: `ray_hits_segment(Vec(0.0, 0.0), Vec(-1.0, 0.0), Vec(2.0, -1.0), Vec(2.0, 1.0)) == None`
/// example: `ray_hits_segment(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(3.0, 0.0), Vec(5.0, 0.0)) == Some(Vec(3.0, 0.0))`
pub fn ray_hits_segment(o: Vec, dir: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> ?Vec = todo

/// The centre of the circle through `a`, `b` and `c`, or `None` when
/// `|(b - a).cross(c - a)| <= eps` (collinear).
///
/// cost: O(1)
/// traps: never
/// example: `circumcenter(Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0)) == Some(Vec(3.0, 4.0))`
/// example: `circumcenter(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None`
pub fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec = todo

/// O(1) value type. A 2D circle with center `c` and non-negative radius `r`.
pub struct Circle(c: Vec, r: F64):
    /// The circle through `a`, `b` and `c`: centre `circumcenter(a, b, c, eps)`, radius its distance
    /// to `a`; `None` when collinear.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Circle.through(Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0)) == Some(Circle(Vec(3.0, 4.0), 5.0))`
    /// example: `Circle.through(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None`
    pub fn through(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle = todo

    /// The disk area, computed as `PI * r * r`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Circle(Vec(0.0, 0.0), 1.0).area == PI`
    @get pub fn area(self) -> F64 = todo

    /// The circumference, computed as `2.0 * PI * r`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Circle(Vec(0.0, 0.0), 0.5).circumference == PI`
    @get pub fn circumference(self) -> F64 = todo

    /// Where `p` is relative to the disk, with `d = c.dist(p)`: `Boundary` when `|d - r| <= eps`,
    /// else `Inside` when `d < r`, else `Outside`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Circle(Vec(0.0, 0.0), 5.0).locate(Vec(3.0, 4.0)) == Location.Boundary`
    /// example: `Circle(Vec(0.0, 0.0), 5.0).locate(Vec(1.0, 1.0)) == Location.Inside`
    /// example: `Circle(Vec(0.0, 0.0), 5.0).locate(Vec(6.0, 0.0)) == Location.Outside`
    pub fn locate(self, p: Vec, eps: F64 = 1e-9) -> Location = todo

/// The inscribed circle of the triangle `a b c`, or `None` when `|(b - a).cross(c - a)| <= eps`
/// (collinear).
///
/// The centre is the side-length-weighted mean `(|bc| a + |ca| b + |ab| c) / (|ab| + |bc| + |ca|)`
/// and the radius `|(b - a).cross(c - a)| / (|ab| + |bc| + |ca|)`.
///
/// cost: O(1)
/// traps: never
/// example: `incenter(Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0)) == Some(Circle(Vec(2.0, 2.0), 2.0))`
/// example: `incenter(Vec(0.0, 0.0), Vec(1.0, 1.0), Vec(2.0, 2.0)) == None`
pub fn incenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Circle = todo

/// The points where `circle` meets the infinite line through `a` and `b`: two when it crosses,
/// one (the foot of the perpendicular from the centre) when the distance is within `eps` of `r`,
/// and none otherwise.
///
/// cost: O(1)
/// empty: `[]` when the line misses
/// order: along the direction `a -> b`; a new `List`, keyed from 0
/// traps: when `a == b`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 3.0), Vec(10.0, 3.0)) == [Vec(-4.0, 3.0), Vec(4.0, 3.0)]`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(10.0, 3.0), Vec(-10.0, 3.0)) == [Vec(4.0, 3.0), Vec(-4.0, 3.0)]`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 5.0), Vec(10.0, 5.0)) == [Vec(0.0, 5.0)]`
/// example: `circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 9.0), Vec(10.0, 9.0)) == []`
pub fn circle_line_intersect(circle: Circle, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// The points where the circles `a` and `b` meet: two when they cross, one when tangent (within
/// `eps`), none when apart, nested, or identical (infinitely many).
///
/// cost: O(1)
/// empty: `[]`
/// order: the point clockwise of the ray from `a.c` to `b.c` first; a new `List`, keyed from 0
/// traps: never
/// example: `circle_intersect(Circle(Vec(0.0, 0.0), 5.0), Circle(Vec(8.0, 0.0), 5.0)) == [Vec(4.0, -3.0), Vec(4.0, 3.0)]`
/// example: `circle_intersect(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(2.0, 0.0), 1.0)) == [Vec(1.0, 0.0)]`
/// example: `circle_intersect(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)) == []`
pub fn circle_intersect(a: Circle, b: Circle, eps: F64 = 1e-9) -> List<Vec> = todo

/// The points of `circle` where a tangent line through `p` touches it: two when `p` is outside,
/// `[p]` when `p` is on the circle (within `eps`), none when inside.
///
/// cost: O(1)
/// empty: `[]` when `p` is inside
/// order: the point clockwise of the ray from the centre to `p` first; a new `List`, keyed from 0
/// traps: never
/// example: `circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0)]`
/// example: `circle_tangents(Circle(Vec(0.0, 0.0), 2.0), Vec(1.0, 0.0)) == []`
/// example: `let t = circle_tangents(Circle(Vec(0.0, 0.0), 3.0), Vec(5.0, 0.0)); t.len == 2 and close(t[0].x, 1.8) and close(t[0].y, -2.4) and close(t[1].y, 2.4)`
pub fn circle_tangents(circle: Circle, p: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// The common tangent segments `(p1, p2)` of the circles `a` and `b`, `p1` on `a` and `p2` on
/// `b`: the outer tangents, or the inner ones (crossing between the centres) with `inner: true`.
///
/// Two segments when they exist, one (with `p1 == p2`) when the circles touch on that side, none
/// otherwise, and none for identical circles.
///
/// cost: O(1)
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
) -> List<(Vec, Vec)> = todo

/// The area of the intersection of the disks `a` and `b`: 0.0 when apart, the smaller disk's
/// `area` when one contains the other (within `eps`), else the sum of the two circular segments.
///
/// cost: O(1)
/// traps: never
/// example: `circle_circle_area(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)) == 0.0`
/// example: `circle_circle_area(Circle(Vec(0.0, 0.0), 2.0), Circle(Vec(0.0, 0.0), 1.0)) == PI`
/// example: `close(circle_circle_area(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(1.0, 0.0), 1.0)), 1.228369698608757)`
pub fn circle_circle_area(a: Circle, b: Circle, eps: F64 = 1e-9) -> F64 = todo

/// The inverse of `p` in the circle `across`: `c + (p - c) * (r * r / (p - c).norm2)`.
///
/// cost: O(1)
/// traps: when `p == across.c`
/// example: `invert(Vec(4.0, 0.0), Circle(Vec(0.0, 0.0), 2.0)) == Vec(1.0, 0.0)`
/// example: `invert(Vec(2.0, 0.0), Circle(Vec(0.0, 0.0), 2.0)) == Vec(2.0, 0.0)`
pub fn invert(p: Vec, across: Circle) -> Vec = todo

/// The area of the union of the disks `circles`, by Green's theorem over the arcs on the union's
/// boundary.
///
/// Equal circles (within `eps`) count once; zero-radius circles add nothing.
///
/// cost: O(n^2 log n) time, O(n) memory
/// empty: 0.0
/// traps: never
/// example: `circles_union_area([]) == 0.0`
/// example: `close(circles_union_area([Circle(Vec(0.0, 0.0), 1.0)]), PI)`
/// example: `close(circles_union_area([Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)]), 2.0 * PI)`
pub fn circles_union_area(circles: List<Circle>, eps: F64 = 1e-9) -> F64 = todo

// ---- half-plane intersection

/// O(1) value type. Directed line through `p` in direction `dir`, keeping the closed left
/// half-plane `{ q | dir.cross(q - p) >= 0 }`.
pub struct HalfPlane(p: Vec, dir: Vec):
    /// The closed half-plane left of the directed line `a -> b`: `HalfPlane(a, b - a)`.
    ///
    /// cost: O(1)
    /// traps: when `a == b`
    /// example: `HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)) == HalfPlane(Vec(0.0, 0.0), Vec(1.0, 0.0))`
    pub fn through(a: Vec, b: Vec) -> HalfPlane = todo

    /// The half-plane `{ (x, y) | a*x + b*y + c >= 0 }`, as `HalfPlane(p, Vec(b, -a))` with `p` the
    /// line's point nearest the origin, `Vec(-a * c, -b * c) / (a*a + b*b)`.
    ///
    /// cost: O(1)
    /// traps: when `a == 0.0` and `b == 0.0`
    /// example: `HalfPlane.from_line(0.0, 1.0, -2.0) == HalfPlane(Vec(0.0, 2.0), Vec(1.0, 0.0))`
    /// example: `HalfPlane.from_line(1.0, 0.0, 0.0).side(Vec(3.0, 7.0)) == 3.0`
    pub fn from_line(a: F64, b: F64, c: F64) -> HalfPlane = todo

    /// The signed value `dir.cross(q - p)`: positive left of the line (inside), 0 on it, negative
    /// outside. It is `|dir|` times the signed distance.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)).side(Vec(3.0, 2.0)) == 2.0`
    /// example: `HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)).side(Vec(3.0, -2.0)) == -2.0`
    pub fn side(self, q: Vec) -> F64 = todo

fn hp_out(
    h: (p: Vec, dir: Vec, ang: F64),
    pt: Vec,
    eps: F64,
) -> Bool = todo

fn hp_cross(a: (p: Vec, dir: Vec, ang: F64), b: (p: Vec, dir: Vec, ang: F64)) -> Vec = todo

/// The intersection of the closed half-planes `planes`, as a counter-clockwise convex polygon
/// (sort by angle, then a deque sweep).
///
/// With `bound: Some(b)` the box `[-b, b]^2` is added as four more half-planes. A region that is
/// unbounded, empty, or of area at most `eps` gives `[]`. Parallel planes keep the tighter one;
/// vertices within `eps` merge.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]` (unbounded) without `bound`; the box with one
/// order: counter-clockwise, starting at the least `(y, x)`; a new `List`, keyed from 0
/// traps: when a plane's `dir` is the zero vector
/// example: `half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)), HalfPlane.through(Vec(1.0, 0.0), Vec(1.0, 1.0)), HalfPlane.through(Vec(1.0, 1.0), Vec(0.0, 1.0)), HalfPlane.through(Vec(0.0, 1.0), Vec(0.0, 0.0))]) == [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(0.0, 1.0)]`
/// example: `half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0))]) == []`
/// example: `half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0))], bound: Some(1.0)) == [Vec(-1.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(-1.0, 1.0)]`
pub fn half_plane_intersect(
    planes: List<HalfPlane>,
    bound: ?F64 = None,
    eps: F64 = 1e-9,
) -> List<Vec> = todo

/// The signed area of the polygon `poly` (half the shoelace sum): positive counter-clockwise,
/// negative clockwise; 0.0 for fewer than 3 vertices.
///
/// cost: O(n)
/// empty: 0.0
/// traps: never
/// example: `polygon_area([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(0.0, 2.0)]) == 2.0`
/// example: `polygon_area([Vec(0.0, 0.0), Vec(0.0, 2.0), Vec(2.0, 0.0)]) == -2.0`
pub fn polygon_area(poly: List<Vec>) -> F64 = todo

/// The part of `poly` in the closed half-plane left of the directed line `a -> b`
/// (Sutherland-Hodgman).
///
/// Walking the edges from `poly[0]`, it keeps each vertex with `turn_eps(a, b, v, eps) >= 0`
/// and adds the crossing point of every edge that strictly crosses the line. The result keeps the
/// orientation of `poly`.
///
/// cost: O(n) time and memory
/// empty: `[]`
/// order: the walk from `poly[0]`; a new `List`, keyed from 0
/// traps: when `a == b`
/// example: `polygon_cut([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], Vec(1.0, 0.0), Vec(1.0, 2.0)) == [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 2.0), Vec(0.0, 2.0)]`
/// example: `polygon_cut([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], Vec(1.0, 2.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(1.0, 2.0)]`
/// example: `polygon_cut([Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0)], Vec(0.0, 5.0), Vec(1.0, 5.0)) == []`
pub fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec> = todo

/// The smallest circle containing every point of `pts` (Welzl's algorithm, deterministic: points
/// are processed in input order, with no shuffle).
///
/// The circle is unique; its centre is the midpoint of two points or a circumcentre of three.
/// Without the shuffle the expected O(n) bound needs input not ordered adversarially; shuffle
/// first with `Rng.shuffle` if it might be.
///
/// cost: O(n) expected for unadversarial order
/// empty: `Circle(Vec(0.0, 0.0), 0.0)`
/// ties: none; the smallest enclosing circle is unique
/// traps: never
/// example: `min_enclosing_circle([Vec(0.0, 0.0), Vec(2.0, 0.0)]) == Circle(Vec(1.0, 0.0), 1.0)`
/// example: `min_enclosing_circle([Vec(0.0, 0.0), Vec(6.0, 0.0), Vec(0.0, 8.0), Vec(1.0, 1.0)]) == Circle(Vec(3.0, 4.0), 5.0)`
/// example: `min_enclosing_circle([Vec(2.0, 3.0)]) == Circle(Vec(2.0, 3.0), 0.0)`
pub fn min_enclosing_circle(pts: List<Vec>, eps: F64 = 1e-9) -> Circle = todo

fn tri_circle_area(a: Vec, b: Vec, r: F64, eps: F64) -> F64 = todo

/// The signed area of the intersection of the disk `circle` with the simple polygon `poly`:
/// positive when `poly` is counter-clockwise, negative when clockwise. Summed over the triangles
/// from the centre to each edge.
///
/// cost: O(n)
/// empty: 0.0
/// traps: never
/// example: `circle_polygon_area(Circle(Vec(0.0, 0.0), 10.0), [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(0.0, 1.0)]) == 1.0`
/// example: `circle_polygon_area(Circle(Vec(0.0, 0.0), 10.0), [Vec(0.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0), Vec(1.0, 0.0)]) == -1.0`
/// example: `close(circle_polygon_area(Circle(Vec(0.0, 0.0), 1.0), [Vec(-2.0, -2.0), Vec(2.0, -2.0), Vec(2.0, 2.0), Vec(-2.0, 2.0)]), PI)`
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

/// Whether any two segments of `segs` share a point, by a Shamos-Hoey sweep with exact `I128`
/// comparisons (valid for `|x|, |y| <= 1e12`): `Some((i, j))` with `i < j` for a pair that
/// meets, or `None`.
///
/// With `strict: true` only proper interior crossings count, as in `segments_meet`. When several
/// pairs meet, the pair is the first one the sweep finds, not necessarily the least.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `None`
/// ties: the first meeting pair found by the sweep, as `(i, j)` with `i < j`
/// traps: never
/// example: `any_segments_meet([(Pt(0, 0), Pt(2, 2)), (Pt(5, 5), Pt(6, 6)), (Pt(0, 2), Pt(2, 0))]) == Some((0, 2))`
/// example: `any_segments_meet([(Pt(0, 0), Pt(1, 0)), (Pt(0, 1), Pt(1, 1))]) == None`
/// example: `any_segments_meet([(Pt(0, 0), Pt(1, 1)), (Pt(1, 1), Pt(2, 0))], strict: true) == None`
pub fn any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int) = todo

/// `any_segments_meet` over a domain-indexed array: `Some((a, b))` with `a` before `b` in key
/// order for a pair of segments that meet, or `None`.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `None`
/// ties: the first meeting pair found by the sweep, `a` before `b` in key order
/// traps: never
/// example: `any_segments_meet_idx([(Pt(0, 0), Pt(1, 1)), (Pt(1, 1), Pt(2, 0))]) == Some((0, 1))`
pub fn any_segments_meet_idx<D>(segs: [D](a: Pt, b: Pt), strict: Bool = false) -> ?(D, D) = todo

// ---- axis-aligned rectangles, planar subdivisions, and Delaunay / Voronoi

/// O(1) value type. Half-open integer rectangle `x * y` (accepts cut ranges per decision 124).
pub struct Rect(x: Range<Int>, y: Range<Int>):
    /// The number of unit cells, `x.len * y.len`; 0 when either range is empty.
    ///
    /// cost: O(1)
    /// traps: when the product overflows `Int`
    /// example: `Rect(0..3, 0..2).area == 6`
    /// example: `Rect(3..1, 0..2).area == 0`
    @get pub fn area(self) -> Int = todo

    /// Whether either axis range is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Rect(0..0, 0..5).is_empty == true`
    /// example: `Rect(0..1, 0..1).is_empty == false`
    @get pub fn is_empty(self) -> Bool = todo

    /// Whether `p.x` is in `x` and `p.y` is in `y` (half-open, so the top and right edges are out).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Rect(0..3, 0..2).contains(Pt(2, 1)) == true`
    /// example: `Rect(0..3, 0..2).contains(Pt(3, 1)) == false`
    pub fn contains(self, p: Pt) -> Bool = todo

    /// The common part of two rectangles, or `None` when it is empty (including rectangles that only
    /// touch along an edge).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Rect(0..3, 0..3).intersect(Rect(1..5, 2..4)) == Some(Rect(1..3, 2..3))`
    /// example: `Rect(0..1, 0..1).intersect(Rect(1..2, 0..1)) == None`
    pub fn intersect(self, o: Rect) -> ?Rect = todo

/// The area of the union of `rects`, by a sweep over x with a segment tree over the compressed y
/// coordinates. Empty rectangles add nothing.
///
/// cost: O(n log n) time, O(n) memory
/// empty: 0
/// traps: when the area overflows `Int`
/// example: `rects_union_area([Rect(0..2, 0..2), Rect(1..3, 1..3)]) == 7`
/// example: `rects_union_area([Rect(0..2, 0..2), Rect(0..2, 0..2)]) == 4`
/// example: `rects_union_area([]) == 0`
pub fn rects_union_area(rects: List<Rect>) -> Int = todo

/// The faces of the planar straight-line graph with vertices `pts` and undirected `edges`, each
/// as its cycle of vertices: bounded faces counter-clockwise, the outer face of each connected
/// component clockwise.
///
/// Each face is traced by turning to the next edge clockwise at every vertex. A bridge edge
/// appears twice in its face; isolated vertices form no face. The graph must be planar (edges
/// meet only at shared endpoints); that is not checked.
///
/// cost: O((n + m) log m) time, O(n + m) memory
/// empty: `[]` when `edges` is empty
/// order: each cycle starts at its least vertex in key order; faces sorted by cycle; a new `List`
/// traps: when an edge joins a vertex to itself, or an edge repeats
/// example: `planar_faces([Pt(0, 0), Pt(1, 0), Pt(0, 1)], [(0, 1), (1, 2), (2, 0)]) == [[0, 1, 2], [0, 2, 1]]`
/// example: `planar_faces([Pt(0, 0), Pt(1, 0)], [(0, 1)]) == [[0, 1]]`
pub fn planar_faces<V>(pts: [V]Pt, edges: List<(V, V)>) -> List<List<V>> = todo

/// For each query point, the index into `planar_faces(pts, edges)` of the bounded face that
/// contains it, or `None` when it lies on an edge or vertex or in the unbounded region (offline
/// sweep).
///
/// cost: O((n + m + q) log(n + m)) time, O(n + m + q) memory
/// empty: `[]` when `queries` is empty
/// order: one answer per query, in query order; a new `List`, keyed from 0
/// traps: as `planar_faces`
/// example: `point_location_planar([Pt(0, 0), Pt(4, 0), Pt(0, 4)], [(0, 1), (1, 2), (2, 0)], [Pt(1, 1), Pt(0, 0), Pt(5, 5)]) == [Some(0), None, None]`
pub fn point_location_planar<V>(
    pts: [V]Pt,
    edges: List<(V, V)>,
    queries: List<Pt>,
) -> List<?Int> = todo

/// The Delaunay triangulation of distinct integer points (`|coords| <= 1e9`), exact in `I128`,
/// as counter-clockwise handle triples.
///
/// Where four or more points are cocircular the triangulation is not unique; the algorithm picks
/// one of the valid ones. Fewer than 3 points, or all collinear, give `[]`.
///
/// cost: O(n log n) time, O(n) memory
/// empty: `[]`
/// order: each triple rotated to start at its least handle in key order; triples sorted; a new `List`
/// traps: when two points are equal
/// example: `delaunay([Pt(0, 0), Pt(4, 0), Pt(0, 4)]) == [(0, 1, 2)]`
/// example: `delaunay([Pt(0, 0), Pt(4, 0), Pt(0, 4), Pt(1, 1)]) == [(0, 1, 3), (0, 3, 2), (1, 2, 3)]`
/// example: `delaunay([Pt(0, 0), Pt(1, 1), Pt(2, 2)]) == []`
pub fn delaunay<D>(pts: [D]Pt) -> List<(D, D, D)> = todo

/// The Voronoi cell of each point, clipped to the box `[-bound, bound]^2`, as a counter-clockwise
/// polygon (from the Delaunay dual, then half-plane clipping).
///
/// The result keeps the domain of `pts`. A cell that misses the box is `[]`; a single point's
/// cell is the whole box.
///
/// cost: O(n log n) time, O(n) memory
/// empty: an empty array
/// order: each cell counter-clockwise, starting at its least `(y, x)` corner
/// traps: when two points are equal or `bound <= 0.0`
/// example: `voronoi_cells([Pt(-1, 0), Pt(1, 0)], 2.0)[0] == [Vec(-2.0, -2.0), Vec(0.0, -2.0), Vec(0.0, 2.0), Vec(-2.0, 2.0)]`
/// example: `voronoi_cells([Pt(-1, 0), Pt(1, 0)], 2.0)[1] == [Vec(0.0, -2.0), Vec(2.0, -2.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]`
/// example: `voronoi_cells([Pt(0, 0)], 1.0)[0] == [Vec(-1.0, -1.0), Vec(1.0, -1.0), Vec(1.0, 1.0), Vec(-1.0, 1.0)]`
pub fn voronoi_cells<D>(pts: [D]Pt, bound: F64) -> [D]List<Vec> = todo

// ---- 3D exact & floating-point geometry

/// O(1) value type. A 3D point or vector with exact integer coordinates.
pub struct Pt3(x: Int, y: Int, z: Int):
    /// The squared length `x*x + y*y + z*z`, in `Int`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(1, 2, 2).norm2 == 9`
    @get pub fn norm2(self) -> Int = todo

    /// The exact squared length, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `I128`
    /// example: `Pt3(1, 2, 2).norm2_128 == 9`
    @get pub fn norm2_128(self) -> I128 = todo

    /// The Euclidean length as `F64`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt3(1, 2, 2).len == 3.0`
    @get pub fn len(self) -> F64 = todo

    /// `self` as a floating-point `Vec3`: the explicit way across.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt3(1, 2, 2).to_vec3 == Vec3(1.0, 2.0, 2.0)`
    @get pub fn to_vec3(self) -> Vec3 = todo

    /// The dot product `x * o.x + y * o.y + z * o.z`, in `Int`.
    ///
    /// cost: O(1)
    /// traps: when the result or a product overflows `Int`
    /// example: `Pt3(1, 2, 3).dot(Pt3(4, 5, 6)) == 32`
    pub fn dot(self, o: Pt3) -> Int = todo

    /// The exact dot product, computed in `I128`.
    ///
    /// cost: O(1)
    /// traps: when the sum overflows `I128`
    /// example: `Pt3(1, 2, 3).dot128(Pt3(4, 5, 6)) == 32`
    pub fn dot128(self, o: Pt3) -> I128 = todo

    /// The cross product `Pt3(y*o.z - z*o.y, z*o.x - x*o.z, x*o.y - y*o.x)`, perpendicular to both by
    /// the right-hand rule.
    ///
    /// cost: O(1)
    /// traps: when a component overflows `Int`
    /// example: `Pt3(1, 0, 0).cross(Pt3(0, 1, 0)) == Pt3(0, 0, 1)`
    /// example: `Pt3(1, 2, 3).cross(Pt3(4, 5, 6)) == Pt3(-3, 6, -3)`
    pub fn cross(self, o: Pt3) -> Pt3 = todo

    /// The squared Euclidean distance from `self` to `o`, in `Int`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(0, 0, 0).dist2(Pt3(1, 2, 2)) == 9`
    pub fn dist2(self, o: Pt3) -> Int = todo

    /// The Euclidean distance from `self` to `o`, as `F64`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pt3(0, 0, 0).dist(Pt3(1, 2, 2)) == 3.0`
    pub fn dist(self, o: Pt3) -> F64 = todo

    /// The Manhattan (L1) distance `|x - o.x| + |y - o.y| + |z - o.z|`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(1, 1, 1).manhattan(Pt3(2, -1, 4)) == 6`
    pub fn manhattan(self, o: Pt3) -> Int = todo

    /// The Chebyshev (L-infinity) distance `max(|x - o.x|, |y - o.y|, |z - o.z|)`.
    ///
    /// cost: O(1)
    /// traps: when the result overflows `Int`
    /// example: `Pt3(1, 1, 1).chebyshev(Pt3(2, -1, 4)) == 3`
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

/// The 3D convex hull of `pts` as triangular faces of indices, each counter-clockwise seen from
/// outside (so its normal points out), exact in `I128` (valid for `|x|, |y|, |z| <= 1e12`).
///
/// Only strict corners appear; a flat face with more than 3 corners is split as a fan from its
/// least index. Of equal points the smallest index stands for them. Fewer than 4 non-coplanar
/// points give `[]`.
///
/// cost: O(n^2) time, O(n) memory
/// empty: `[]`
/// order: each face rotated to start at its least index; faces sorted; a new `List`, keyed from 0
/// traps: never
/// example: `convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]`
/// example: `convex_hull_3d([Pt3(0, 0, 0), Pt3(4, 0, 0), Pt3(0, 4, 0), Pt3(0, 0, 4), Pt3(1, 1, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]`
/// example: `convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0)]) == []`
pub fn convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)> = todo

/// `convex_hull_3d` over a domain-indexed array, with faces of handles; the same orientation,
/// order and tie rules, in key order.
///
/// cost: O(n^2) time, O(n) memory
/// empty: `[]`
/// order: each face rotated to start at its least handle; faces sorted; a new `List`
/// traps: never
/// example: `convex_hull_3d_idx([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]`
pub fn convex_hull_3d_idx<D>(pts: [D]Pt3) -> List<(D, D, D)> = todo

/// Six times the signed volume of the closed triangulated surface `faces` over `pts`: the sum of
/// `pts[a].dot128(pts[b].cross(pts[c]))` over faces, computed in `I128`. Positive when faces are
/// counter-clockwise seen from outside, as `convex_hull_3d` returns them.
///
/// cost: O(|faces|)
/// empty: 0
/// traps: when a face index is out of range
/// example: `polyhedron_volume6([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)], [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]) == 1`
/// example: `polyhedron_volume6([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)], [(0, 3, 1), (0, 1, 2), (0, 2, 3), (1, 3, 2)]) == -1`
pub fn polyhedron_volume6(pts: List<Pt3>, faces: List<(Int, Int, Int)>) -> I128 = todo

/// O(1) value type. A 3D floating-point point or vector.
pub struct Vec3(x: F64, y: F64, z: F64):
    /// The squared length `x*x + y*y + z*z`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 2.0).norm2 == 9.0`
    @get pub fn norm2(self) -> F64 = todo

    /// The Euclidean length.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 2.0).len == 3.0`
    @get pub fn len(self) -> F64 = todo

    /// The unit vector in the direction of `self`: each coordinate divided by `self.len`.
    ///
    /// cost: O(1)
    /// traps: when `self.len == 0.0`
    /// example: `Vec3(0.0, 0.0, 2.0).unit == Vec3(0.0, 0.0, 1.0)`
    @get pub fn unit(self) -> Vec3 = todo

    /// The dot product `x * o.x + y * o.y + z * o.z`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 3.0).dot(Vec3(4.0, 5.0, 6.0)) == 32.0`
    pub fn dot(self, o: Vec3) -> F64 = todo

    /// The cross product, by the right-hand rule, as for `Pt3.cross`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(1.0, 0.0, 0.0).cross(Vec3(0.0, 1.0, 0.0)) == Vec3(0.0, 0.0, 1.0)`
    pub fn cross(self, o: Vec3) -> Vec3 = todo

    /// The squared Euclidean distance from `self` to `o`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(0.0, 0.0, 0.0).dist2(Vec3(1.0, 2.0, 2.0)) == 9.0`
    pub fn dist2(self, o: Vec3) -> F64 = todo

    /// The Euclidean distance from `self` to `o`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(0.0, 0.0, 0.0).dist(Vec3(1.0, 2.0, 2.0)) == 3.0`
    pub fn dist(self, o: Vec3) -> F64 = todo

    /// `self` rotated by `rad` radians counter-clockwise about the unit vector `axis` (seen with
    /// `axis` pointing at the viewer), by Rodrigues' formula
    /// `v cos t + axis.cross(v) sin t + axis * axis.dot(v) * (1 - cos t)`.
    ///
    /// `axis` must be a unit vector; that is not checked.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Vec3(1.0, 2.0, 3.0).rot(Vec3(0.0, 0.0, 1.0), 0.0) == Vec3(1.0, 2.0, 3.0)`
    /// example: `let v = Vec3(1.0, 0.0, 0.0).rot(Vec3(0.0, 0.0, 1.0), PI / 2.0); close(v.x, 0.0) and close(v.y, 1.0) and close(v.z, 0.0)`
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
    /// The plane through `a`, `b` and `c`: `n = (b - a).cross(c - a).unit`, `d = n.dot(a)`; `None`
    /// when `(b - a).cross(c - a).len <= eps` (collinear).
    ///
    /// The normal follows the right-hand rule, so `a -> b -> c` runs counter-clockwise seen from the
    /// side `n` points to.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Plane3.through(Vec3(0.0, 0.0, 1.0), Vec3(1.0, 0.0, 1.0), Vec3(0.0, 1.0, 1.0)) == Some(Plane3(Vec3(0.0, 0.0, 1.0), 1.0))`
    /// example: `Plane3.through(Vec3(0.0, 0.0, 0.0), Vec3(1.0, 1.0, 1.0), Vec3(2.0, 2.0, 2.0)) == None`
    pub fn through(a: Vec3, b: Vec3, c: Vec3, eps: F64 = 1e-9) -> ?Plane3 = todo

    /// The signed distance `n.dot(p) - d`: positive on the side `n` points to.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Plane3(Vec3(0.0, 0.0, 1.0), 1.0).dist(Vec3(5.0, 5.0, 3.0)) == 2.0`
    /// example: `Plane3(Vec3(0.0, 0.0, 1.0), 1.0).dist(Vec3(5.0, 5.0, 0.0)) == -1.0`
    pub fn dist(self, p: Vec3) -> F64 = todo

    /// The orthogonal projection of `p` onto the plane: `p - n * self.dist(p)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Plane3(Vec3(0.0, 0.0, 1.0), 1.0).project(Vec3(5.0, 5.0, 3.0)) == Vec3(5.0, 5.0, 1.0)`
    pub fn project(self, p: Vec3) -> Vec3 = todo

/// The point where the infinite line through `a` and `b` meets `plane`, or `None` when
/// `|plane.n.dot(b - a)| <= eps` (parallel, including lying in the plane, or `a == b`).
///
/// The point is `a + (b - a) * t` with `t = (plane.d - plane.n.dot(a)) / plane.n.dot(b - a)`.
///
/// cost: O(1)
/// traps: never
/// example: `line_plane_intersect(Vec3(0.0, 0.0, 0.0), Vec3(0.0, 0.0, 2.0), Plane3(Vec3(0.0, 0.0, 1.0), 1.0)) == Some(Vec3(0.0, 0.0, 1.0))`
/// example: `line_plane_intersect(Vec3(0.0, 0.0, 0.0), Vec3(1.0, 0.0, 0.0), Plane3(Vec3(0.0, 0.0, 1.0), 1.0)) == None`
pub fn line_plane_intersect(a: Vec3, b: Vec3, plane: Plane3, eps: F64 = 1e-9) -> ?Vec3 = todo

/// The great-circle distance between the directions of `a` and `b` on a sphere of radius `r`:
/// `r * atan2(a.cross(b).len, a.dot(b))`. Lengths of `a` and `b` do not matter; 0.0 when either
/// is the zero vector.
///
/// cost: O(1)
/// traps: never
/// example: `sphere_dist(Vec3(1.0, 0.0, 0.0), Vec3(0.0, 1.0, 0.0)) == PI / 2.0`
/// example: `sphere_dist(Vec3(1.0, 0.0, 0.0), Vec3(-3.0, 0.0, 0.0), r: 2.0) == 2.0 * PI`
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
