// expect: E0608 at 5
// rule: 05.2.2
var xs = List(1, 2)
let c = xs.len > 1
if c:
    let ys = xs
print(xs.len)
