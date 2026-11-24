// expect: E0303
// `LinkedList(xs)` is an error, not an empty list.
use std.deque.{LinkedList}
let a: List<Int> = [3, 1, 3]
let l = LinkedList(a) // <- here
