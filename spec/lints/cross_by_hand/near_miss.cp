let n: 3..=1000 = read()
let xs: [0..n]Int = read()
let ys: [0..n]Int = read()
for i in 1..n:
    print(xs[i] * ys[i] - xs[i - 1] * ys[i - 1])
