// expect: E0603
// rule: 05.3.5
// `Deque<T>.push(x: T)` keeps what it takes, so a lent row cannot go into a `mut` parameter's deque.
use std.deque.{Deque}
fn f(xs: List<Int>, mut q: Deque<List<Int>>):
    q.push(xs) // <- here
print(1)
