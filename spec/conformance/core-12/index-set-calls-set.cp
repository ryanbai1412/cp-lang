// expect: ok, output "7 4\n9 8"
// rule: W8
// A struct whose place protocol is `set` (IndexSet) takes `x[k] = v` as `x.set(k, v)` and
// `x[k] op= v` as `x.set(k, x[k] op v)`; a user type gets this exactly as std's `Deque` does.
use std.deque.{Deque}
struct Box(xs: List<Int>):
    pub fn at(self, i: Int) -> Int = xs[i]
    pub fn set(mut self, i: Int, v: Int):
        xs[i] = v
var b = Box([1, 2])
b[1] = 7
b[0] += 3
print(b[1], b[0])
var q = Deque.of([7, 8])
q[0] = 9
print(q.to_list())
