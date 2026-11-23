// The ends of a sequence: `a.first` and `a.last` are optionals on a list
// and on an array, `none` when it is empty, and `xs.last!` is a cell that
// can be assigned.
struct Run:
    var head: Int
    var len: Int
let n = 4
let a: [0..n]Int = [0..n] i => i * 3
print(a.first, a.last!)
let top: Int = a.last!
print(top + 1)
var runs = [Run(1, 2), Run(3, 4)]
runs.last!.head = 9
runs.first!.len = 7
print(runs[0].len, runs[1].head)
var ys = [1, 2, 3]
ys.last! = 10
ys.first! += 5
print(ys)
var zs: [0..n]Int = 0
zs.last! = 4
zs.first! -= 1
print(zs)
// An array from 1: the builder's `i` is the index, not the offset.
let from_one: [1..=3]Int = [1..=3] i => i * 10
print(from_one, from_one.first, from_one.last)
domain I = 0..n
let by_domain: [I]Int = [I] i => i.to_idx + 10
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => r.to_idx * 3 + c.to_idx
domain Z = 0..0
let e: [Z]Int = [Z] z => 0
print(by_domain.first, by_domain.last, g.first, g.last, e.first, e.last)
