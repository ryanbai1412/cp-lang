let n: 1..=100 = read()
let a: [0..n]Int = read()
var last = -1
for i in 0..n:
    if a[i] == 0:
        last = i
print(last)
