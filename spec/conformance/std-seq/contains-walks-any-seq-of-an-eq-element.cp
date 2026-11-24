// expect: ok, output "YES NO YES NO YES NO\nYES"
// rule: 12.4.7
// `s.contains(x)` on any `Seq` of an `Eq` element is std's one O(n) scan: a Deque, a LinkedList and
// a user Seq all take it. `x in s` stays for a type with a `contains` of its own.
use std.deque.{Deque, LinkedList}
struct Evens(n: Int)
impl Seq<Int> for Evens:
    fn iter(self) -> yields Int:
        for i in 0..n: yield 2 * i
struct Everything(n: Int):
    fn contains(self, x: Int) -> Bool = true
var dq = Deque<Int>()
dq.push(3)
dq.push(5)
var ll = LinkedList<Int>()
ll.push(7)
let ev = Evens(4)
print(yes(dq.contains(5)), yes(dq.contains(4)), yes(ll.contains(7)), yes(ll.contains(8)), yes(ev.contains(6)), yes(ev.contains(7)))
print(yes(9 in Everything(0)))
