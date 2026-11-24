# Standard library design: `std.geometry`

> **0.3.0 status.** This chapter is the family's design survey and API.
> Where it disagrees with the core chapters or the aligned rounds, they
> win: `core/03-numbers-layout.md` (four number types, no `I32`/`U32`,
> `Mod<P>` not `Mint`), `core/05-ownership.md` (constructors own their
> results), `core/07-sequences.md` (capabilities, views keep keys),
> `lib/ops-ranges.md` (`Op`/`Monoid`/laws, `each(...)`, no
> `RangeFenwick`, `MinHeap`/`MaxHeap` as `Heap(by:)`) and
> `lib/sequence-patterns.md`. The stubs in `std/` are the signatures of
> record. Reconciliation of this text is tracked in `delta/`.


Status: design for `plan/STDLIB.md` section 4.14. Covers 2D competitive-
programming geometry: exact integer points and vectors, orientation, angular
sorting, segments, convex hulls and Minkowski sums, polygon area and
containment, rotating calipers, closest pair, half-plane intersection, and
careful floating-point geometry.

## The model in one paragraph

Geometry is exact by default: `Pt` holds integer coordinates, `turn`
gives the orientation of three points from one cross product, and
`ByAngle` is a sort key, so angular order is `ps.sort(by: ByAngle)`
rather than a comparator full of special cases. Hulls (`convex_hull`,
`lower_hull`, `upper_hull`, and `_idx` forms that return positions),
segment tests (`on_segment`, `segments_meet`) and polygon area and
containment build on those primitives. Floating-point geometry is a
separate, explicitly marked layer, used only when the problem needs it.

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
    upper chains in \(O(n)\) on already-sorted points.
  - `ucup5s05e.cp` hand-rolls lower and upper hulls *keeping collinear points*
    while preserving original point IDs.
  - `cf2138f.cp` (`research/frictions/cf2138f.md`) hand-rolls a `F64` vector
    type `V` with `len`, `Add`, `Sub`, and scalar `times`/`over`.
  - `etsr23r3p2.cp` asks for `F64` points and `reflect(p, a, b)`.
  - `dmopc20c7p2.cp` (`research/archive/frictions/ugly-code-friction-audit-3.md` section 9)
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
`.norm2`, `.len`, `.perp`, `.perp_cw`, `.angle`, `.dot(o)`,
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

<!-- spec-only: signatures and costs, not a program -->
```cp
struct Pt(x: Int, y: Int)         // Add, Sub, Neg, Eq, Ord (x, y), Hash, Read, Show ("x y")
    @get fn norm2(self) -> Int    // O(1); x*x + y*y
    @get fn len(self) -> F64      // O(1); sqrt(F64(norm2))
    @get fn perp(self) -> Pt      // O(1); 90 deg CCW: Pt(-y, x)
    @get fn perp_cw(self) -> Pt   // O(1); 90 deg CW: Pt(y, -x)
    @get fn angle(self) -> F64    // O(1); atan2(y, x) in [-pi, pi]
    @get fn unit_dir(self) -> Pt  // O(log min(|x|, |y|)); reduced by gcd, Pt(0, 0) for zero
    @get fn to_vec(self) -> Vec   // O(1); Vec(F64(x), F64(y))
    fn scale(self, k: Int) -> Pt  // O(1); Pt(x * k, y * k)
    fn dot(self, o: Pt) -> Int    // O(1)
    fn cross(self, o: Pt) -> Int  // O(1)
    fn dist2(self, o: Pt) -> Int  // O(1)
    fn dist(self, o: Pt) -> F64   // O(1)

fn dot(a: Pt, b: Pt) -> Int          // O(1); safe for |coords| <= 2e9
fn cross(a: Pt, b: Pt) -> Int        // O(1); safe for |coords| <= 2e9
fn dot128(a: Pt, b: Pt) -> I128      // O(1); safe for all Int
fn cross128(a: Pt, b: Pt) -> I128    // O(1); safe for all Int
fn dist2(a: Pt, b: Pt) -> Int        // O(1); (a - b).norm2
fn dist(a: Pt, b: Pt) -> F64         // O(1); (a - b).len
fn manhattan(a: Pt, b: Pt) -> Int    // O(1); |a.x - b.x| + |a.y - b.y|
fn turn(a: Pt, b: Pt, c: Pt) -> Int  // O(1), I128; 1 CCW, -1 CW, 0 collinear
fn same_dir(a: Pt, b: Pt) -> Bool    // O(1), I128; parallel and same ray
```

### 3.2 Angular sorting

<!-- spec-only: signatures and costs, not a program -->
```cp
struct ByAngle(p: Pt)                          // Ord: CCW in [0, 2*pi) from +x; zero first; I128
struct ByAngleFrom(base: Pt, p: Pt)            // Ord: CCW in [base, base + 2*pi); zero first; I128
fn angle_less(base: Pt, a: Pt, b: Pt) -> Bool  // O(1), I128; a precedes b CCW from base
```

`ByAngle` works directly with `sort(mut dirs, by: ByAngle)` and around an
origin via `sort(mut pts, by: p => ByAngle(p - o))`. `ByAngleFrom(base, p)`
orders directions starting from any non-zero ray `base` without splitting or
wrapping the array twice.

### 3.3 Segments

<!-- spec-only: signatures and costs, not a program -->
```cp
fn on_segment(p: Pt, a: Pt, b: Pt) -> Bool
// O(1), I128; true iff p lies on the closed segment [a, b]
fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool
// O(1), I128; closed segments share a point, or proper interior crossing when strict: true
```

### 3.4 Convex hulls, chains, and Minkowski sums

<!-- spec-only: signatures and costs, not a program -->
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

Keeping `lower_hull` and `upper_hull` \(O(n)\) on already x-sorted inputs is
essential for \(O(n \log n)\) divide-and-conquer hulls (such as `ucup4s25e`).
`convex_hull` sorts, deduplicates, and stitches the two chains.

### 3.5 Polygons and rotating calipers

<!-- spec-only: signatures and costs, not a program -->
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
fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int)
// O(log n), I128; tangent vertex indices from strictly outside point p to CCW hull
fn min_bounding_box(hull: List<Pt>) -> (area: F64, perimeter: F64)
// O(n), I128 calipers; minimum-area and minimum-perimeter enclosing rectangle of a CCW hull
```

### 3.6 Closest pair

<!-- spec-only: signatures and costs, not a program -->
```cp
fn closest_pair2(pts: List<Pt>) -> Int
// O(n log n) time, O(n) space; min squared distance among pts (requires pts.len >= 2)
fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int)
// O(n log n) time, O(n) space; closest distinct pair of D handles and squared distance
```

### 3.7 Careful floating-point geometry and half-plane intersection

<!-- spec-only: signatures and costs, not a program -->
```cp
const PI: F64 = 3.14159265358979323846

fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool  // O(1); |a - b| <= eps
fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int        // O(1); 1, 0, or -1 with tolerance eps

struct Vec(x: F64, y: F64)         // Add, Sub, Neg; intentionally no Show
    @get fn norm2(self) -> F64     // O(1)
    @get fn len(self) -> F64       // O(1)
    @get fn unit(self) -> Vec      // O(1); traps if len == 0
    @get fn perp(self) -> Vec      // O(1); Vec(-y, x)
    @get fn perp_cw(self) -> Vec   // O(1); Vec(y, -x)
    @get fn angle(self) -> F64     // O(1); atan2(y, x) in [-pi, pi]
    fn scale(self, k: F64) -> Vec  // O(1); Vec(x * k, y * k)
    fn div(self, k: F64) -> Vec    // O(1); Vec(x / k, y / k)
    fn dot(self, o: Vec) -> F64    // O(1)
    fn cross(self, o: Vec) -> F64  // O(1)
    fn dist2(self, o: Vec) -> F64  // O(1)
    fn dist(self, o: Vec) -> F64   // O(1)
    fn rot(self, by: Vec) -> Vec   // O(1); rotate by unit vector `by`

fn polygon_area(poly: List<Vec>) -> F64                      // O(n); signed area (+ CCW, - CW)
fn project(p: Vec, a: Vec, b: Vec) -> Vec                    // O(1); orthogonal projection onto line ab
fn reflect(p: Vec, a: Vec, b: Vec) -> Vec                    // O(1); reflection across line ab
fn line_dist(p: Vec, a: Vec, b: Vec) -> F64                  // O(1); perpendicular distance from p to line ab
fn seg_dist(p: Vec, a: Vec, b: Vec, eps: F64 = 1e-9) -> F64  // O(1); distance from p to segment [a, b]
fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec
// O(1); intersection of lines ab and cd, None if parallel
fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>
// O(n); Sutherland-Hodgman clip to left half-plane of directed line a -> b
fn circle_line_intersect(c: Vec, r: F64, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>
// O(1); 0, 1, or 2 intersection points of circle (c, r) with line ab
fn circle_intersect(c1: Vec, r1: F64, c2: Vec, r2: F64, eps: F64 = 1e-9) -> List<Vec>
// O(1); 0, 1, or 2 intersection points of two circles
fn circle_tangents(center: Vec, r: F64, p: Vec, eps: F64 = 1e-9) -> List<Vec>
// O(1); 0, 1, or 2 tangent points from p to circle (center, r)
fn circle_circle_tangents(
    c1: Vec,
    r1: F64,
    c2: Vec,
    r2: F64,
    inner: Bool = false,
    eps: F64 = 1e-9,
) -> List<(Vec, Vec)>
// O(1); 0, 1, or 2 outer (or inner when inner: true) common tangent segments (on c1, on c2)
fn circles_union_area(circles: List<(c: Vec, r: F64)>, eps: F64 = 1e-9) -> F64
// O(n^2 log n); total area of the union of disks via Green's theorem arc sweep
fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec
// O(1); circumcenter of triangle abc, None if collinear

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
- **Closest pair**: `closest_pair2(pts)` matches \(O(n^2)\) brute force, and
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
- **Delaunay triangulation, Voronoi diagrams, and KD-trees** (niche; exact integer
  incircle is a degree-4 determinant reaching `16e36` for `1e9` coordinates,
  overflowing `I128` without multi-precision integers).
- **General non-convex polygon union area** (rare compared to circle union and
  convex clipping; user code or `local-lib`).
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
        best[p.x - 1] = Pt(p.x, max(best[p.x - 1].y, p.y))

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
    first(
        s for s in Site if turn(pts[hull[0]], pts[hull[2]], pts[s]) != 0 and turn(
            pts[hull[1]],
            pts[hull[3]],
            pts[s],
        ) != 0
    )
else: None

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
    let d = t / F64(k)
    let side = Vec(0.5, d.y - 0.5)
    ladder(2, 1, 1, i => side + d * F64(i - 1), i => d * F64(i))
else:
    let d = t.unit
    let (t1, t0) = (t - d * F64(k - 1), t - d * F64(k))
    let start = if fits(t0, p2): t0 else: Vec(
        0.5 + 0.75 * sqrt(3.0 / 13.0),
        0.75 + sqrt(3.0 / 13.0) * 0.5,
    )
    fn side() -> Vec:
        for x in -64..=64:
            for y in -64..=64:
                let c = Vec(F64(x) / 64.0, F64(y) / 64.0)
                if (
                    fits(c, p1) and fits(c, p2) and fits(c, start) and fits(c, t0) and fits(c, t1)
                ): return c
        p1
    let s = side()
    let a = draw(1, 2, s)
    let b = draw(2, a, start)
    let c = draw(a, b, t1)
    ladder(a, c, 2, i => s + d * F64(i - 1), i => t0 + d * F64(i))
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
use std.geometry.{Vec, HalfPlane, half_plane_intersect, polygon_area}

let n: 1..=2e5 = read()
let bound = F64(1e9)
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
print("{polygon_area(poly):.10f}")
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
- `any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int)` — \(O(n \log n)\) Shamos–Hoey sweep line using an exact `I128` comparator in a deterministic treap (handles vertical segments, shared endpoints, and both `strict: false` and `strict: true`).
- `Pt3(x: Int, y: Int, z: Int)` with `Add`, `Sub`, `Neg`, `Show`, `norm2`, `norm2_128`, `len`, `scale`, `dot3`, `dot3_128`, `cross3`, `dist2_3`, and \(O(n^2)\) exact `I128` `convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)>`.

## Coverage

Audit against AtCoder Library (which omits geometry), KACTL (`content/geometry/`), cp-algorithms (Geometry), and top competitors' personal templates (ecnerwala, tourist, benq, jiangly). Each topic is marked **have** (implemented and tested in `stdlib/geometry.cp`), **new** (specified in `plan/STDLIB.md` 4.14 `[new]`), or **out** (excluded with reason).

| # | Topic | Status | Surface / Reason |
| --- | --- | --- | --- |
| 1 | Exact 2D point/vector arithmetic & `I128` predicates | **have** | `Pt`, `norm2`, `norm2_128`, `len`, `perp`, `perp_cw`, `angle`, `unit_dir`, `to_vec`, `dot`, `cross`, `dot128`, `cross128`, `dist2`, `dist2_128`, `dist`, `turn`, `same_dir` |
| 2 | Exact angular sorting from `+x` | **have** | `ByAngle`, `arg`, `angle_less` |
| 3 | Segment containment & pairwise intersection | **have** | `on_segment(..., strict:)`, `segments_meet(..., strict:)` |
| 4 | Any-two-segments intersection (Shamos–Hoey sweep) | **have** | `any_segments_meet(..., strict:)` |
| 5 | 2D convex hull & x-monotone chains (batch) | **have** | `lower_hull`, `upper_hull`, `convex_hull`, `convex_hull_idx` |
| 6 | Minkowski sum (full & upper chain) | **have** | `minkowski_sum`, `minkowski_upper` |
| 7 | Polygon signed area & point location (simple & convex) | **have** | `area2`, `area2_128`, `Location`, `locate`, `locate_convex`, `polygon_area` |
| 8 | Rotating calipers (diameter & min bounding box) | **have** | `diameter2`, `min_bounding_box` |
| 9 | Convex hull \(O(\log n)\) queries (extreme & tangents) | **have** | `hull_extreme`, `hull_tangents` |
| 10 | Closest pair of points (2D exact) | **have** | `closest_pair2`, `closest_pair<D>` |
| 11 | Careful floating-point 2D vectors & lines | **have** | `Vec`, `close`, `sign_eps`, `vdot`, `vcross`, `vdist`, `vturn`, `project`, `reflect`, `line_dist`, `seg_dist`, `line_intersect`, `circumcenter` |
| 12 | Circle-line, circle-circle intersections & tangents | **have** | `circle_line_intersect`, `circle_intersect`, `circle_tangents`, `circle_circle_tangents` |
| 13 | Circle-polygon intersection area & union of circles | **have** | `circle_polygon_area`, `circles_union_area` |
| 14 | Minimum enclosing circle (Welzl) | **have** | `min_enclosing_circle` |
| 15 | Half-plane intersection & Sutherland–Hodgman clip | **have** | `HalfPlane`, `half_plane_intersect`, `polygon_cut` |
| 16 | 3D exact integer points & 3D convex hull | **have** | `Pt3`, `dot3`, `dot3_128`, `cross3`, `dist2_3`, `convex_hull_3d` |
| 17 | L1 / L-infinity metrics & `Pt`/`Vec`/`Pt3` method parity | **new** | `manhattan`, `chebyshev`, `.dot`/`.cross`/`.dist2`/`.dist` on `Pt`, `Vec`, `Pt3`, `Vec.perp_cw`, `Vec.from_angle`, `angle_between`, `PI` |
| 18 | Arbitrary-base exact angular sorting | **new** | `ByAngleFrom`, `angle_less(..., from:)` |
| 19 | Segment/ray floating-point intersections | **new** | `segment_intersect`, `ray_hits_segment` |
| 20 | Domain-indexed segment & half-hull twins | **new** | `any_segments_meet_idx<D>`, `lower_hull_idx<D>`, `upper_hull_idx<D>`, `convex_hull_3d_idx<D>` |
| 21 | Online (incremental) 2D convex hull | **new** | `OnlineHull` (`add`, `locate`, `extreme`, `area2`, `len`, `is_empty`, `vertices`, `clear`) |
| 22 | Minkowski lower chain & convex polygon overlap/intersection | **new** | `minkowski_lower`, `convex_polygons_overlap`, `convex_polygon_intersect` |
| 23 | Convex hull width & \(O(\log n)\) line-hull intersection | **new** | `hull_width`, `hull_line_intersect` |
| 24 | Farthest pair & Manhattan (L1) MST candidates | **new** | `farthest_pair<D>`, `manhattan_mst<D>` |
| 25 | Simple polygon centroid, kernel, triangulation & Pick's theorem | **new** | `polygon_centroid`, `polygon_kernel`, `polygon_triangulate`, `pick_interior`, `pick_boundary` |
| 26 | First-class `Circle`, incenter, circle-circle area & inversion | **new** | `Circle` (`through`, `area`, `circumference`, `locate`), `incenter`, `circle_circle_area`, `invert` |
| 27 | `HalfPlane` factories & optional bounding box | **new** | `HalfPlane.through`, `HalfPlane.from_line`, `HalfPlane.side`, `half_plane_intersect(..., bound:)` |
| 28 | Axis-aligned integer rectangles & union area sweep | **new** | `Rect(x: Range<Int>, y: Range<Int>)`, `rects_union_area` |
| 29 | Planar straight-line graph faces & offline point location | **new** | `planar_faces<V>`, `point_location_planar<V>` |
| 30 | Exact Delaunay triangulation & Voronoi cells | **new** | `delaunay<D>`, `voronoi_cells<D>` |
| 31 | 3D floating-point vectors, planes, polyhedron volume & spherical distance | **new** | `Vec3`, `Plane3`, `line_plane_intersect`, `sphere_dist`, `polyhedron_volume6` |
| 32 | 1D convex hull trick / `LineContainer` / `LiChao` | **out** | Owned by `std.dp` (4.16): 1D linear envelopes `k*x + m` are DP optimizations, not 2D geometry. |
| 33 | KD-tree / 2D orthogonal range tree | **out** | Problem-specific pruning and monoids belong in `std.ranges` or user code; static 2D sums are `Prefix<R, C, T>` / `Fenwick2D`; removing every point of a rectangle is `PointSet` (42). |
| 34 | General non-convex polygon boolean ops (union/diff/intersection) | **out** | Rare in contests and edge-case heavy; `polygon_cut`, `convex_polygon_intersect`, `circles_union_area`, and `rects_union_area` cover contest needs. |
| 35 | General 3D mesh boolean ops / CSG / 3D half-space intersection | **out** | Virtually absent from contests; `convex_hull_3d`, `Plane3`, and `polyhedron_volume6` cover 3D contest geometry. |
| 36 | Exact homogeneous / rational points (`Hom(x, y, w)`) | **out** | Specialized to exact line-arrangement graphs; belongs in `std.math` (`Frac`) or `examples/local-lib/` rather than duplicating every 2D primitive. |
| 37 | Generic sweep-line event framework | **out** | Sweep state transitions are problem-specific; concrete sweeps (`any_segments_meet`, `circles_union_area`, `rects_union_area`, `point_location_planar`) and exact keys (`ByAngle`, `turn`) stay composable. |
| 38 | Spatial hashing / quadtrees / BVH | **out** | Heuristic acceleration structures; `closest_pair` and `delaunay` cover exact proximity. |
| 39 | Ellipses, Bezier curves, and general conics | **out** | Rare in contests; affine scaling reduces ellipses to circles and ternary search handles general smooth curves. |
| 40 | 3D dynamic convex hull / 3D Delaunay / 3D Voronoi | **out** | Outside standard contest scope; \(O(n^2)\) static `convex_hull_3d` suffices. |
| 41 | Sub-polygon chord-area prefix structure | **out** | Two lines composing `Prefix<D, Int>` over `cross(poly[i], poly[i.next])`; no standalone type needed. |
| 42 | Draining the points of a rectangle (an edge to every point of `[L,R) x [D,U)` in a shortest path) | **new** | `PointSet.of(pts)`, `remove(i)`, `drain(x, y)`: amortized \(O(\log^2 n + k)\) |

Totals: **16 have**, **16 new**, **10 out** (42 topics).

## Cross-area

### Construction and naming coherence
- **Construction convention:** Plain positional constructors `T(...)` for all value types (`Pt(x, y)`, `Pt3(x, y, z)`, `Vec(x, y)`, `Vec3(x, y, z)`, `Circle(c, r)`, `HalfPlane(p, dir)`, `Plane3(n, d)`, `Rect(x, y)`, `ByAngle(p)`, `ByAngleFrom(base, p)`) and empty constructor `OnlineHull()` for stateful structures (matching `Dsu()`, `MinHeap()`, `IntervalMap()` in `plan/STDLIB.md` §2). Alternate representations use named factories (`HalfPlane.through`, `HalfPlane.from_line`, `Circle.through`, `Plane3.through`, `Vec.from_angle`).
- **Argument vocabulary:** `strict: Bool = false`, `collinear: Bool = false`, `inner: Bool = false`, `from: Pt = Pt(1, 0)`, `by: Vec`, `bound: ?F64 = none`, `eps: F64 = 1e-9` — no `_by` or `_with` twins. Domain-indexed twins of `List<Pt>` functions consistently use `_idx` (`convex_hull_idx`, `lower_hull_idx`, `upper_hull_idx`, `any_segments_meet_idx`, `convex_hull_3d_idx`); domain-first functions (`closest_pair<D>`, `farthest_pair<D>`, `manhattan_mst<D>`, `delaunay<D>`, `voronoi_cells<D>`, `planar_faces<V>`) keep the base name.
- **Method parity across vector types:** Adding `.dot(o)`, `.cross(o)`, `.dist2(o)`, `.dist(o)` on `Pt`, `Vec`, `Pt3`, and `Vec3` removes the `dot`/`vdot`/`dot3` split at method call sites while preserving short free functions `dot`/`cross`/`turn` for `Pt`.

### Cross-area name boundaries and collisions
- **`dist` (`std.geometry` vs `std.tree.Rooted.dist` / `std.graph`):** `std.geometry.dist(a: Pt, b: Pt) -> F64` is a free function (and `.dist(o)` method on `Pt`/`Vec`/`Pt3`/`Vec3`); `Rooted.dist(a: V, b: V) -> Int` is a method on `Rooted`. No collision when `dist` is called as a method or imported explicitly.
- **`Line` / convex hull trick (`std.dp` vs `std.geometry`):** `std.dp` (4.16) owns `Line(k, m)` and `LiChao` for 1D linear DP envelopes. `std.geometry` deliberately omits a `Line` struct (using `HalfPlane` or endpoint pairs `(a, b)`) and restricts `OnlineHull` to 2D integer point hulls.
- **`PI`, `close`, `sign_eps` (`std.geometry` vs `std.math`):** `PI` belongs canonically in `std.math` (also needed by FFT in `std.ntt`) and is re-exported by `std.geometry` to avoid duplication. `close` and `sign_eps` live in `std.geometry` today; if `std.math` adds floating-point root-finding or ternary search, they should move to `std.math` with re-exports here.
- **`Range<Int>` (`core` / `std.interval` vs `std.geometry.Rect`):** `Rect(x: Range<Int>, y: Range<Int>)` reuses core `Range<Int>` (and cut-range conversion per decision 124) instead of defining a second interval type.
- **`manhattan_mst` and `planar_faces` (`std.geometry` vs `std.graph`):** Owned by `std.geometry` because both are driven by geometric sweeps / angular sorting (`ByAngle`) and return plain edge lists or vertex cycles that compose directly with `std.graph` and `Dsu`.
- **`std.prelude` slots:** Only `Pt` and `ByAngle` are in `std.prelude` (2 of 80 slots); all other names require `use std.geometry.{...}`.

## Powers

Language powers needed to write the full `std.geometry` spec without special cases or copies:

1. **Trigonometric intrinsics (`sin`, `cos`, `acos`, `asin`)** (`decisions.md` §1.5; today only `atan2` exists).
   - **Signatures:** `Vec.from_angle(rad: F64) -> Vec`, `Vec3.rot(self, axis: Vec3, rad: F64) -> Vec3`, `sphere_dist(a: Vec3, b: Vec3, r: F64 = 1.0) -> F64`.
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let q = v.rot(by: Vec.from_angle(PI / 3.0))
     ```
2. **Protocols with associated types (`Indexed<Item = T>` / `Seq<Item = T>`).**
   - **Signatures:** Generalizing read-only sequence inputs from `List<Pt>` / `List<Vec>` to `S: Indexed<Item = Pt>` / `S: Indexed<Item = Vec>` across `area2`, `area2_128`, `locate`, `locate_convex`, `lower_hull`, `upper_hull`, `convex_hull`, `minkowski_sum`, `minkowski_upper`, `minkowski_lower`, `hull_extreme`, `hull_tangents`, `hull_width`, `hull_line_intersect`, `diameter2`, `min_bounding_box`, `closest_pair2`, `polygon_area`, `polygon_cut`, `polygon_centroid`, `polygon_kernel`, `polygon_triangulate`, `circle_polygon_area`, `pick_interior`, `pick_boundary`.
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let left = upper_hull(pref[lo..md])   // zero-copy slice of [D]Pt or List<Pt>
     ```
3. **Range-parameterized structs with defaults (`struct Pt<R = Int>(x: R, y: R)`).**
   - **Signatures:** `Pt<R = Int>`, `Pt3<R = Int>` (packing `Pt<-1e9..=1e9>` into 8 bytes and carrying coordinate bounds for compile-time overflow proofs on `dot`/`cross`/`dist2`).
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let p: [V]Pt<1..=100> = read()
     ```
4. **Wide multiplication intrinsic (`mulhi` / exact 256-bit sign)** (`plan/STDLIB.md` §3 `[new, core lane]`).
   - **Signatures:** `delaunay<D>(pts: [D]Pt) -> List<(D, D, D)>`, `voronoi_cells<D>(pts: [D]Pt, bound: F64) -> [D]List<Vec>` (exact degree-4 `incircle` determinant reaches `1.92e38` for `|coords| <= 1e9`, exceeding `I128.MAX = 1.70e38`).
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let tris = delaunay(pts)   // exact incircle for |x|, |y| <= 1e9
     ```
5. **Public `Read` protocol (`impl Read for T`).**
   - **Signatures:** `impl Read for Vec`, `impl Read for Pt3`, `impl Read for Vec3`, `impl Read for Circle`, `impl Read for HalfPlane`, `impl Read for Rect` (today only `Pt` has compiler-assisted `Read`).
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let cs: [C]Circle = read()
     ```
6. **Half-open cut-range conversion (`Range<D.bounds>` -> `Range<D>`, decision 124).**
   - **Signatures:** `Rect(x: Range<Int>, y: Range<Int>)` and passing `poly[j..i]` (with `j, i: D.bounds`) into `area2`, `upper_hull`, `lower_hull`, `locate`.
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let a = area2(poly[j..i])
     ```
7. **User-defined optional niches (`?Vec`, `?Pt`, `?Circle`, `?Rect`, `?Plane3`).**
   - **Signatures:** `line_intersect(...) -> ?Vec`, `circumcenter(...) -> ?Vec`, `ray_hits_segment(...) -> ?Vec`, `Circle.through(...) -> ?Circle`, `incenter(...) -> ?Circle`, `Rect.intersect(...) -> ?Rect`, `OnlineHull.extreme(...) -> ?Pt`, `Plane3.through(...) -> ?Plane3`, `line_plane_intersect(...) -> ?Vec3` (avoids an extra 8-byte discriminant word; e.g. NaN-niche keeps `?Vec` at 16 B instead of 24 B).
   - **Smallest example:**
     <!-- spec-only: signatures and costs, not a program -->
     ```cp
     let hit: ?Vec = line_intersect(a, b, c, d)
     ```

## Dreams

Audit of every dream in `proto/ports/*/dream/` that touches `std.geometry`:

1. **`proto/ports/ecnerwala/dream/cf1146h.cp`**
   - Uses `use std.geometry.{Pt, turn}`, `[V]Pt = read()`, `turn(pts[i], pts[j], pts[k]) > 0`.
   - **Status:** Every API is in the spec (`[have]`) with identical spelling.
2. **`proto/ports/ecnerwala/dream/cf1284e.cp`**
   - Uses `use std.geometry.{cross}`, `[V]Pt = read()`, `pts[v] - pts[c]`, `.sorted(by: ByAngle)`, `cross(r, s) > 0`.
   - **Status:** Every API is in the spec (`[have]`) with identical spelling (`Pt` and `ByAngle` are in `std.prelude`).
3. **`proto/ports/ecnerwala/dream/cf2138f.cp`**
   - Uses `use std.geometry.Vec`, `Vec(p, q)`, `(a - b).len`, `t / k`, `d * (i - 1)`, `t.unit`, `Map<Vec, Int>`.
   - **Status:** All geometry operations (`Vec`, `Add`, `Sub`, `Mul<F64>`, `Div<F64>`, `.len`, `.unit`) are in the spec (`[have]`). Two dream idioms differ intentionally by spec rule (§2): integer arguments to `Vec` / scalar ops require explicit `F64(...)` (or `Pt(p, q).to_vec`) so exact ints never silently widen to float, and `Vec` omits `Eq`/`Hash` (so the compiling port tracks integer point IDs instead of `Map<Vec, Int>`).
4. **`proto/ports/ecnerwala/dream/cf2178i.cp`**
   - Uses `let p: [V]Vec2<1..=100> = read()` and `(p[u] - p[v]).norm2 == k`.
   - **Status:** Spelled `Pt` today (`[have]` in `std.prelude`, with `Sub` and `.norm2`), and `Pt<1..=100>` once range-parameterized structs (`## Powers` #3) land — `std.geometry` spells the integer 2D point type `Pt`, not `Vec2`.

