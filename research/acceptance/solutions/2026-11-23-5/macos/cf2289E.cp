@!multitest
let (n, d): (2..=2e5, 1..=n) = read()
let m = 2 * n - 1
var kids: [0..m](Int, Int) = (0, 0)
for k in n..m:
    let (x, y): (1..=2 * n - 2, 1..=2 * n - 2) = read()
    kids[k] = (x - 1, y - 1)
var dep: [0..m]Int = 0
for k in (n..m).rev:
    let (x, y) = kids[k]
    dep[x] = dep[k] + 1
    dep[y] = dep[k] + 1
var heaps: List<MaxHeap<Int>> = []
var hid: [0..m]Int = 0
for v in 0..n:
    var h = MaxHeap<Int>()
    h.push(v + 1)
    heaps.push(h)
    hid[v] = v
var ans: [0..n]Int = 0
for k in n..m:
    let (x, y) = kids[k]
    let (hx, hy) = (hid[x], hid[y])
    let mx = heaps[hx].peek()!
    let my = heaps[hy].peek()!
    while (heaps[hx].peek() ?? 0) > my + d - 1:
        let v = heaps[hx].pop()!
        ans[v - 1] = dep[v - 1] - dep[k]
    while (heaps[hy].peek() ?? 0) > mx + d - 1:
        let v = heaps[hy].pop()!
        ans[v - 1] = dep[v - 1] - dep[k]
    let (big, small) = if heaps[hx].len >= heaps[hy].len: (hx, hy) else: (hy, hx)
    while let v = heaps[small].pop():
        heaps[big].push(v)
    hid[k] = big
while let v = heaps[hid[m - 1]].pop():
    ans[v - 1] = dep[v - 1]
print(ans)
