@!multitest
use std.geometry.{turn}
type Coord = -1e9..=1e9
let (x1, y1, x2, y2, x3, y3) = read<(Coord, Coord, Coord, Coord, Int, Int)>()
let side = turn(Pt(x1, y1), Pt(x2, y2), Pt(x3, y3))
if side == 1:
    print("LEFT")
elif side == -1:
    print("RIGHT")
else:
    print("TOUCH")
