// cf2009G1 Yunli's Subarray Queries (easy): a window becomes consecutive by keeping the
// most frequent value of a[i] - Int(i), so f = k - (largest count in the window). Slide a
// window of k with a counter and a count of counts.
@!multitest
let (n, k, q): (1..=2e5, 1..=n, 1..=2e5) = read()
domain P = 1..=n
let a = read<[P]1..=n>()
let key = [P] i => a[i] - Int(i)

var cnt = Counter<Int>(0)
var with_count: [0..=k]Int = 0        // values that occur exactly c times
var top = 0
var f: [P]Int = 0                     // f of the window starting at l
for r in P:
    if r - k >= 1:
        let y = key[P(r - k)]
        let c = cnt.get(y)
        with_count[c] -= 1
        if c == top and with_count[c] == 0: top -= 1
        cnt.add(y, -1)
        with_count[c - 1] += 1
    let x = key[r]
    with_count[cnt.get(x)] -= 1
    cnt.add(x, 1)
    with_count[cnt.get(x)] += 1
    top max= cnt.get(x)
    if Int(r) >= k: f[P(r - k + 1)] = k - top
repeat q:
    let (l, _r): (P, P) = read()
    print(f[l])
