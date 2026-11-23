// E. Arithmetic Subarray
// https://codeforces.com/contest/2285/problem/E
@!multitest
let (n, k): (2..=2e5, 1..=1500) = read()
let a: [0..n]1..=1e9 = read()
// every kept element lies on one line v = delta * i + c; consecutive kept ones are at most k + 1 apart
var ids: HashMap<(Int, Int), Int> = HashMap()
var last: List<Int> = []
var pos: List<List<Int>> = []
var left: List<Int> = []
var best = 1
for j in 0..n:
    for d in 1..=min(k + 1, j):
        let i = j - d
        let diff = a[j] - a[i]
        if diff % d != 0: continue
        let delta = diff / d
        let key = (delta, a[i] - delta * i)
        if key not in ids:
            ids[key] = pos.len
            pos.push([i])
            last.push(i)
            left.push(0)
        let id = ids[key]
        if last[id] == j: continue
        if last[id] != i:
            pos[id] = [i]
            left[id] = 0
        pos[id].push(j)
        last[id] = j
        let y = pos[id].len - 1
        while (pos[id][y] - y) - (pos[id][left[id]] - left[id]) > k:
            left[id] += 1
        best = max(best, y - left[id] + 1)
print(min(n, best + k))
