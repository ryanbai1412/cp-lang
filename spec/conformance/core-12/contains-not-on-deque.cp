// expect: E0606
// rule: 12.4.7
// A Deque is scanned in O(n), so it has no `Contains`: the search is `q.contains(x)`.
use std.deque.{Deque}
var q = Deque<Int>()
q.push(3)
print(yes(3 in q)) // <- here
