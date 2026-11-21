@! multitest
let (n, q): (Int, Int) = read()
let a: [1..=n]Int = read()

var prev_gt: [1..=n]?Int = none
var stk: List<Int> = []
for i in 1..=n:
    while let top = stk.last, a[top] < a[i]: let _ = stk.pop()
    prev_gt[i] = stk.last
    stk.push(i)

var bad: [1..=n]Int = n + 1
stk.clear()
for i in (1..=n).rev:
    while let top = stk.last, a[top] > a[i]: let _ = stk.pop()
    if let l = prev_gt[i], let r = stk.last:
        bad[l] min= r
    stk.push(i)

for i in (1..n).rev: bad[i] min= bad[i + 1]

for _ in 0..q:
    let (l, r): (Int, Int) = read()
    print(if r >= bad[l]: "NO" else: "YES")
