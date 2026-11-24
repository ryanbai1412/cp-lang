// expect: ok, output "2\n2\n3"
// A field bound by a `let` moves only when nothing reads it again (8c5b0e71a); the next iteration reads `s.xs`, so each `t` is a copy.
struct S:
    xs: List<Int>
    n: Int
var s = S([1, 2], 3)
for i in 0..2:
    var t = s.xs
    t.push(i)
    print(s.xs.len)
s.xs.push(9)
print(s.xs.len)
