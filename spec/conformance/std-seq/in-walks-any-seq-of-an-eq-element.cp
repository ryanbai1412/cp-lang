// expect: ok, output "YES NO YES NO YES NO\nYES"
// rule: 07.9.5
// `x in s` on any `Seq` of an `Eq` element is std's one O(n) `contains`: a Deque, a LinkedList and a
// user Seq all take it, and a type's own `contains` wins.
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
print(yes(5 in dq), yes(4 in dq), yes(7 in ll), yes(8 in ll), yes(6 in ev), yes(7 in ev))
print(yes(9 in Everything(0)))
