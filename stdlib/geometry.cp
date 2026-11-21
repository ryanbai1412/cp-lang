//! Plane geometry on exact integer points (`Pt`) and careful floating-point
//! vectors (`Vec`). Exact predicates compute cross and dot signs in `I128`,
//! staying overflow-free for coordinates up to `1e18`.

fn igcd(a0: Int, b0: Int) -> Int:
    var a = a0
    var b = b0
    while b != 0:
        let t = a % b
        a = b
        b = t
    a

// ---- exact integer points

/// A 2D point or vector with exact integer coordinates.
pub struct Pt(x: Int, y: Int):
    /// The squared length (`x*x + y*y`). O(1).
    pub @get fn norm2(self) -> Int = x * x + y * y

    /// The exact squared length in 128-bit arithmetic. O(1).
    pub @get fn norm2_128(self) -> I128 = I128(x) * I128(x) + I128(y) * I128(y)

    /// The Euclidean length as `F64`. O(1).
    pub @get fn len(self) -> F64 = sqrt(F64(x) * F64(x) + F64(y) * F64(y))

    /// Rotated 90 degrees counter-clockwise. O(1).
    pub @get fn perp(self) -> Pt = Pt(-y, x)

    /// Rotated 90 degrees clockwise. O(1).
    pub @get fn perp_cw(self) -> Pt = Pt(y, -x)

    /// Polar angle in `(-pi, pi]`. O(1).
    pub @get fn angle(self) -> F64 = atan2(F64(y), F64(x))

    /// Reduced integer direction `(x / g, y / g)` with `g = gcd(|x|, |y|)`;
    /// returns `Pt(0, 0)` for the zero vector. O(log max(|x|, |y|)).
    pub @get fn unit_dir(self) -> Pt:
        if x == 0 and y == 0: return Pt(0, 0)
        let g = igcd(abs(x), abs(y))
        Pt(x / g, y / g)

    /// Explicit conversion to a floating-point `Vec`. O(1).
    pub @get fn to_vec(self) -> Vec = Vec(F64(x), F64(y))

    /// Scale by an integer factor `k`. O(1).
    pub fn scale(self, k: Int) -> Pt = Pt(x * k, y * k)

impl Add for Pt:
    fn add(a, b) = Pt(a.x + b.x, a.y + b.y)

impl Sub for Pt:
    fn sub(a, b) = Pt(a.x - b.x, a.y - b.y)

impl Neg for Pt:
    fn neg(a) = Pt(-a.x, -a.y)

impl Show for Pt:
    fn show(a) = "{a.x} {a.y}"

/// Dot product of `a` and `b`. O(1).
pub fn dot(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y

/// 2D cross product of `a` and `b`: positive when `b` is counter-clockwise
/// from `a`. O(1).
pub fn cross(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x

/// Exact 128-bit dot product of `a` and `b`. O(1).
pub fn dot128(a: Pt, b: Pt) -> I128 = I128(a.x) * I128(b.x) + I128(a.y) * I128(b.y)

/// Exact 128-bit cross product of `a` and `b`. O(1).
pub fn cross128(a: Pt, b: Pt) -> I128 = I128(a.x) * I128(b.y) - I128(a.y) * I128(b.x)

/// Squared Euclidean distance between `a` and `b`. O(1).
pub fn dist2(a: Pt, b: Pt) -> Int = (a - b).norm2

/// Exact 128-bit squared Euclidean distance between `a` and `b`. O(1).
pub fn dist2_128(a: Pt, b: Pt) -> I128:
    let dx = I128(a.x) - I128(b.x)
    let dy = I128(a.y) - I128(b.y)
    dx * dx + dy * dy

/// Euclidean distance between `a` and `b` as `F64`. O(1).
pub fn dist(a: Pt, b: Pt) -> F64 = (a - b).len

fn cross128_3(a: Pt, b: Pt, c: Pt) -> I128:
    (I128(b.x) - I128(a.x)) * (I128(c.y) - I128(a.y)) - (I128(b.y) - I128(a.y)) * (I128(c.x) - I128(a.x))

fn dot128_3(a: Pt, b: Pt, c: Pt) -> I128:
    (I128(b.x) - I128(a.x)) * (I128(c.x) - I128(a.x)) + (I128(b.y) - I128(a.y)) * (I128(c.y) - I128(a.y))

/// Which way the path `a -> b -> c` turns: 1 left, -1 right, 0 straight.
/// Exact in `I128` up to `1e18` coordinates. O(1).
pub fn turn(a: Pt, b: Pt, c: Pt) -> Int:
    let k = cross128_3(a, b, c)
    if k > I128(0): 1 else: if k < I128(0): -1 else: 0

/// True when `a` and `b` are nonzero and point in the same direction. O(1).
pub fn same_dir(a: Pt, b: Pt) -> Bool = cross128(a, b) == I128(0) and dot128(a, b) > I128(0)

// ---- angular sort

/// 0 for the zero vector, 1 for directions in `[0, pi)`, 2 for `[pi, 2*pi)`.
fn half(a: Pt) -> Int:
    if a.x == 0 and a.y == 0: return 0
    if a.y > 0 or (a.y == 0 and a.x > 0): 1 else: 2

/// Exact counter-clockwise angle comparator starting from the positive x-axis.
/// Vectors in the same direction compare equal; the zero vector comes first. O(1).
pub fn angle_less(a: Pt, b: Pt) -> Bool:
    let ha = half(a)
    let hb = half(b)
    if ha != hb: ha < hb else: cross128(a, b) > I128(0)

/// Counter-clockwise by direction, starting from the positive x axis. Exact;
/// vectors in the same direction compare equal, the zero vector comes first.
/// For sorting around an origin `base`, use `ByAngle(p - base)`.
pub struct ByAngle(p: Pt)

impl Ord for ByAngle:
    fn lt(a, b) = angle_less(a.p, b.p)

// ---- segments

/// Is `p` on the segment from `a` to `b`? With `strict: true`, endpoints do
/// not count. Exact in `I128`. O(1).
pub fn on_segment(p: Pt, a: Pt, b: Pt, strict: Bool = false) -> Bool:
    if cross128_3(p, a, b) != I128(0): return false
    let d = dot128_3(p, a, b)
    if strict: d < I128(0) else: d <= I128(0)

/// Do the segments `a b` and `c d` share a point? With `strict: true`,
/// requires a proper interior crossing (touching/collinear do not count).
/// Exact in `I128`. O(1).
pub fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool:
    let (t1, t2, t3, t4) = (turn(a, b, c), turn(a, b, d), turn(c, d, a), turn(c, d, b))
    if t1 * t2 < 0 and t3 * t4 < 0: return true
    if strict: return false
    on_segment(c, a, b) or on_segment(d, a, b) or on_segment(a, c, d) or on_segment(b, c, d)

// ---- convex hull and Minkowski sum

/// Lower convex hull chain (left to right) of `(x, y)`-sorted points.
/// Set `collinear: true` to keep boundary collinear points. O(n).
pub fn lower_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt>:
    var h: List<Pt> = []
    for p in sorted_pts:
        if h.len > 0 and h[h.len - 1] == p: continue
        while h.len >= 2:
            let t = turn(h[h.len - 2], h[h.len - 1], p)
            if t < 0 or (not collinear and t == 0): h.pop()
            else: break
        h.push(p)
    h

/// Upper convex hull chain (left to right) of `(x, y)`-sorted points.
/// Set `collinear: true` to keep boundary collinear points. O(n).
pub fn upper_hull(sorted_pts: List<Pt>, collinear: Bool = false) -> List<Pt>:
    var h: List<Pt> = []
    for p in sorted_pts:
        if h.len > 0 and h[h.len - 1] == p: continue
        while h.len >= 2:
            let t = turn(h[h.len - 2], h[h.len - 1], p)
            if t > 0 or (not collinear and t == 0): h.pop()
            else: break
        h.push(p)
    h

/// Counter-clockwise convex hull starting at the lexicographically smallest
/// `(x, y)` point. Set `collinear: true` to keep collinear boundary points.
/// O(n log n).
pub fn convex_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt>:
    if pts.len <= 1: return copy(pts)
    var s = copy(pts)
    sort(mut s)
    var uniq: List<Pt> = []
    for p in s:
        if uniq.len == 0 or uniq[uniq.len - 1] != p: uniq.push(p)
    if uniq.len <= 1: return uniq
    let lo = lower_hull(uniq, collinear: collinear)
    let hi = upper_hull(uniq, collinear: collinear)
    if lo.len == uniq.len and hi.len == uniq.len:
        // all unique points are collinear
        return lo
    var res = lo
    if hi.len >= 2:
        for i in (1..hi.len - 1).rev:
            res.push(hi[i])
    res

/// Counter-clockwise convex hull returning domain positions `D` instead of
/// copying points. Duplicate coordinates keep the smallest position. O(n log n).
pub fn convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>:
    var order = [i for i in D]
    if order.len <= 1: return order
    sort(mut order, by: i => pts[i])
    var uniq: List<D> = []
    for i in order:
        if uniq.len == 0 or pts[uniq[uniq.len - 1]] != pts[i]: uniq.push(i)
    if uniq.len <= 1: return uniq
    var lo: List<D> = []
    for i in uniq:
        let p = pts[i]
        while lo.len >= 2:
            let t = turn(pts[lo[lo.len - 2]], pts[lo[lo.len - 1]], p)
            if t < 0 or (not collinear and t == 0): lo.pop()
            else: break
        lo.push(i)
    var hi: List<D> = []
    for i in uniq:
        let p = pts[i]
        while hi.len >= 2:
            let t = turn(pts[hi[hi.len - 2]], pts[hi[hi.len - 1]], p)
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

fn rotate_min_yx(poly: List<Pt>) -> List<Pt>:
    var best = 0
    for i in 1..poly.len:
        if poly[i].y < poly[best].y or (poly[i].y == poly[best].y and poly[i].x < poly[best].x):
            best = i
    [poly[(best + i) % poly.len] for i in 0..poly.len]

/// Minkowski sum of two non-empty CCW convex polygons. O(|a| + |b|).
pub fn minkowski_sum(a: List<Pt>, b: List<Pt>) -> List<Pt>:
    assert a.len > 0 and b.len > 0, "minkowski_sum requires non-empty polygons"
    if a.len == 1: return [a[0] + q for q in b]
    if b.len == 1: return [p + b[0] for p in a]
    let pa = rotate_min_yx(a)
    let pb = rotate_min_yx(b)
    let na = pa.len
    let nb = pb.len
    var res: List<Pt> = []
    var i = 0
    var j = 0
    while i < na or j < nb:
        let cur = pa[i % na] + pb[j % nb]
        if res.len == 0 or res[res.len - 1] != cur: res.push(cur)
        if i == na:
            j += 1
        else:
            if j == nb:
                i += 1
            else:
                let ea = pa[(i + 1) % na] - pa[i % na]
                let eb = pb[(j + 1) % nb] - pb[j % nb]
                let c = cross128(ea, eb)
                if c > I128(0):
                    i += 1
                else:
                    if c < I128(0):
                        j += 1
                    else:
                        i += 1
                        j += 1
    while res.len > 1 and res[res.len - 1] == res[0]:
        res.pop()
    res

/// Minkowski sum of two non-empty left-to-right upper hulls. O(|a| + |b|).
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
        while res.len >= 2 and turn(res[res.len - 2], res[res.len - 1], nxt) >= 0:
            res.pop()
        if res[res.len - 1] != nxt: res.push(nxt)
    res

// ---- polygons and rotating calipers

/// Twice the signed area of a polygon: positive when vertices run CCW. O(n).
pub fn area2(poly: List<Pt>) -> Int:
    if poly.len < 3: return 0
    var total = 0
    for i in 0..poly.len: total += cross(poly[i], poly[(i + 1) % poly.len])
    total

/// Exact 128-bit twice signed area of a polygon. O(n).
pub fn area2_128(poly: List<Pt>) -> I128:
    if poly.len < 3: return I128(0)
    var total = I128(0)
    for i in 0..poly.len: total += cross128(poly[i], poly[(i + 1) % poly.len])
    total

/// Where `p` is relative to a simple polygon (either orientation):
/// 1 inside, 0 on the boundary, -1 outside. Exact in `I128`. O(n).
pub fn locate(poly: List<Pt>, p: Pt) -> Int:
    if poly.len == 0: return -1
    if poly.len == 1: return if p == poly[0]: 0 else: -1
    var inside = false
    for i in 0..poly.len:
        var (a, b) = (poly[i], poly[(i + 1) % poly.len])
        if on_segment(p, a, b): return 0
        if a.y > b.y: swap(mut a, mut b)
        if a.y <= p.y and p.y < b.y and cross128_3(p, a, b) > I128(0): inside = not inside
    if inside: 1 else: -1

/// Where `p` is relative to a CCW convex polygon:
/// 1 inside, 0 on the boundary, -1 outside. Exact in `I128`. O(log n).
pub fn locate_convex(hull: List<Pt>, p: Pt) -> Int:
    let n = hull.len
    if n == 0: return -1
    if n == 1: return if p == hull[0]: 0 else: -1
    if n == 2: return if on_segment(p, hull[0], hull[1]): 0 else: -1
    let t1 = turn(hull[0], hull[1], p)
    let t2 = turn(hull[0], hull[n - 1], p)
    if t1 < 0 or t2 > 0: return -1
    if t1 == 0: return if on_segment(p, hull[0], hull[1]): 0 else: -1
    if t2 == 0: return if on_segment(p, hull[0], hull[n - 1]): 0 else: -1
    var lo = 1
    var hi = n - 1
    while hi - lo > 1:
        let mid = (lo + hi) / 2
        if turn(hull[0], hull[mid], p) >= 0: lo = mid
        else: hi = mid
    let t = turn(hull[lo], hull[lo + 1], p)
    if t > 0: 1 else: if t == 0: 0 else: -1

/// Squared diameter of a CCW convex polygon by rotating calipers. O(n).
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

fn better_dir(a: Pt, b: Pt, dir: Pt) -> Bool:
    let da = dot128(a, dir)
    let db = dot128(b, dir)
    if da != db: return da > db
    dot128(a, dir.perp) > dot128(b, dir.perp)

/// Index of a vertex of a non-empty CCW convex polygon maximizing
/// `dot128(hull[i], dir)`. O(log n).
pub fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int:
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
        let up_lo = better_dir(hull[(lo + 1) % n], hull[lo], dir)
        let up_mid = better_dir(hull[(mid + 1) % n], hull[mid], dir)
        if not up_mid and better_dir(hull[mid], hull[(mid + n - 1) % n], dir):
            return mid
        if up_lo:
            if up_mid and better_dir(hull[mid], hull[lo], dir): lo = mid
            else: hi = mid
        else:
            if not up_mid and better_dir(hull[mid], hull[lo], dir): hi = mid
            else: lo = mid
    var best = lo
    for k in lo..hi:
        let idx = k % n
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

/// Indices `(left, right)` of the two tangent vertices from a strictly
/// exterior point `p` to a strictly convex CCW polygon `hull` (`hull.len >= 3`).
/// Every vertex `q` satisfies `turn(p, hull[left], q) <= 0` and
/// `turn(p, hull[right], q) >= 0`. O(log n).
pub fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int):
    assert hull.len >= 3, "hull_tangents requires at least 3 hull vertices"
    (left: find_tangent(hull, p, 1), right: find_tangent(hull, p, -1))

// ---- closest pair

fn cp_solve(
    mut ord: List<(p: Pt, id: Int)>,
    mut buf: List<(p: Pt, id: Int)>,
    l: Int,
    r: Int,
) -> (a: Int, b: Int, dist2: Int):
    if r - l <= 3:
        var best = Int.MAX
        var ba = ord[l].id
        var bb = ord[l + 1].id
        for i in l..r:
            for j in i + 1..r:
                let d = dist2(ord[i].p, ord[j].p)
                if d < best:
                    best = d
                    ba = ord[i].id
                    bb = ord[j].id
        // sort ord[l..r] by y
        for i in l + 1..r:
            var k = i
            while k > l and ord[k].p.y < ord[k - 1].p.y:
                let tmp = ord[k]
                ord[k] = ord[k - 1]
                ord[k - 1] = tmp
                k -= 1
        return (ba, bb, best)
    let mid = (l + r) / 2
    let mid_x = ord[mid].p.x
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
        if ord[i].p.y <= ord[j].p.y:
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
    for t in l..r:
        let dx = I128(ord[t].p.x) - I128(mid_x)
        if dx * dx < I128(best):
            buf[s_end] = ord[t]
            s_end += 1
    for si in l..s_end:
        var sj = si + 1
        while sj < s_end:
            let dy = I128(buf[sj].p.y) - I128(buf[si].p.y)
            if dy * dy >= I128(best): break
            let d = dist2(buf[si].p, buf[sj].p)
            if d < best:
                best = d
                ba = buf[si].id
                bb = buf[sj].id
            sj += 1
    (ba, bb, best)

/// Minimum squared Euclidean distance between any two distinct points in `pts`.
/// Requires `pts.len >= 2`. O(n log n).
pub fn closest_pair2(pts: List<Pt>) -> Int:
    assert pts.len >= 2, "closest_pair2 requires at least 2 points"
    var ord = [(p: pts[i], id: i) for i in 0..pts.len]
    sort(mut ord, by: q => q.p)
    for i in 1..ord.len:
        if ord[i].p == ord[i - 1].p: return 0
    var buf = copy(ord)
    cp_solve(mut ord, mut buf, 0, ord.len).dist2

/// Closest pair of distinct positions in a domain-indexed point array.
/// Requires at least 2 points. O(n log n).
pub fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int):
    let ids = [i for i in D]
    assert ids.len >= 2, "closest_pair requires at least 2 points"
    var ord = [(p: pts[ids[k]], id: k) for k in 0..ids.len]
    sort(mut ord, by: q => q.p)
    for i in 1..ord.len:
        if ord[i].p == ord[i - 1].p:
            return (ids[ord[i - 1].id], ids[ord[i].id], 0)
    var buf = copy(ord)
    let res = cp_solve(mut ord, mut buf, 0, ord.len)
    (ids[res.a], ids[res.b], res.dist2)

// ---- careful floating-point geometry

/// True when `|a - b| <= eps`. O(1).
pub fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool = abs(a - b) <= eps

/// Tolerance-aware sign: 1 if `x > eps`, -1 if `x < -eps`, else 0. O(1).
pub fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int = if x > eps: 1 else: if x < -eps: -1 else: 0

/// A 2D floating-point point or vector. Kept distinct from `Pt` so exact
/// integer predicates never silently mix with approximate arithmetic.
pub struct Vec(x: F64, y: F64):
    /// Squared Euclidean length. O(1).
    pub @get fn norm2(self) -> F64 = x * x + y * y

    /// Euclidean length. O(1).
    pub @get fn len(self) -> F64 = sqrt(x * x + y * y)

    /// Rotated 90 degrees counter-clockwise. O(1).
    pub @get fn perp(self) -> Vec = Vec(-y, x)

    /// Polar angle in `(-pi, pi]`. O(1).
    pub @get fn angle(self) -> F64 = atan2(y, x)

    /// Unit vector in the same direction. Asserts nonzero length. O(1).
    pub @get fn unit(self) -> Vec:
        let l = self.len
        assert l > 0.0, "unit of zero Vec"
        Vec(x / l, y / l)

    /// Scale by `k`. O(1).
    pub fn scale(self, k: F64) -> Vec = Vec(x * k, y * k)

    /// Divide by `k`. O(1).
    pub fn div(self, k: F64) -> Vec = Vec(x / k, y / k)

impl Add for Vec:
    fn add(a, b) = Vec(a.x + b.x, a.y + b.y)

impl Sub for Vec:
    fn sub(a, b) = Vec(a.x - b.x, a.y - b.y)

impl Neg for Vec:
    fn neg(a) = Vec(-a.x, -a.y)

/// Floating-point dot product. O(1).
pub fn vdot(a: Vec, b: Vec) -> F64 = a.x * b.x + a.y * b.y

/// Floating-point 2D cross product. O(1).
pub fn vcross(a: Vec, b: Vec) -> F64 = a.x * b.y - a.y * b.x

/// Euclidean distance between `a` and `b`. O(1).
pub fn vdist(a: Vec, b: Vec) -> F64 = (a - b).len

/// Tolerance-aware orientation of `a -> b -> c`: 1 left, -1 right, 0 straight. O(1).
pub fn vturn(
    a: Vec,
    b: Vec,
    c: Vec,
    eps: F64 = 1e-9,
) -> Int = sign_eps(vcross(b - a, c - a), eps: eps)

/// Orthogonal projection of `p` onto the line through `a` and `b`. O(1).
pub fn project(p: Vec, a: Vec, b: Vec) -> Vec:
    let ab = b - a
    a + ab.scale(vdot(p - a, ab) / ab.norm2)

/// Reflection of `p` across the line through `a` and `b`. O(1).
pub fn reflect(p: Vec, a: Vec, b: Vec) -> Vec:
    let proj = project(p, a, b)
    proj + (proj - p)

/// Perpendicular distance from `p` to the infinite line through `a` and `b`. O(1).
pub fn line_dist(p: Vec, a: Vec, b: Vec) -> F64 = abs(vcross(b - a, p - a)) / (b - a).len

/// Shortest distance from `p` to the closed segment `a b`. O(1).
pub fn seg_dist(p: Vec, a: Vec, b: Vec) -> F64:
    let ab = b - a
    let n2 = ab.norm2
    if n2 == 0.0: return (p - a).len
    let t = vdot(p - a, ab) / n2
    if t <= 0.0: return (p - a).len
    if t >= 1.0: return (p - b).len
    (p - (a + ab.scale(t))).len

/// Intersection of the infinite lines `a b` and `c d`, or `none` when parallel. O(1).
pub fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec:
    let ab = b - a
    let cd = d - c
    let den = vcross(ab, cd)
    if abs(den) <= eps: return none
    some(a + ab.scale(vcross(c - a, cd) / den))

/// Circumcenter of triangle `(a, b, c)`, or `none` when collinear. O(1).
pub fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec:
    let u = b - a
    let v = c - a
    let den = 2.0 * vcross(u, v)
    if abs(den) <= eps: return none
    let u2 = u.norm2
    let v2 = v.norm2
    some(a + Vec(v.y * u2 - u.y * v2, u.x * v2 - v.x * u2).div(den))

/// Intersection points (0, 1, or 2) of the circle `(center, r)` with the
/// infinite line through `a` and `b`. O(1).
pub fn circle_line_intersect(center: Vec, r: F64, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>:
    let h = project(center, a, b)
    let d2 = (center - h).norm2
    let r2 = r * r
    if d2 > r2 + eps: return []
    let rem = r2 - d2
    if rem <= eps: return [h]
    let dir = (b - a).unit.scale(sqrt(rem))
    [h - dir, h + dir]

/// Intersection points (0, 1, or 2) of two circles `(c1, r1)` and `(c2, r2)`. O(1).
pub fn circle_intersect(c1: Vec, r1: F64, c2: Vec, r2: F64, eps: F64 = 1e-9) -> List<Vec>:
    let d_vec = c2 - c1
    let d = d_vec.len
    if d <= eps: return []
    if d > r1 + r2 + eps or d + min(r1, r2) + eps < max(r1, r2): return []
    let x = (d * d + r1 * r1 - r2 * r2) / (2.0 * d)
    let h2 = r1 * r1 - x * x
    let base = c1 + d_vec.scale(x / d)
    if h2 <= eps: return [base]
    let off = d_vec.perp.scale(sqrt(h2) / d)
    [base - off, base + off]

// ---- half-plane intersection

/// Directed line through `p` in direction `dir`, keeping the closed left
/// half-plane `{ q | vcross(dir, q - p) >= 0 }`.
pub struct HalfPlane(p: Vec, dir: Vec)

fn hp_out(
    h: (p: Vec, dir: Vec, ang: F64),
    pt: Vec,
    eps: F64,
) -> Bool = vcross(h.dir, pt - h.p) < -eps

fn hp_cross(a: (p: Vec, dir: Vec, ang: F64), b: (p: Vec, dir: Vec, ang: F64)) -> Vec:
    let den = vcross(a.dir, b.dir)
    a.p + a.dir.scale(vcross(b.p - a.p, b.dir) / den)

/// Counter-clockwise polygon of the bounded intersection of `planes`, or `[]`
/// when empty or unbounded. Add a bounding box for unbounded regions. O(n log n).
pub fn half_plane_intersect(planes: List<HalfPlane>, eps: F64 = 1e-9) -> List<Vec>:
    var norm: List<(p: Vec, dir: Vec, ang: F64)> = []
    for pl in planes:
        let l = pl.dir.len
        if l <= eps: continue
        let d = pl.dir.div(l)
        norm.push((p: pl.p, dir: d, ang: d.angle))
    if norm.len < 3: return []
    sort(mut norm, by: h => h.ang)
    // Deduplicate parallel half-planes with the same direction, keeping the leftmost.
    var uniq: List<(p: Vec, dir: Vec, ang: F64)> = []
    for h in norm:
        if uniq.len > 0:
            let prev = uniq[uniq.len - 1]
            if abs(vcross(prev.dir, h.dir)) <= eps and vdot(prev.dir, h.dir) > 0.0:
                if vcross(prev.dir, h.p - prev.p) > 0.0:
                    uniq[uniq.len - 1] = h
                continue
        uniq.push(h)
    // Also check wrap-around between last and first (angles near -pi and +pi).
    if uniq.len >= 2:
        let first_h = uniq[0]
        let last_h = uniq[uniq.len - 1]
        if abs(vcross(first_h.dir, last_h.dir)) <= eps and vdot(first_h.dir, last_h.dir) > 0.0:
            if vcross(first_h.dir, last_h.p - first_h.p) > 0.0:
                uniq[0] = last_h
            uniq.pop()
    var dq: List<(p: Vec, dir: Vec, ang: F64)> = []
    var head = 0
    for h in uniq:
        while dq.len - head >= 2 and hp_out(h, hp_cross(dq[dq.len - 2], dq[dq.len - 1]), eps):
            dq.pop()
        while dq.len - head >= 2 and hp_out(h, hp_cross(dq[head], dq[head + 1]), eps):
            head += 1
        if dq.len - head >= 1 and vcross(dq[dq.len - 1].dir, h.dir) <= eps:
            return []
        dq.push(h)
    while dq.len - head >= 3 and hp_out(dq[head], hp_cross(dq[dq.len - 2], dq[dq.len - 1]), eps):
        dq.pop()
    while dq.len - head >= 3 and hp_out(dq[dq.len - 1], hp_cross(dq[head], dq[head + 1]), eps):
        head += 1
    let m = dq.len - head
    if m < 3: return []
    var poly: List<Vec> = []
    for i in 0..m:
        let a = dq[head + i]
        let b = dq[head + (i + 1) % m]
        if vcross(a.dir, b.dir) <= eps: return []
        poly.push(hp_cross(a, b))
    poly

/// Signed area of a floating-point polygon (positive for CCW, negative for CW). O(n).
pub fn polygon_area(poly: List<Vec>) -> F64:
    let n = poly.len
    if n < 3: return 0.0
    var s = 0.0
    for i in 0..n:
        s += vcross(poly[i], poly[(i + 1) % n])
    0.5 * s

/// Clip `poly` to the closed left half-plane of directed line `a -> b`. O(n).
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
            out.push(cur + (nxt - cur).scale(t))
    out

/// Smallest enclosing circle of `pts` via deterministic Welzl's algorithm.
/// Expected O(n). Returns `(center: Vec(0.0, 0.0), radius: 0.0)` for empty input.
pub fn min_enclosing_circle(pts: List<Vec>, eps: F64 = 1e-9) -> (center: Vec, radius: F64):
    let n = pts.len
    if n == 0: return (center: Vec(0.0, 0.0), radius: 0.0)
    if n == 1: return (center: pts[0], radius: 0.0)
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
                    c = (ps[i] + ps[j]).scale(0.5)
                    r = vdist(c, ps[i])
                    for k in 0..j:
                        if vdist(c, ps[k]) > r + eps:
                            if let o = circumcenter(ps[i], ps[j], ps[k], eps: eps):
                                c = o
                                r = vdist(c, ps[i])
    (center: c, radius: r)

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
    let foot = a + d.scale(t_mid)
    let h2 = foot.norm2
    if h2 >= r2 - eps:
        return 0.5 * r2 * atan2(s, vdot(a, b))
    let dt = sqrt(max(0.0, r2 - h2) / a2)
    let t1 = max(0.0, min(1.0, t_mid - dt))
    let t2 = max(0.0, min(1.0, t_mid + dt))
    let p1 = a + d.scale(t1)
    let p2 = a + d.scale(t2)
    var res = 0.0
    if t1 > eps: res += 0.5 * r2 * atan2(vcross(a, p1), vdot(a, p1))
    res += 0.5 * vcross(p1, p2)
    if 1.0 - t2 > eps: res += 0.5 * r2 * atan2(vcross(p2, b), vdot(p2, b))
    res

/// Signed area of intersection of disk `(c, r)` with polygon `poly`. O(n).
pub fn circle_polygon_area(c: Vec, r: F64, poly: List<Vec>, eps: F64 = 1e-9) -> F64:
    let n = poly.len
    if n < 3 or r <= eps: return 0.0
    var total = 0.0
    for i in 0..n:
        total += tri_circle_area(poly[i] - c, poly[(i + 1) % n] - c, r, eps)
    total

// ---- tests

fn random_points(n: Int, span: Int) -> List<Pt> = [
    Pt(rand(-span, span), rand(-span, span)) for _ in 0..n
]

@test
fn vectors_traits_and_large_coords():
    let (a, b) = (Pt(3, 4), Pt(-1, 2))
    assert a + b == Pt(2, 6) and a - b == Pt(4, 2) and -a == Pt(-3, -4)
    assert a.scale(3) == Pt(9, 12)
    assert dot(a, b) == 5 and cross(a, b) == 10 and a.norm2 == 25 and close(a.len, 5.0)
    assert a.perp == Pt(-4, 3) and a.perp_cw == Pt(4, -3) and dot(a, a.perp) == 0
    assert Pt(6, -9).unit_dir == Pt(2, -3) and Pt(0, 0).unit_dir == Pt(0, 0)
    assert same_dir(Pt(2, 4), Pt(3, 6)) and not same_dir(Pt(2, 4), Pt(-2, -4))
    assert "{a}" == "3 4"
    // Ord and Map/Set support
    var pts = [Pt(2, 1), Pt(1, 5), Pt(1, 2)]
    sort(mut pts)
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
    let base = Pt(10, -5)
    var shifted = [base + d for d in dirs]
    shuffle(mut shifted)
    sort(mut shifted, by: p => ByAngle(p - base))
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
                assert locate(h, p) >= 0
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
        sort(mut sa)
        sort(mut sb)
        let ua = upper_hull(sa)
        let ub = upper_hull(sb)
        let mu = minkowski_upper(ua, ub)
        var all_sums = [p + q for p in ua for q in ub]
        sort(mut all_sums)
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
            let want = if min(turns)! > 0: 1 else: if min(turns)! == 0: 0 else: -1
            assert locate(poly, p) == want
            assert locate(poly.rev, p) == want
            assert locate_convex(poly, p) == want
    // concave L shape
    let l = [Pt(0, 0), Pt(4, 0), Pt(4, 2), Pt(2, 2), Pt(2, 4), Pt(0, 4)]
    assert area2(l) == 24
    assert locate(l, Pt(1, 3)) == 1 and locate(l, Pt(3, 3)) == -1 and locate(l, Pt(2, 3)) == 0
    assert locate(l, Pt(3, 1)) == 1 and locate(l, Pt(5, 2)) == -1 and locate(l, Pt(-1, 0)) == -1
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
                let best_dot = max(dot128(p, dir) for p in h)!
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
    assert line_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == none
    // circumcenter equidistant from all 3 vertices
    let cc = circumcenter(Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(0.0, 3.0))!
    assert close(cc.x, 2.0) and close(cc.y, 1.5)
    assert close(vdist(cc, Vec(0.0, 0.0)), vdist(cc, Vec(4.0, 0.0)))
    assert close(vdist(cc, Vec(0.0, 0.0)), vdist(cc, Vec(0.0, 3.0)))
    // circle-line and circle-circle intersections
    let cl = circle_line_intersect(Vec(0.0, 0.0), 5.0, Vec(-10.0, 3.0), Vec(10.0, 3.0))
    assert cl.len == 2
    for q in cl:
        assert close(q.len, 5.0) and close(q.y, 3.0)
    let cc_pts = circle_intersect(Vec(0.0, 0.0), 5.0, Vec(6.0, 0.0), 5.0)
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
    var area = 0.0
    for i in 0..poly.len:
        area += vcross(poly[i], poly[(i + 1) % poly.len])
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
            var dy = F64(rand(-5, 5))
            if dx == 0.0 and dy == 0.0: dx = 1.0
            hps.push(HalfPlane(Vec(px, py), Vec(dx, dy)))
        let res = half_plane_intersect(hps)
        if res.len >= 3:
            for v in res:
                for hp in hps:
                    assert vcross(hp.dir, v - hp.p) >= -1e-6 * hp.dir.len
            var a2 = 0.0
            for i in 0..res.len:
                a2 += vcross(res[i], res[(i + 1) % res.len])
            assert a2 >= -1e-6

@test
fn hull_extreme_and_locate_convex_edge_cases():
    // degenerate 0/1/2-point hulls
    assert locate_convex([], Pt(0, 0)) == -1
    assert locate_convex([Pt(1, 2)], Pt(1, 2)) == 0
    assert locate_convex([Pt(1, 2)], Pt(1, 3)) == -1
    assert locate_convex([Pt(0, 0), Pt(4, 0)], Pt(2, 0)) == 0
    assert locate_convex([Pt(0, 0), Pt(4, 0)], Pt(2, 1)) == -1
    // hull_extreme on collinear: true hulls
    for _ in 0..40:
        let pts = random_points(20, 4)
        let h = convex_hull(pts, collinear: true)
        if h.len > 0:
            for _ in 0..10:
                let dir = Pt(rand(-4, 4), rand(-4, 4))
                let idx = hull_extreme(h, dir)
                let best_dot = max(dot128(p, dir) for p in h)!
                assert dot128(h[idx], dir) == best_dot

@test
fn hull_tangents_matches_brute():
    for _ in 0..50:
        let h = convex_hull(random_points(24, 8))
        if h.len < 3: continue
        for _ in 0..15:
            let p = Pt(rand(-15, 15), rand(-15, 15))
            if locate_convex(h, p) >= 0: continue
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
    assert min_enclosing_circle([]).radius == 0.0
    assert close(min_enclosing_circle([Vec(3.0, 4.0)]).radius, 0.0)
    let sq = [Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]
    let mc = min_enclosing_circle(sq)
    assert close(mc.center.x, 1.0) and close(mc.center.y, 1.0) and close(mc.radius, sqrt(2.0))
    // Randomized stress against brute-force over all pairs & triples
    for _ in 0..25:
        let pts = [Vec(F64(rand(-10, 10)), F64(rand(-10, 10))) for _ in 0..10]
        let got = min_enclosing_circle(pts)
        for p in pts:
            assert vdist(got.center, p) <= got.radius + 1e-6
        var best_r = 1.0e18
        for i in 0..pts.len:
            for j in i + 1..pts.len:
                let c2 = (pts[i] + pts[j]).scale(0.5)
                let r2 = vdist(c2, pts[i])
                if all(vdist(c2, q) <= r2 + 1e-7 for q in pts): best_r = min(best_r, r2)
                for k in j + 1..pts.len:
                    if let c3 = circumcenter(pts[i], pts[j], pts[k]):
                        let r3 = vdist(c3, pts[i])
                        if all(vdist(c3, q) <= r3 + 1e-7 for q in pts): best_r = min(best_r, r3)
        if best_r < 1.0e17:
            assert close(got.radius, best_r, eps: 1e-5)
    // Circle-polygon intersection area
    let pi = 4.0 * atan2(1.0, 1.0)
    // Disk radius 0.5 strictly inside [0,2]^2 -> area = pi * 0.25
    assert close(circle_polygon_area(Vec(1.0, 1.0), 0.5, sq), pi * 0.25, eps: 1e-6)
    // Square strictly inside disk radius 5 -> area = 4.0
    assert close(circle_polygon_area(Vec(1.0, 1.0), 5.0, sq), 4.0, eps: 1e-6)
    // Quarter disk at corner (0, 0) with radius 1 -> area = pi / 4
    assert close(circle_polygon_area(Vec(0.0, 0.0), 1.0, sq), pi * 0.25, eps: 1e-6)
    // Disk centered at (1, 0) with radius 1 -> half disk = pi / 2
    assert close(circle_polygon_area(Vec(1.0, 0.0), 1.0, sq), pi * 0.5, eps: 1e-6)
