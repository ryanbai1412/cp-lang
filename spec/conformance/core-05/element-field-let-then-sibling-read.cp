// expect: ok, output "3 2 2"
// A field of a list element bound by a `let` is copied; the list and its other elements stay whole.
struct S:
    xs: List<Int>
    n: Int
var ss = [S([1, 2], 0), S([3], 1)]
var acc = ss[0].xs
acc.push(ss[1].n)
print(acc.len, ss[0].xs.len, ss.len)
