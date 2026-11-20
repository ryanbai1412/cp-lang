@!multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()
var cnt_ops: [0..=n]Int = 0
var cnt_trade: [0..=n]Int = 0
for i in 0..n:
    cnt_ops[i + 1] = cnt_ops[i] + bit_width(a[i] - 1)
    cnt_trade[i + 1] = cnt_trade[i] + (if a[i] == 2: 1 else: bit_width(a[i] - 2))
for _ in 0..q:
    let (l1, r): (Int, Int) = read()
    let l = l1 - 1
    print((cnt_trade[r] - cnt_trade[l] + cnt_ops[r] - cnt_ops[l]) / 2)
