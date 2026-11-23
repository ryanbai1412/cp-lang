let xs = [1, 2, 3, 4]
let sq = List(xs.map(x => x * x))
var t = 0
for i in 0..xs.len:
    for j in 0..xs.len:
        t += sq[i] + sq[j]
print(t)
