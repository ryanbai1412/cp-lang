// expect: ok, output "1 3 4 5\n8 9\n7"
// rule: std.heap.MeldHeaps
// input: "3\n4 3 5 1 4\n2 9 8\n1 7\n"
// A pool and its heaps declared inside a test case start empty in every case: the library keeps no
// state of its own between cases.
use std.heap.{Meld, MeldHeap, MeldHeaps}
@!multitest
let k: 1..=10 = read()
let xs: [0..k]Int = read()
var p = MeldHeaps<Int>()
var lo = Meld<Int>()
var hi: MeldHeap<Int> = MeldHeap()
for (i, x) in xs.items:
    if i % 2 == 0: p.push(mut lo, x) else: hi.push(x)
var other = p.of(hi.drain())
var all = p.meld(lo, other)
print(p.drain(mut all))
