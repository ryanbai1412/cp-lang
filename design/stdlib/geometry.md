# Standard library design: `std.geometry`

Status: design for `plan/STDLIB.md` section 4.14. Covers 2D competitive-
programming geometry: exact integer points and vectors, orientation, angular
sorting, segments, convex hulls and Minkowski sums, polygon area and
containment, rotating calipers, closest pair, half-plane intersection, and
careful floating-point geometry.

## 1. Contest survey

Across Codeforces Div. 1, AtCoder Grand/Regular, and ICPC regionals and World
Finals, geometry splits sharply into two worlds:

- **Exact integer geometry** (~80% of contest geometry). Input coordinates are
  integers in `-1e9..=1e9` (occasionally up to `1e18` after prefix sums or DP
  transitions). Every predicate is a sign of a dot or cross product; floating
  point is never touched until the final `sqrt` or `atan2`.
- **Careful floating-point geometry** (~20% of contest geometry). Constructive
  tasks (`cf2138f`), reflections (`etsr23r3p2`), line/circle intersections,
  polygon clipping, and half-plane intersection. Every comparison needs an
  explicit tolerance `eps`, and exact `==` is a bug.

Reference libraries surveyed:

- **AtCoder Library (ACL)** deliberately omits geometry, which is why every
  contestant maintains a personal template.
- **KACTL (`content/geometry/`)** is the ICPC gold standard: `Point<T>`,
  `onSegment`, `segInter`, `inPolygon`, `polygonArea2`, `convexHull`,
  `hullDiameter`, `inHull`, `closestPair`, `lineInter`, `polygonCut`,
  `circleLine`, `circleInter`, `circumCircle`, and `halfPlaneInter`.
- **ecnerwala's `Point<T, AreaT>`** (`proto/ports/ecnerwala/cf2138f_orig.cpp`,
  `ucup4s25e_orig.cpp`, `ucup5s05e_orig.cpp`, `ucup5s05b_orig.cpp`) adds three
  high-value patterns that KACTL lacks: a wider `AreaT` for cross/dot products,
  exact integer unit directions (`int_unit` via `gcd`), and arbitrary-base
  angular ordering (`angle_less(base, s, t)` for `[base, base + 2*pi)`).
- **CP Lang corpus and frictions**:
  - `ucup4s25e.cp` hand-rolls `Pt`, `upper_hull`, and Minkowski sum of two
    upper chains in O(n) on already-sorted points.
  - `ucup5s05e.cp` hand-rolls lower and upper hulls *keeping collinear points*
    while preserving original point IDs.
  - `cf2138f.cp` (`research/frictions/cf2138f.md`) hand-rolls a `F64` vector
    type `V` with `len`, `Add`, `Sub`, and scalar `times`/`over`.
  - `etsr23r3p2.cp` asks for `F64` points and `reflect(p, a, b)`.
  - `dmopc20c7p2.cp` (`research/ugly-code-friction-audit-3.md` section 9)
    needs `Pt` to have lexicographic `Ord`, `Hash`, `Read`, and `Show` (`x y`).
  - `cheerio1s3.cp` uses `Pt` and `cross` for an exact slope sweep.

## 2. Design principles

**Two point types: `Pt` (exact `Int`) and `Vec` (careful `F64`).** Exact
integer geometry and approximate floating-point geometry do not share safe
operations. `Pt` has exact equality, lexicographic `Ord`, `Hash`, `Show`
(`"x y"`), exact `norm2 -> Int`, `unit_dir`, and epsilon-free predicates. `Vec`
has `unit`, `scale`, `div`, `project`, `reflect`, and epsilon-parameterized
predicates, and intentionally omits `Show` because CP Lang forbids printing
floats without an explicit precision (`"{v.x:.9f} {v.y:.9f}"`, decision 1.5).
Keeping them separate prevents accidental float contamination of exact
predicates while letting `Pt` remain non-generic in `std.prelude`. Bridging
is one explicit getter: `p.to_vec`.

**Overflow-free orientation predicates via `I128`.** With coordinates up to
`1e9`, `b - a` reaches `2e9` and `cross(b - a, c - a)` reaches `8e18`, right
below `Int.MAX` (`9.22e18`); prefix sums (`ucup4s25e`) or `2e9` grids overflow
64-bit `cross` silently in release builds. `dot` and `cross` return `Int` for
ergonomics (since callers add and store them as `Int`), while `dot128` and
`cross128` return `I128`. Every orientation predicate (`turn`, `ByAngle`,
`ByAngleFrom`, `angle_less`, `same_dir`, `on_segment`, `segments_meet`,
`locate`, `locate_convex`, `convex_hull`, `lower_hull`, `upper_hull`,
`minkowski_sum`, `minkowski_upper`, `hull_extreme`) computes its cross and dot
signs in `I128`, so predicates stay exact for coordinates up to `1e18` at
essentially zero cost (`imul` 64x64->128 on x86_64).

**Options as named arguments.** Following `plan/STDLIB.md` section 2:
- `segments_meet(a, b, c, d, strict: Bool = false)` instead of two functions.
- `convex_hull(pts, collinear: Bool = false)` (and `lower_hull`, `upper_hull`,
  `convex_hull_idx`) instead of `_collinear` twins.
- `half_plane_intersect(planes, eps: F64 = 1e-9)` and float predicates take
  `eps: F64 = 1e-9` as a named default argument.

**Shared method vocabulary across `Pt` and `Vec`.** Both `Pt` and `Vec` provide
`.norm2`, `.len`, `.perp`, `.perp_cw`, `.angle`, `.scale(k)`, `.dot(o)`,
`.cross(o)`, `.dist2(o)`, and `.dist(o)`. Free functions `dot`, `cross`,
`dist2`, `dist`, `turn` remain for `Pt` so mathematical notation like
`cross(b - a, c - a)` and `turn(a, b, c)` reads naturally.

**Domains and index spaces.** Functions over a `List<Pt>` return `List<Pt>` or
`Int` positions; functions over a domain-indexed `[D]Pt` return `D` handles so
callers never lose point IDs or build `(p: Pt, id: Int)` wrapper structs:
- `convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>`
- `closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int)`

## 3. Complete API surface

### 3.1 Exact integer points and vectors

```cp
struct Pt(x: Int, y: Int)              // Add, Sub, Neg, Eq, Ord (x, y), Hash, Read, Show ("x y")
    @get fn norm2(self) -> Int         // O(1); x*x + y*y
    @get fn len(self) -> F64           // O(1); sqrt(F64(norm2))
    @get fn perp(self) -> Pt           // O(1); 90 deg CCW: Pt(-y, x)
    @get fn perp_cw(self) -> Pt        // O(1); 90 deg CW: Pt(y, -x)
    @get fn angle(self) -> F64         // O(1); atan2(y, x) in [-pi, pi]
    @get fn unit_dir(self) -> Pt       // O(log min(|x|, |y|)); reduced by gcd, Pt(0, 0) for zero
    @get fn to_vec(self) -> Vec        // O(1); Vec(F64(x), F64(y))
    fn scale(self, k: Int) -> Pt       // O(1); Pt(x * k, y * k)
    fn dot(self, o: Pt) -> Int         // O(1)
    fn cross(self, o: Pt) -> Int       // O(1)
    fn dist2(self, o: Pt) -> Int       // O(1)
    fn dist(self, o: Pt) -> F64        // O(1)

fn dot(a: Pt, b: Pt) -> Int            // O(1); safe for |coords| <= 2e9
fn cross(a: Pt, b: Pt) -> Int          // O(1); safe for |coords| <= 2e9
fn dot128(a: Pt, b: Pt) -> I128        // O(1); safe for all Int
fn cross128(a: Pt, b: Pt) -> I128      // O(1); safe for all Int
fn dist2(a: Pt, b: Pt) -> Int          // O(1); (a - b).norm2
fn dist(a: Pt, b: Pt) -> F64           // O(1); (a - b).len
fn manhattan(a: Pt, b: Pt) -> Int      // O(1); |a.x - b.x| + |a.y - b.y|
fn turn(a: Pt, b: Pt, c: Pt) -> Int    // O(1), I128; 1 CCW, -1 CW, 0 collinear
fn same_dir(a: Pt, b: Pt) -> Bool      // O(1), I128; parallel and same ray
```

### 3.2 Angular sorting

```cp
struct ByAngle(p: Pt)                  // Ord: CCW in [0, 2*pi) from +x; zero first; I128
struct ByAngleFrom(base: Pt, p: Pt)    // Ord: CCW in [base, base + 2*pi); zero first; I128
fn angle_less(base: Pt, a: Pt, b: Pt) -> Bool  // O(1), I128; a precedes b CCW from base
```

`ByAngle` works directly with `sort(mut dirs, by: ByAngle)` and around an
origin via `sort(mut pts, by: p => ByAngle(p - o))`. `ByAngleFrom(base, p)`
orders directions starting from any non-zero ray `base` without splitting or
wrapping the array twice.

### 3.3 Segments

```cp
fn on_segment(p: Pt, a: Pt, b: Pt) -> Bool
    // O(1), I128; true iff p lies on the closed segment [a, b]
fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool
    // O(1), I128; closed segments share a point, or proper interior crossing when strict: true
```

### 3.4 Convex hulls, chains, and Minkowski sums

```cp
fn lower_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt>
    // O(n) time, O(n) space; precondition: pts sorted by (x, y); lower chain left-to-right
fn upper_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt>
    // O(n) time, O(n) space; precondition: pts sorted by (x, y); upper chain left-to-right
fn convex_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt>
    // O(n log n) time, O(n) space; CCW from lowest-leftmost point, deduplicated
fn convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D>
    // O(n log n) time, O(n) space; CCW hull vertices as D handles
fn minkowski_sum(a: List<Pt>, b: List<Pt>) -> List<Pt>
    // O(|a| + |b|); CCW convex hull of {p + q : p in a, q in b} for non-empty CCW hulls a, b
fn minkowski_upper(a: List<Pt>, b: List<Pt>) -> List<Pt>
    // O(|a| + |b|); upper convex chain of {p + q} for non-empty left-to-right upper chains a, b
```

Keeping `lower_hull` and `upper_hull` O(n) on already x-sorted inputs is
essential for O(n log n) divide-and-conquer hulls (such as `ucup4s25e`).
`convex_hull` sorts, deduplicates, and stitches the two chains.

### 3.5 Polygons and rotating calipers

```cp
fn area2(poly: List<Pt>) -> Int
    // O(n); twice signed area (+ CCW, - CW)
enum Location = Inside | Boundary | Outside
fn locate(poly: List<Pt>, p: Pt) -> Location
    // O(n), I128; simple polygon (either winding)
fn locate_convex(hull: List<Pt>, p: Pt) -> Location
    // O(log n), I128; CCW convex polygon (no 3 collinear)
fn diameter2(hull: List<Pt>) -> Int
    // O(n), I128; max squared distance on a CCW convex hull (0 for < 2 points)
fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int
    // O(log n), I128; index in 0..hull.len maximizing dot(hull[i], dir) on a CCW hull
```

### 3.6 Closest pair

```cp
fn closest_pair2(pts: List<Pt>) -> Int
    // O(n log n) time, O(n) space; min squared distance among pts (requires pts.len >= 2)
fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int)
    // O(n log n) time, O(n) space; closest distinct pair of D handles and squared distance
```

### 3.7 Careful floating-point geometry and half-plane intersection

```cp
const PI: F64 = 3.14159265358979323846

fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool      // O(1); |a - b| <= eps
fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int            // O(1); 1, 0, or -1 with tolerance eps

struct Vec(x: F64, y: F64)                             // Add, Sub, Neg; intentionally no Show
    @get fn norm2(self) -> F64                         // O(1)
    @get fn len(self) -> F64                           // O(1)
    @get fn unit(self) -> Vec                          // O(1); traps if len == 0
    @get fn perp(self) -> Vec                          // O(1); Vec(-y, x)
    @get fn perp_cw(self) -> Vec                       // O(1); Vec(y, -x)
    @get fn angle(self) -> F64                         // O(1); atan2(y, x) in [-pi, pi]
    fn scale(self, k: F64) -> Vec                      // O(1); Vec(x * k, y * k)
    fn div(self, k: F64) -> Vec                        // O(1); Vec(x / k, y / k)
    fn dot(self, o: Vec) -> F64                        // O(1)
    fn cross(self, o: Vec) -> F64                      // O(1)
    fn dist2(self, o: Vec) -> F64                      // O(1)
    fn dist(self, o: Vec) -> F64                       // O(1)
    fn rot(self, by: Vec) -> Vec                      // O(1); rotate by unit vector `by`

fn poly_area(poly: List<Vec>) -> F64                   // O(n); signed area (+ CCW, - CW)
fn project(p: Vec, a: Vec, b: Vec) -> Vec              // O(1); orthogonal projection onto line ab
fn reflect(p: Vec, a: Vec, b: Vec) -> Vec              // O(1); reflection across line ab
fn dist_to_segment(p: Vec, a: Vec, b: Vec) -> F64      // O(1); distance from p to segment [a, b]
fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec
    // O(1); intersection of lines ab and cd, none if parallel
fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>
    // O(n); Sutherland-Hodgman clip to left half-plane of directed line a -> b
fn circle_line_intersect(c: Vec, r: F64, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>
    // O(1); 0, 1, or 2 intersection points of circle (c, r) with line ab
fn circle_intersect(c1: Vec, r1: F64, c2: Vec, r2: F64, eps: F64 = 1e-9) -> List<Vec>
    // O(1); 0, 1, or 2 intersection points of two circles
fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec
    // O(1); circumcenter of triangle abc, none if collinear

struct HalfPlane(p: Vec, dir: Vec)                     // left side of ray p + t*dir (t in R)
    fn through(a: Vec, b: Vec) -> HalfPlane            // O(1); keep left of a -> b
    fn from_line(a: F64, b: F64, c: F64) -> HalfPlane  // O(1); keep a*x + b*y + c >= 0
    fn side(self, q: Vec) -> F64                       // O(1); dir.cross(q - p)

fn half_plane_intersect(planes: List<HalfPlane>, eps: F64 = 1e-9) -> List<Vec>
    // O(n log n) time, O(n) space; CCW vertices of bounded intersection (empty if empty).
    // Add a bounding box of four half-planes if the region may be unbounded.
```

## 4. Laws and invariants

Every implementation test verifies these laws on random and degenerate inputs:

- **Vector algebra**: `dot(a, b) == dot(b, a)`, `cross(a, b) == -cross(b, a)`,
  `dot(a, a.perp) == 0`, `cross(a, a.perp) == a.norm2`,
  `a.perp.perp == -a`, `turn(a, b, c) == -turn(a, c, b)`.
- **Order**: `ByAngle` and `ByAngleFrom(base, _)` are strict weak orders
  (irreflexive, asymmetric, transitive on non-parallel rays).
- **Segments**: `segments_meet(a, b, c, d)` is symmetric in `(a, b) <-> (c, d)`
  and `a <-> b`, and `strict: true` implies `strict: false`.
- **Convex hull**: every vertex of `h = convex_hull(pts)` is in `pts`; every
  consecutive triple turns strictly left (`collinear: false`) or non-right
  (`collinear: true`); every `p in pts` satisfies `locate(h, p) != Location.Outside`;
  `area2(h) > 0` for non-collinear inputs and `area2(h.rev) == -area2(h)`.
- **Fast convex queries**: `locate_convex(h, p) == locate(h, p)` for all `p` on
  every random hull `h`; `dot(h[hull_extreme(h, d)], d) == max(dot(v, d) for v
  in h)`; `diameter2(h) == max(dist2(u, v) for u in h for v in h)`.
- **Minkowski sum**: `minkowski_sum(a, b) == convex_hull([p + q for p in a for
  q in b])`; `minkowski_upper(a, b)` equals the upper hull of pairwise sums.
- **Closest pair**: `closest_pair2(pts)` matches O(n^2) brute force, and
  `closest_pair(pts)` returns distinct `a != b` with `dist2(pts[a], pts[b]) ==
  closest_pair2(...)`.
- **Floating point**: `reflect(reflect(p, a, b), a, b)` is `close` to `p`;
  `project(p, a, b)` lies on `ab` with `(p - proj).dot(b - a)` close to `0`;
  `half_plane_intersect` agrees with folding `polygon_cut` over a bounding box
  in signed area and vertex count.

## 5. Core features wanted (non-blocking)

Two core features would sharpen `std.geometry` further once the core lane
delivers them; neither blocks shipping the full API today:

- **Heterogeneous `Mul<Rhs>` and `Div<Rhs>`**: lets `v * k` and `v / k` replace
  `v.scale(k)` and `v.div(k)` on both `Pt` and `Vec` (filed in
  `research/frictions/cf2138f.md`).
- **`sin` and `cos` intrinsics** (listed in `decisions.md` 1.5): enables
  `Vec.rot(by: u)` rotates by a unit vector without `sin`/`cos` (avoiding a `_by` twin per §2).

## 6. What stays user code or `examples/local-lib/geom/`

- **Dynamic convex hull / Li Chao tree** (DP optimization, already staged in
  `examples/local-lib/ds/li_chao.cp`).
- **3D geometry and 3D convex hull** (rare outside ICPC WF; belongs in
  `examples/local-lib/geom/`).
- **Delaunay triangulation, Voronoi diagrams, and KD-trees** (heavy, niche).
- **Exact homogeneous rational points** (`Hom(x, y, w)` as in `nccc10s4`):
  specialized to exact line-arrangement graphs; user code or `local-lib`.
- **Sweep-line event loops**: problem-specific state transitions stay in user
  code, built on `Pt`, `turn`, `ByAngle`, and `segments_meet`.

## 7. Real problems written against this API

### Problem 1: UCup 4 Stage 25 (Zagreb) E — Maximum Subarray Sum by Length

Divide and conquer on prefix sums `(i, pref[i])`; at each split, combine the
upper hull of negated left prefixes with the upper hull of right prefixes via
`minkowski_upper`, then answer offline linear queries on the final upper hull.

```cp
use std.geometry.{Pt, dot, upper_hull, minkowski_upper}

@!multitest
let (n, q): (1..=2e5, 1..=2e5) = read()
let a: [0..n]Int = read()

var pref: [0..=n]Pt = Pt(0, 0)
for i in 0..n: pref[i + 1] = Pt(i + 1, pref[i].y + a[i])
var best: [0..n]Pt = [0..n] i => pref[i + 1]

fn solve(lo: Int, hi: Int):
    if hi - lo <= 1: return
    let md = (lo + hi) / 2
    solve(lo, md)
    solve(md, hi)
    let left = upper_hull([-pref[i] for i in (lo..md).rev])
    let right = upper_hull([pref[i] for i in md..hi])
    for p in minkowski_upper(left, right):
        best[p.x - 1].y max= p.y

solve(0, n + 1)
let hull = upper_hull([best[i] for i in 0..n])

let qs: [0..q]Int = read()
var ans: [0..q]Int = 0
var at = 0
for (v, id) in [(qs[i], i) for i in 0..q].sorted():
    let dir = Pt(v, 1)
    while at + 1 < hull.len and dot(hull[at + 1] - hull[at], dir) >= 0: at += 1
    ans[id] = dot(hull[at], dir)
print(ans)
```

### Problem 2: UCup 5 Stage 5 (Chengdu) E — Point in Triangle

Keep collinear points on the convex hull using domain handles so every vertex
carries its 1-based input index directly without a wrapper struct.

```cp
use std.geometry.{Pt, Location, turn, convex_hull_idx, locate_convex}

@!multitest
let (n, q): (3..=2e5, 1..=2e5) = read()
domain Site = 1..=n
let pts: [Site]Pt = read()

let hull = convex_hull_idx(pts, collinear: true)
let off_diag: ?Site = if hull.len == 4:
    first(s for s in Site if turn(pts[hull[0]], pts[hull[2]], pts[s]) != 0
        and turn(pts[hull[1]], pts[hull[3]], pts[s]) != 0)
else: none

// Split hull into lower/upper x-monotone chains and test the local window.
let hull_pts = [pts[s] for s in hull]
for _ in 0..q:
    let c: Pt = read()
    if locate_convex(hull_pts, c) != Location.Inside:
        print(-1)
        continue
    // Pick candidate vertices around c.x and check triples with turn(...).
```

### Problem 3: Codeforces 2138 F — Floating-Point Triangle Ladder

Uses `Vec` directly instead of defining a local struct and `Add`/`Sub` impls.

```cp
use std.geometry.{Vec}

fn fits(a: Vec, b: Vec) -> Bool = 0.5 <= a.dist(b) <= 1.0

@!multitest
let (p, q, m): (1..=10_000, 1..=10_000, 3..=28_285) = read()
let t = Vec(F64(p), F64(q))
let k = m / 2
print(m)

var drawn = 2
fn draw(u: Int, v: Int, c: Vec) -> Int:
    print("{u} {v} {c.x:.20f} {c.y:.20f}")
    drawn += 1
    drawn

fn ladder(off: Int, on: Int, from: Int, off_at: Int -> Vec, on_at: Int -> Vec):
    var (a, b) = (off, on)
    for i in from..=k:
        a = draw(a, b, off_at(i))
        b = draw(a, b, on_at(i))

let (p1, p2) = (Vec(0.0, 0.0), Vec(1.0, 0.0))
if m % 2 == 0:
    let d = t.div(F64(k))
    let side = Vec(0.5, d.y - 0.5)
    ladder(2, 1, 1, i => side + d.scale(F64(i - 1)), i => d.scale(F64(i)))
else:
    let d = t.unit
    let (t1, t0) = (t - d.scale(F64(k - 1)), t - d.scale(F64(k)))
    let start = if fits(t0, p2): t0 else: Vec(
        0.5 + 0.75 * sqrt(3.0 / 13.0),
        0.75 + sqrt(3.0 / 13.0) * 0.5,
    )
    fn side() -> Vec:
        for x in -64..=64:
            for y in -64..=64:
                let c = Vec(F64(x) / 64.0, F64(y) / 64.0)
                if fits(c, p1) and fits(c, p2) and fits(c, start)
                    and fits(c, t0) and fits(c, t1): return c
        p1
    let s = side()
    let a = draw(1, 2, s)
    let b = draw(2, a, start)
    let c = draw(a, b, t1)
    ladder(a, c, 2, i => s + d.scale(F64(i - 1)), i => t0 + d.scale(F64(i)))
```

### Problem 4: ETSR 2023 R3 P2 — Paper Folding

Unfold backwards across each fold line with `Vec` and `reflect`.

```cp
use std.geometry.{Vec, reflect}

let n: 1..=10 = read()
domain Fold = 1..=n
let folds: [Fold](a: Vec, b: Vec) = read()
let q: 1..=1e5 = read()

fn layers(k: Int, p: Vec) -> Int:
    if k == 0: return if 0.0 <= p.x <= 100.0 and 0.0 <= p.y <= 100.0: 1 else: 0
    let (a, b) = folds[Fold(k)]
    if (b - a).cross(p - a) < 0.0: return 0
    layers(k - 1, p) + layers(k - 1, reflect(p, a, b))

for _ in 0..q:
    let p: Vec = read()
    print(layers(n, p))
```

### Problem 5: Area of Intersection of Half-Planes

Classic ICPC / Library Checker task: given `n` directed half-planes inside
`[-B, B]^2`, compute the area of their intersection.

```cp
use std.geometry.{Vec, HalfPlane, half_plane_intersect, poly_area}

let n: 1..=2e5 = read()
let bound = 1e9
var planes: List<HalfPlane> = [
    HalfPlane.through(Vec(-bound, -bound), Vec(bound, -bound)),
    HalfPlane.through(Vec(bound, -bound), Vec(bound, bound)),
    HalfPlane.through(Vec(bound, bound), Vec(-bound, bound)),
    HalfPlane.through(Vec(-bound, bound), Vec(-bound, -bound)),
]
for _ in 0..n:
    let (a, b): (Vec, Vec) = read()
    planes.push(HalfPlane.through(a, b))

let poly = half_plane_intersect(planes)
print("{poly_area(poly):.10f}")
```

## 10. Overflow bounds, Shamos–Hoey, float-free `arg`, and 3D hull

### Coordinate-range overflow table

Exact integer geometry is the default path. Coordinate bounds in declarations
(e.g. `Pt<-1e9..=1e9>` once parameterized ranged structs land; today documented
on `Pt`/`Pt3`) determine which width is overflow-free:

| Coordinate bound | Max diff | `dot` / `cross` / `dist2` | `turn` / `cross128` / `dot128` | `any_segments_meet` (`I128`) | `convex_hull_3d` (`I128`) |
| --- | --- | --- | --- | --- | --- |
| `\|c\| <= 1e6` | `2e6` | `8e12` (`Int` safe) | `8e12` (`I128` safe) | `1.6e19` (`I128` safe) | `4.8e19` (`I128` safe) |
| `\|c\| <= 1e9` (`Pt<-1e9..=1e9>`) | `2e9` | `8e18` (2-pt diff fits `Int`; `dot`/`cross` of single vectors <= `2e18`) | `8e18` (`I128` safe, 19 orders spare) | `1.6e28` (`I128` safe) | `4.8e28` (`I128` safe) |
| `\|c\| <= 1e12` | `2e12` | overflows `Int` | `8e24` (`I128` safe) | `1.6e37` (`I128` safe) | `4.8e37` (`I128` safe) |
| `\|c\| <= 2e18` | `4e18` | overflows `Int` | `3.2e37` (`I128` safe) | overflows `I128` | overflows `I128` |

### Additional exact API

- `arg(p: Pt) -> ByAngle` — float-free angular sort key (`sort(mut pts, by: arg)` or `sort(mut pts, by: p => arg(p - base))`).
- `any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int)` — O(n log n) Shamos–Hoey sweep line using an exact `I128` comparator in a deterministic treap (handles vertical segments, shared endpoints, and both `strict: false` and `strict: true`).
- `Pt3(x: Int, y: Int, z: Int)` with `Add`, `Sub`, `Neg`, `Show`, `norm2`, `norm2_128`, `len`, `scale`, `dot3`, `dot3_128`, `cross3`, `dist2_3`, and O(n^2) exact `I128` `convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)>`.

