// expect: E0602
// rule: 05.5.3
// `xs.pairs()` yields from `xs`, so `xs` can't change while `f` lives.
var xs = [1, 2, 3]
let f = xs.pairs()
xs.push(9) // <- here
var n = 0
for _ in f:
    n += 1
print(n)
