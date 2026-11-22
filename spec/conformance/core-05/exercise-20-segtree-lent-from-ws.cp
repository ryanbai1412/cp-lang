// expect: ok
// rule: 05.7.2
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
let ws = read<[V]Int>()
var st = SegTree((x, y) => if ws[x] <= ws[y]: x else: y, [V] v => v)
st[i] = i
