// expect: ok, output "a 2"
// rule: R05.5.3
// A result lends from its signature alone: `xs` is a `List<T>`, not a `T`, and the result is not
// a `T`, so `heads` lends nothing and `xs` is free to change while `hs` is used.
fn heads<T>(xs: List<T>) -> List<T>:
    var out: List<T> = []
    out.push(xs[0])
    out
var xs = ["a"]
let hs = heads(xs)
xs.push("b")
print(hs[0], xs.len)
