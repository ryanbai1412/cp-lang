// expect: ok, output "4 5 100\n9 100"
// rule: std.heap.MeldHeaps
// input: "2\n3 5 3 4\n2 9 1\n"
// A pool that frees and reuses nodes inside one test case starts empty in the next: no free node
// carries over between cases.
use std.heap.{Meld, MeldHeaps}
@!multitest
let k: 1..=10 = read()
let xs: [0..k]Int = read()
var p = MeldHeaps<Int>()
var h = Meld<Int>()
for x in xs: p.push(mut h, x)
p.pop(mut h)
p.push(mut h, 100)
print(p.drain(mut h))
