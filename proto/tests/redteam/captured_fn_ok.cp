// fns that change a captured array are fine while nothing else holds it; recursion included
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
var seen: [V]Bool = false
fn go(i: V):
    a[i] += 1
    seen[i] = true
    if i != V.last:
        go(i + 1)
go(V.first)
for v in V:
    if seen[v]: go(v)
let part = a[V(1)..=V(2)]
print(sum(part))
go(V.last)
print(a)
