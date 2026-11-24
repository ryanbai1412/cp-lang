
@!multitest
let (n, d): (2..=2e5, 1..=2e5) = read()
let m = 2 * n - 1
var kids = [m + 1].filled((0, 0))
for k in n + 1..=m:
    let (x, y): (1..=2 * n - 2, 1..=2 * n - 2) = read()
    kids[k] = (x, y)
var depth = [m + 1].filled(0)
depth[m] = 1
for k in (n + 1..=m).rev:
    depth[kids[k].0] = depth[k] + 1
    depth[kids[k].1] = depth[k] + 1
var ans = [n + 1].filled(0)
for x in 1..=n: ans[x] = depth[x] - 1
var pool = [SortedSet.of([x]) for x in 0..=n]
var id = [m + 1].filled(0)
for x in 1..=n: id[x] = x
for k in n + 1..=m:
    var a = id[kids[k].0]
    var b = id[kids[k].1]
    let wa = pool[a].last!
    let wb = pool[b].last!
    for (s, lim) in [(a, wb + d), (b, wa + d)]:
        while (pool[s].last ?? 0) >= lim:
            let x = pool[s].pop_last()!
            ans[x] = depth[x] - depth[k]
    if pool[a].len < pool[b].len: (a, b) = (b, a)
    for x in pool[b]: pool[a].add(x)
    id[k] = a
print(ans[1..])
