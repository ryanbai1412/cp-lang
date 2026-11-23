let n: 1..=100 = read()
let a: [0..n]Int = read()
var xs: List<Int> = []
for i in 0..n:  // expect: push_loop
    if a[i] > 0:
        xs.push(a[i])
for i in 0..xs.len:  // expect: index_only_loop
    print(xs[i])
