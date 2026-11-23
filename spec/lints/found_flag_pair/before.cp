let n: 1..=100 = read()
let a: [0..n]Int = read()
var found = false
var last = 0
for i in 0..n:
    if a[i] % 2 == 0:
        last = a[i]
        found = true
if found:
    print(last)
else:
    print(-1)
