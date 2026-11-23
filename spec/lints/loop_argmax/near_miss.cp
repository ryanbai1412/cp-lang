let n: 1..=100 = read()
let a: [0..n]Int = read()
var j = 0
for i in 0..n:
    if a[i] >= a[j]:
        j = i
print(j)
