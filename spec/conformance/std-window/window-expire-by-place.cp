// expect: ok, output "5 3 3\n-1\n6"
// A window keyed by place drops what falls below `before` without being told its value, oldest
// first, on any op; an empty expire is a no-op.
var w = Window(max)
var out: List<Int> = []
for (i, x) in [5, 1, 3, 2].items:
    w.push(x, at: i)
    w.expire(before: i - 1)
    if i >= 1: out.push(w.all!)
print(out)
w.expire(before: 9)
print(w.all ?? -1)
var s = Window(+)
s.push(1, at: 0)
s.push(2, at: 0)
s.push(3, at: 4)
s.push(3, at: 4)
s.expire(before: 1)
s.expire(before: 1)
print(s.all!)
