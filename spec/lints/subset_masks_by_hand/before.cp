let n: 1..=20 = read()
let a: [0..n]Int = read()
var best = 0
for mask in 0..(1 << n):
    let s = sum(a[i] for i in 0..n if mask.bit(i))
    if s % 3 == 0:
        best max= s
print(best)
