// expect: ok, output "3 3 4"
// rule: 05.5.3
// `map` and `items` are views that hold `xs` (R05.5.3), but `collect` builds an owned list from
// each, so once both are built `xs` is free to change.
var xs = [1, 2, 3]
let a = xs.map(x => x + 1).collect()
let e = xs.items.collect()
xs.push(9)
print("{a.len} {e.len} {xs.len}")
