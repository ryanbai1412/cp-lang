// expect: ok, output "1 3"
// rule: R05.5.3
// An `Int` read with `get` holds nothing, so `xs` is free to change while `x` is used.
var xs = [1, 2]
let x = xs.get(0)
xs.push(3)
print(x, xs.len)
