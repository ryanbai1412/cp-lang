// expect: E0608
// rule: 05.2.2
// A std container owns heap storage through its fields, so `let` moves it like a list.
use std.deque.{Deque}
var q: Deque<Int> = Deque()
q.push(1)
let r = q
print(q.len + r.len) // <- here
