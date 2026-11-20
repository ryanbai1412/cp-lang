// Codeforces 2207 B: simulate each second by incrementing the earliest minimum
// element of the active window and dropping the front element on flashlight times.
@!multitest
let (n, m_in, l): (Int, Int, Int) = read()
let a: [0..n]Int = read()

var idx = 0
var m = min(m_in, n + 1)
var vals: List<Int> = List.filled(m, 0)
for t in 1..=l:
    var best = 0
    for i in 1..vals.len:
        if vals[i] < vals[best]: best = i
    vals[best] += 1
    if idx < n and t == a[idx]:
        idx += 1
        let nm = min(m, n - idx + 1)
        var next_vals = vals[1..vals.len]
        while next_vals.len < nm: next_vals.push(0)
        while next_vals.len > nm: next_vals.pop()
        vals = next_vals
        m = nm
print(vals[0])
