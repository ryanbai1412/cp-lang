let a: List<Int> = [1, 2, 3]
var s = 0
for i in 0..3:
    for j in 0..3:
        s += a[j]
print(s)
