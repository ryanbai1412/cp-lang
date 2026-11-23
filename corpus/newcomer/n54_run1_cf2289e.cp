// First draft from acceptance run 1 (cold agent, docs only), Codeforces 2289E.
@!multitest
let (n, d): (2..=2e5, 1..=2e5) = read()
let m = 2 * n - 1
var kids: List<(Int, Int)> = []  // expect: read_loop
for _ in 0..n - 1:
    let (x, y): (1..=m, 1..=m) = read()
    kids.push((x, y))
var depth: [0..=m]Int = 0  // expect: missing int-node-ids
for k in (n + 1..=m).rev:
    let (x, y) = kids[k - n - 1]
    depth[x] = depth[k] + 1
    depth[y] = depth[k] + 1
var sets: List<SortedSet<Int>> = []
var id: [0..=m]Int = 0
var top: [0..=m]Int = 0
for x in 1..=n:
    var s = SortedSet<Int>()
    s.add(x)
    sets.push(s)
    id[x] = x - 1
    top[x] = x
var ans: [0..=n]Int = 0
for k in n + 1..=m:
    let (x, y) = kids[k - n - 1]
    for (c, cap) in [(x, top[y] + d - 1), (y, top[x] + d - 1)]:
        while (sets[id[c]].last ?? 0) > cap:
            let w = sets[id[c]].last!
            sets[id[c]].remove(w)
            ans[w] = depth[w] - depth[k]
    var big = id[x]
    var small = id[y]
    if sets[big].len < sets[small].len:  // expect: missing manual-small-to-large
        (big, small) = (small, big)
    for w in sets[small]:
        sets[big].add(w)
    id[k] = big
    top[k] = sets[big].last!
for w in sets[id[m]]:
    ans[w] = depth[w]
print([ans[w] for w in 1..=n])
