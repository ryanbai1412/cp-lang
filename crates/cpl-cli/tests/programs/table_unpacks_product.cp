// `[P] (i, d) => e` over `domain P = (A, Dir)` names the parts of each
// position, the same table as `[A, Dir] (i, d) => e` (R01.8.1, R01.8.2).
enum Dir = N | E | S | W
domain A = 0..2
domain P = (A, Dir)
let t: [P]Int = [P] (i, d) => Int(i) * 10 + Int(d)
print(t)
print(t[1, Dir.S], t[P((0, Dir.W))])
let u: [A, Dir]Int = [A, Dir] (i, d) => Int(i) * 10 + Int(d)
print(u[1, Dir.S])
