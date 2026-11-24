// expect: ok
// rule: 05.7.2
use std.algebra.opt
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
let ws = read<[V]Int>()
let lighter = (x: V, y: V) => if ws[x] <= ws[y]: x else: y
var st = SegTree(lighter.opt(), [V] v => Some(v))
st[i] = Some(i)
