let n: Int = read()
let a = read<[n]Int>()
var last = Map<Int, Int>()
var best = -1
for i in 0..n:
    let x = a[i]
    if x in last:
        best max= i - last[x]
        last[x] = i
    else:
        last[x] = i
print(best)
