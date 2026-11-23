let n: 1..=100 = read()
domain V = 0..n
let a: [V]Int = read()
var j = V.first!
for i in V:
    if a[i] > a[j]:
        j = i
print(j)
