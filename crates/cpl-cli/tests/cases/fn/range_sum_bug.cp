// cp: 0.1 cc89938+dirty
let n: 1..=20 = read()
domain P = 1..=n
let a: [P]-100..=100 = read()

var pre: [0..=n]Int = 0
for i in P:
    pre[i - P.first! + 1] = pre[i - P.first!] + a[i]

fn range_sum(l: P, r: P) -> Int:
    if r < l: return 0 else: return pre[r - P.first!] - pre[l - P.first!]

@alt(range_sum)
fn range_sum_slow(l: P, r: P) -> Int:
    sum(a[i] for i in l..=r)

print(range_sum(P.first!, P.last!))
