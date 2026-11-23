// expect: ok, output "YES YES NO"
// rule: 04.2.18
// A literal fills a position inside a tuple or record compared with one.
domain V = 0..3
let p: (V, Bool) = (2, false)
let q: (root: V, flip: Bool) = (root: 2, flip: false)
print(yes(p == (2, false)), yes(q == (root: 2, flip: false)), yes(q == (root: 1, flip: false)))
