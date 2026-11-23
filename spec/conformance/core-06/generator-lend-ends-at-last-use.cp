// expect: ok, output "6 5"
// rule: 06.3.2
// The generator lends `xs` until its last use; after it, `xs` may change.
fn evens(xs: List<Int>) -> yields Int:
    for x in xs:
        if x % 2 == 0: yield x
var xs = [1, 2, 3, 4]
let s = evens(xs).sum()
xs.push(6)
print(s, xs.len)
