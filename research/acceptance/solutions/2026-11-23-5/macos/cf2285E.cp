@!multitest
let (n, k): (2..=2e5, 1..=min(n, 1500)) = read()
let a: [0..n]1..=1e9 = read()
var ids: HashMap<(Int, Int), Int> = HashMap()
var lines: List<List<Int>> = []
for j in 0..n:
    for gap in 1..=min(k + 1, j):
        let i = j - gap
        let diff = a[j] - a[i]
        if diff % gap != 0: continue
        let step = diff / gap
        let key = (step, a[j] - step * j)
        if let id = ids.get(key):
            let last = lines[id][lines[id].len - 1]
            if last == j: continue
            if last < i: lines[id].push(i)
            lines[id].push(j)
        else:
            ids[key] = lines.len
            lines.push([i, j])
var best = 1
for line in lines:
    var p = 0
    for q in 0..line.len:
        while (line[q] - q) - (line[p] - p) > k: p += 1
        best max= q - p + 1
print(min(n, best + k))
