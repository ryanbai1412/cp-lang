let n: 3..=1000 = read()
let xs: [0..n]Int = read()
let ys: [0..n]Int = read()
var twice = 0
for i in 0..n:
    let j = (i + 1) % n
    twice += xs[i] * ys[j] - xs[j] * ys[i]
print(abs(twice))
