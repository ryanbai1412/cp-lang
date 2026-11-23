let n: 1..=1000 = read()
let xs: [0..n]Int = read()
let ys: [0..n]Int = read()
let p = [0..n] i => Pt(xs[i], ys[i])
for i in 0..n:
    let d = p[i] - p[0]
    print(d.x, d.y)
