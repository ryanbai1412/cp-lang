// A 4-subset Q encloses center p iff Q does not lie inside any open half-plane through p.
// With no three points collinear, each non-enclosing 4-subset has a unique first ray in
// CCW order around p whose other 3 rays lie strictly within (0, pi) ahead of it.
// Sort the n - 1 rays around each p by angle and count the rays ahead in each open half-plane.
use std.geometry.{ByAngle, Pt, cross}
use std.math.choose

let n: 5..=2500 = read()
let pts: [n]Pt = read()

fn non_enclosing(center: Pt) -> Int:
    let dirs = [p - center for p in pts if p != center].sorted(by: ByAngle)
    let ahead = dirs.cyclic_span(while: (a, b) => cross(a, b) > 0)
    [choose(k - 1, 3) for k in ahead].sum()

print(n * choose(n - 1, 4) - [non_enclosing(p) for p in pts].sum())
