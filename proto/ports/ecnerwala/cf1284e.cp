// A 4-point subset Q encloses p iff Q does not lie inside any open half-plane through p.
// Because no three points are collinear, each non-enclosing 4-subset has a unique first
// ray in CCW order around p whose remaining 3 rays lie strictly within (0, pi) ahead of it.
// Sort the other n - 1 vectors around each p by angle and sweep a half-plane window.
use std.geometry.{Pt, arg, cross}

let n: 5..=2500 = read()
domain V = 0..n
let pts: [V]Pt = read()

fn choose3(k: Int) -> Int = k * (k - 1) * (k - 2) / 6
let total_quads = (n - 1) * (n - 2) * (n - 3) * (n - 4) / 24

fn enclosures(center: V) -> Int:
    let dirs = [pts[v] - pts[center] for v in V if v != center].sorted(by: d => arg(d))
    let m = dirs.len
    var bad = 0
    var j = 0
    for i in 0..m:
        if j < i + 1: j = i + 1
        while j < i + m and cross(dirs[i], dirs[j % m]) > 0:
            j += 1
        bad += choose3(j - i - 1)
    total_quads - bad

print([enclosures(v) for v in V].sum())
