let n: 3..=1000 = read()
let xs: [0..n]Int = read()
let ys: [0..n]Int = read()  // expect: missing tuple_points
var area2 = 0
for i in 0..n:
    let j = (i + 1) % n
    area2 += xs[i] * ys[j] - xs[j] * ys[i]  // expect: missing shoelace_by_hand
print(abs(area2))
