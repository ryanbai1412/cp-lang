let a: List<Int> = [1, 2, 3]
var s = 0
for j in 0..a.len:
    s += a[j]
print(s)
var d = 0
for k in 0..2:
    for j in 1..a.len:
        d += k * a[j]
print(d)
