let (n, q): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var p: [0..=n]Int = 0
for i in 0..n:
    p[i + 1] = p[i] + a[i]  // expect: hand_rolled_prefix_sum
for _ in 0..q:
    let (l, r): (1..=n, 1..=n) = read()
    print(p[r] - p[l - 1])  // expect: query_index_shift
