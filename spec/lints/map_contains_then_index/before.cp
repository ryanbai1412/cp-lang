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
