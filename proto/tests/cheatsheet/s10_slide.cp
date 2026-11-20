let (n, cap): (1..=10, 1..=100) = read()
let a: [0..n]Int = read()
let pre = prefix(a)
var out = ""
for (lo, hi) in slide_until(0..n, (l, r) => pre.sum(l..=r) <= cap): out += "{lo}:{hi} "
print(out)
