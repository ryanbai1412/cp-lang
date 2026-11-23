use std.heap.{MaxHeap}
@!multitest
let (n, d): (2..=2e5, 1..=2e5) = read()
let ms: [0..n - 1](1..=4e5, 1..=4e5) = read()
let root = 2 * n - 1
var dep: [0..2 * n]Int = 0
dep[root] = 1
for k in (n + 1..=root).rev:
    let (x, y) = ms[k - n - 1]
    dep[x] = dep[k] + 1
    dep[y] = dep[k] + 1
var heaps: List<MaxHeap<Int>> = []
var hid: [0..2 * n]Int = 0
for x in 1..=n:
    var h = MaxHeap<Int>()
    h.push(x)
    hid[x] = heaps.len
    heaps.push(h)
var ans: [0..n + 1]Int = 0
for k in n + 1..=root:
    let (x, y) = ms[k - n - 1]
    let mx = heaps[hid[x]].peek!
    let my = heaps[hid[y]].peek!
    while (heaps[hid[x]].peek ?? 0) >= my + d:
        let w = heaps[hid[x]].pop()!
        ans[w] = dep[w] - dep[k]
    while (heaps[hid[y]].peek ?? 0) >= mx + d:
        let w = heaps[hid[y]].pop()!
        ans[w] = dep[w] - dep[k]
    var big = hid[x]
    var small = hid[y]
    if heaps[big].len < heaps[small].len: (big, small) = (small, big)
    while let w = heaps[small].pop():
        heaps[big].push(w)
    hid[k] = big
while let w = heaps[hid[root]].pop():
    ans[w] = dep[w] - 1
print([ans[x] for x in 1..=n])
