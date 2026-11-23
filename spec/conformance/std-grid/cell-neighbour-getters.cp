// expect: ok, output "0 1 2 1 1 2 YES YES"
// `p.up`/`p.down`/`p.left`/`p.right` are getters returning `?(R, C)`, `None` off the grid (lib/graphs-trees.md).
domain R = 0..3
domain C = 0..3
let p = (R(1), C(1))
let (ur, uc) = p.up!
let (dr, dc) = p.down!
let (lr, lc) = p.left!
let (rr, rc) = p.right!
print(Int(ur), Int(uc), Int(dr), Int(dc), Int(rr), Int(rc), yes((R(0), C(0)).up == None), yes(p.step(Dir4.up) == p.up))
