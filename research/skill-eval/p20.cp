@!multitest
use std.geometry.{turn}

let (p1, p2, p3) = read<(Pt, Pt, Pt)>()
let t = turn(p1, p2, p3)
if t > 0:
    print("LEFT")
elif t < 0:
    print("RIGHT")
else:
    print("TOUCH")
