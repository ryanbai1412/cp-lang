let (n, cap): (1..=10, 1..=100) = read()
domain I = 0..n
let a: [I]Int = read()
let pre = prefix(a)
var out = ""
for (lo, hi) in slide_until(I.all, (l, r) => pre.sum(l..=r) >= cap): out += "{lo}:{hi} "
print(out)
