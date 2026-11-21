use std.geometry.{any_segments_meet}

let n: Int = read()
var segs: List<(a: Pt, b: Pt)> = []
for _ in 0..n:
    let (ax, ay, bx, by): (Int, Int, Int, Int) = read()
    segs.push((a: Pt(ax, ay), b: Pt(bx, by)))
let hit = any_segments_meet(segs)
print(if hit != none: 1 else: 0)
