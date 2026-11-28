// expect: ok, output "4 1"
// engines: next
struct P(a: Int)
var m = Map<Int, P>()
m[1] = P(3)
m[1] = P(4)
print(m[1].a, m.len)
