// CF1945F Kirill and Mushrooms. Using k mushrooms zeroes p[1..k-1]; the best
// strength for k is k times the k-th largest remaining power, found with a
// Fenwick tree over the powers in descending order.
@!multitest
let n: 1..=2e5 = read()
domain V = 1..=n
let v = read<[V]1..=1e9>()
let p = read<[V]V>()
let ord = v.keys.sorted(by: i => desc(v[i]))
domain R = [n]
var rk: [V]Int = 0
for (j, i) in ord.items: rk[i] = j
var fw = Fenwick([R] _ => 1)
var best = 0
var bestk = 0
for k in 1..=n:
    if k >= 2: fw.add(R(rk[p[V(k - 1)]]), -1)
    let r = fw.kth(k) else: break
    let s = k * v[ord[r.ord]]
    if s > best:
        best = s
        bestk = k
print(best, bestk)
