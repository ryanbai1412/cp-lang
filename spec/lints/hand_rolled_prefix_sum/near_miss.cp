let a = [1, 2, 3, 4]
var p = List.filled(a.len + 1, 0)
for i in 0..a.len:
    p[i + 1] = max(p[i], a[i])
print(p[3])
