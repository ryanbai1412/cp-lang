use std.geometry.{closest_pair}

let n: Int = read()
domain P = 0..n
let pts: [P]Pt = read()
let res = closest_pair(pts)
print(res.dist2)
