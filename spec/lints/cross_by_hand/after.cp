let n: 3..=1000 = read()
let xs: [0..n]Int = read()
let ys: [0..n]Int = read()
let poly = [Pt(xs[i], ys[i]) for i in 0..n]
print(abs(area2(poly)))
