// expect: L0521 at 3
let a = [1, 2, 3, 4]
var p: [0..=4]Int = 0
for i in 0..a.len:
    p[i + 1] = p[i] + a[i]
print(p[3] - p[1])
