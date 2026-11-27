// expect: ok, output "132"
// rule: R07.6.2
// A generator called outside a `for` is a value; each walk reruns it from the start.
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i
let g = evens(7)
var t = 0
for x in g: t += x
for x in g: t += x * 10
print(t)
