@!multitest
let (n, d): (2..=2e5, 1..=2e5) = read()
let total = 2 * n - 1
var parent = [0 for _ in 0..=total]
var kids = [(0, 0) for _ in 0..=total]
for k in n + 1..=total:
    let (x, y): (1..=total, 1..=total) = read()
    kids[k] = (x, y)
    parent[x] = k
    parent[y] = k
var depth = [0 for _ in 0..=total]
for k in (n + 1..total).rev:
    depth[k] = depth[parent[k]] + 1
var heaps = [MaxHeap<Int>() for _ in 0..=total]
for i in 1..=n:
    heaps[i].push(i)
var hid = [i for i in 0..=total]
var out = [0 for _ in 0..=total]
for k in n + 1..=total:
    let (x, y) = kids[k]
    var ha = hid[x]
    var hb = hid[y]
    let ta = heaps[hb].peek! + d - 1
    let tb = heaps[ha].peek! + d - 1
    while (heaps[ha].peek ?? 0) > ta:
        out[heaps[ha].pop()!] = k
    while (heaps[hb].peek ?? 0) > tb:
        out[heaps[hb].pop()!] = k
    if heaps[ha].len < heaps[hb].len:
        (ha, hb) = (hb, ha)
    while let v = heaps[hb].pop():
        heaps[ha].push(v)
    hid[k] = ha
print([depth[parent[i]] + 1 - (if out[i] == 0: 0 else: depth[out[i]]) for i in 1..=n])
