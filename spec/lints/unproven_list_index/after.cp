domain D = 0..3
let a: [D]Int = [1, 2, 3]
var s = 0
for _ in D:
    for j in D:
        s += a[j]
print(s)
