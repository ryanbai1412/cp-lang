// expect: ok, output "YES"
use std.geometry.{hull_extreme, Pt}
let sq = [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
let oct = [Pt(1, 0), Pt(2, 0), Pt(3, 1), Pt(3, 2), Pt(2, 3), Pt(1, 3), Pt(0, 2), Pt(0, 1)]
var ok = true
for h in [sq, oct]:
    for dx in -3..=3:
        for dy in -3..=3:
            let d = Pt(dx, dy)
            var want = 0
            for i in 0..h.len:
                if h[i].x * dx + h[i].y * dy > h[want].x * dx + h[want].y * dy: want = i
            ok = ok and hull_extreme(h, d) == want
print(yes(ok))
