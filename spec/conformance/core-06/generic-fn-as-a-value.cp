// expect: ok, output "1\n2\n3\n3 2 1"
// rule: 06.1.5
// A generic fn named as a value is the closure that calls it: `by: desc` is `by: x => desc(x)`,
// also where the parameter is optional (`?(T -> K)`), and a user's generic fn works the same.
use std.algebra.{desc}
use std.heap.{Heap}
fn pick<T, K>(x: T, by: ?(T -> K) = None) -> Int = 1
fn pick2<T, K>(x: T, by: T -> K) -> Int = 2
fn twice<T>(x: T) -> (T, T) = (x, x)
print(pick(3, by: desc))
print(pick2(3, by: twice))
var h: Heap<Int> = Heap(by: desc)
h.push(1)
h.push(3)
print(h.pop()!)
print([1, 3, 2].sorted(by: desc))
