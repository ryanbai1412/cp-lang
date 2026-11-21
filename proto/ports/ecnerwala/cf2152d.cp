@!multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()
let cnt_ops = prefix_n([bit_width(x - 1) for x in a])
let cnt_trade = prefix_n([if x == 2: 1 else: bit_width(x - 2) for x in a])

for _ in 0..q:
    let (l, r): (Int, Int) = read()
    print((cnt_trade.sum(l - 1..r) + cnt_ops.sum(l - 1..r)) / 2)
