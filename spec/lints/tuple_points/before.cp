let n: 1..=1000 = read()
let xs: [0..n]Int = read()
let ys: [0..n]Int = read()
for i in 0..n:
    print(xs[i] - xs[0], ys[i] - ys[0])
