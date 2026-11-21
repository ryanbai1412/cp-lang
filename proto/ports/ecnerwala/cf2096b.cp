@!multitest
let (n, k): (Int, Int) = read()
let l: [0..n]Int = read()
let r: [0..n]Int = read()
let mins = sorted([min(l[i], r[i]) for i in 0..n], desc: true)
print(sum(max(l[i], r[i]) for i in 0..n) + mins[0..k - 1].sum() + 1)
