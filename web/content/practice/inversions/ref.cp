let n: 1..=2e5 = read()
domain V = 1..=n
let a: [0..n]V = read()
var seen = Fenwick<V, Int>.new()
var count = 0
for x in a:
    count += seen.fold(x..)
    seen.add(x, 1)
print(count)
