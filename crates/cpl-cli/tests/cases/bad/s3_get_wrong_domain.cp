// error: `a.get` takes an Int for `i`, but `c` is a ?C
// `a.get(i)` reads by zero-based `Int` position (decision: `get`/`at` are the uniform `Indexed` accessors); an optional key of another domain is no position.
let n, m: 1..=10 = read()
domain R = 1..=n
domain C = 1..=m
let a: [R]Int = read()
let c: ?C = C.first
print(a.get(c) ?? 0)
