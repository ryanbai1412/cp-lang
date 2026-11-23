// expect: ok, output "7 9\nh y\n0 1\n-1"
// `first` and `last` read the same way on a List, a Str and a Deque, and are `None` when empty.
let xs = [7, 8, 9]
print(xs.first!, xs.last!)
let s = "hey"
print(s.first!, s.last!)
var d = Deque<Int>()
d.push(1)
d.push_front(0)
print(d.first!, d.last!)
let e = List<Int>()
print(e.first ?? -1)
