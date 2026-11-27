// expect: ok, output "1 0 3\n1 5 9\n5 9 1 9\n-1 -1\n1 0 2\n500 1 999 499"
// rule: R07.2.4
// A `Set` keeps its elements once each, in `<` order: it walks them sorted, finds the
// neighbours of a value, and stays balanced through many adds and removes.
fn b(x: Bool) -> Int = if x: 1 else: 0
var s: Set<Int> = Set()
let a = s.insert(5)
let c = s.insert(5)
s.insert(1)
s.insert(9)
print(b(a), b(c), s.len)
var out: List<Int> = []
for x in s: out.push(x)
print(out[0], out[1], out[2])
print(s.first_ge(2) ?? -1, s.first_gt(5) ?? -1, s.last_le(4) ?? -1, s.last ?? -1)
print(s.last_lt(1) ?? -1, s.first_gt(9) ?? -1)
let r1 = s.remove(5)
let r2 = s.remove(5)
print(b(r1), b(r2), s.len)
var big: Set<Int> = Set()
for i in 0..1000: big.insert((i * 7919) % 1000)
for i in 0..500: big.remove(i * 2)
let n0 = big.len
let f = big.first ?? -1
let l = big.pop_last() ?? -1
print(n0, f, l, big.len)
