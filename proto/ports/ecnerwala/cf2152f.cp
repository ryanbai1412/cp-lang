@!multitest
let (n, z): (Int, Int) = read()
let x: [0..n]Int = read()
let q: Int = read()
var qs: List<(Int, Int)> = []
for _ in 0..q:
    let (l, r): (Int, Int) = read()
    qs.push((l - 1, r))

var depth: [0..=n]Int = 0
var par: [0..n]Int = 0
var jloc: [0..n]Int = 0
var jdist: [0..n]Int = 0
var j = n
for i in (0..n).rev:
    while x[j - 1] - x[i] > z: j -= 1
    par[i] = j
    depth[i] = depth[j] + 1
    if j != n and jloc[j] != n and jdist[j] == jdist[jloc[j]]:
        jdist[i] = 1 + jdist[j] * 2
        jloc[i] = jloc[jloc[j]]
    else:
        jdist[i] = 1
        jloc[i] = j

var depth2: [0..=n]Int = 0
var par2: [0..n]Int = 0
var jloc2: [0..n]Int = 0
var jdist2: [0..n]Int = 0
for i in (0..n).rev:
    var a = i
    var b = i + 1
    if depth[a] > depth[b]: a = par[a]
    if a != b:
        while par[a] != par[b]:
            if jloc[a] != jloc[b]:
                a = jloc[a]
                b = jloc[b]
            else:
                a = par[a]
                b = par[b]
        a = par[a]
    let lca = a
    depth2[i] = depth2[lca] + (depth[i] + depth[i + 1] - 2 * depth[lca])
    par2[i] = lca
    if lca != n and jloc2[lca] != n and jdist2[lca] == jdist2[jloc2[lca]]:
        jdist2[i] = 1 + jdist2[lca] * 2
        jloc2[i] = jloc2[jloc2[lca]]
    else:
        jdist2[i] = 1
        jloc2[i] = lca

for (l0, r) in qs:
    var cur = l0
    while par2[cur] < r:
        if jloc2[cur] < r: cur = jloc2[cur]
        else: cur = par2[cur]
    var ans = depth2[l0] - depth2[cur]
    let l = cur
    for st in [l, l + 1]:
        if st >= r: continue
        var c = st
        while c < r:
            if jloc[c] < r: c = jloc[c]
            else: c = par[c]
        ans += depth[st] - depth[c]
    print(ans)
