// expect: ok, output "2 4 3\n1 5\n1 2"
// rule: 07.2.11
// A built-in member returns the type its row is written with: `a[..1].to_list()` is a `List<Int>`, so `+` joins two of them into a typed `List<Int>`, and `.clone()` of a `[D]T` is a `[D]T` over the same index.
struct P:
    var x: Int
let a = [1, 2, 3, 4]
let b = a[..1].to_list() + a[3..].to_list()
let xs = [1]
let c = xs + [2] + [3]
print(b.len, b[1], c.len)
let ps: [2]P = P(1)
var qs = ps.clone()
qs[0].x = 5
print(ps[0].x, qs[0].x)
var l = xs.clone()
l.push(9)
print(xs.len, l.len)
