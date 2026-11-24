// expect: ok, output "YES YES 0"
// rule: std.heap.MeldHeaps.meld
// Ascending and descending runs build long skew-heap spines; melding walks them without recursion,
// and every heap still drains in sorted order.
use std.heap.{Meld, MeldHeap, MeldHeaps}
let n = 20_000
var p = MeldHeaps<Int>()
var a = Meld<Int>()
var b = Meld<Int>()
for i in 0..n:
    p.push(mut a, i)
    p.push(mut b, n - i)
var both = p.meld(a, b)
let got = p.drain(mut both)
var want = List(0..n) + List(1..=n)
want.sort()
var h: MeldHeap<Int> = MeldHeap()
for i in 0..n: h.push(if i % 2 == 0: i else: n - i)
var g = MeldHeap<Int>()
for i in 0..n: g.push(-i)
h.meld(g.take())
let hs = h.drain()
print(yes(got == want), yes(hs.is_sorted), g.len)
