let n: Int = read()
let a = read<[n]Int>()
var first = Map<Int, Int>()
var best = -1
for i in 0..n:
    let x = a[i]
    if x in first:
        best max= i - first[x]
    else:
        first[x] = i
print(best)
fn twice(m: Map<Int, Int>, k: Int) -> Int:
    if k not in m: return 0
    m[k] + m[k]
print(twice(first, 0))
