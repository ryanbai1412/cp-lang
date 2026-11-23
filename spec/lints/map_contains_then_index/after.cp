let n: Int = read()
let a = read<[n]Int>()
var first = Map<Int, Int>()
var best = -1
for i in 0..n:
    let x = a[i]
    if let v = first.get(x):
        best max= i - v
    else:
        first[x] = i
print(best)
