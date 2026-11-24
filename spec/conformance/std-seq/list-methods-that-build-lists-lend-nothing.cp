// expect: ok, output "3 3 4"
// rule: 05.5.3
// A list's `map` and `items` build new lists, so `xs` is free to change.
var xs = [1, 2, 3]
let a = xs.map(x => x + 1)
let e = xs.items
xs.push(9)
print("{a.len} {e.len} {xs.len}")
