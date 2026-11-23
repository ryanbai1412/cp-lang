let (n, q): (1..=200000, 1..=200000) = read()
var d: [0..=n]Int = 0
for _ in 0..q:
    let (l, r, x): (1..=n, 1..=n, Int) = read()
    d[l - 1] += x  // expect: diff_by_hand
    d[r] -= x
var cur = 0
var out: List<Int> = []
for i in 0..n:  // expect: missing prefix-totals-list
    cur += d[i]
    out.push(cur)
print(out)
