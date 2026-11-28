// expect: ok, output "0 2 0 ab\n1 1\n0 2"
// rule: 05.2.2
// `take()` moves a List, Str, ?T or Deque out of a place that stays alive and leaves its empty value behind.
use std.deque.{Deque}
var xs = [1, 2]
let ys = xs.take()
var s = "ab"
let t = s.take()
var a: ?Int = Some(3)
let b = a.take()
print(xs.len, ys.len, s.len, t)
print(if a == None: 1 else: 0, if b == Some(3): 1 else: 0)
var q = Deque.of([1, 2])
let r = q.take()
print(q.len, r.len)
