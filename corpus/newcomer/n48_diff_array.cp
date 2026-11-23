let (n, q): (1..=200000, 1..=200000) = read()
var d: [0..=n]Int = 0
for _ in 0..q:
    let (l, r, x): (1..=n, 1..=n, Int) = read()
    d[l - 1] += x  // expect: missing manual-difference-array
    d[r] -= x
var cur = 0
var out: List<Int> = []
for i in 0..n:  // expect: hand_rolled_prefix_sum
    cur += d[i]
    out.push(cur)
print(out)
