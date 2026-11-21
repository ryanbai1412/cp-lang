// Codeforces 2207 B: simulate each second by incrementing the earliest minimum
// element of the active window and dropping the front element on flashlight times.
@!multitest
let (n, m_in, l): (Int, Int, Int) = read()
let a: [0..n]Int = read()

let m0 = min(m_in, n + 1)
var vals: [0..m0]Int = 0
var m = m0
var idx = 0

for t in 1..=l:
    let best = (0..m).min(by: i => vals[i])!
    vals[best] += 1
    if idx < n and t == a[idx]:
        idx += 1
        for i in 0..m - 1: vals[i] = vals[i + 1]
        vals[m - 1] = 0
        m min= n - idx + 1

print(vals[0])
