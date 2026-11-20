@!multitest
let (n, k): (Int, Int) = read()
var par: [0..n]Int = 0
for i in 1..n: par[i] = read::<Int>() - 1
var depth: [0..n]Int = 0
for i in 1..n: depth[i] = depth[par[i]] + 1
var sz: [0..n]Int = 1
for i in (1..n).rev: sz[par[i]] += sz[i]

var leaf_depth = depth[n - 1]
for i in 0..n:
    if sz[i] == 1: leaf_depth min= depth[i]

var freq: [0..n]Int = 0
for i in 0..n: freq[depth[i]] += 1

fn is_good(l: Int) -> Bool:
    var ff: [0..=n]Int = 0
    for i in 0..l: ff[freq[i]] += 1
    for i in l..n: ff[1] += freq[i]
    var pos: [0..=n]Int = 0
    pos[0] = 1
    var npos: [0..=n]Int = 0
    for i in 1..=n:
        if ff[i] == 0: continue
        for rem in 0..i:
            var ways = 0
            var v = rem
            while v <= n:
                ways += pos[v]
                npos[v] = Int(ways > 0)
                if v >= ff[i] * i: ways -= pos[v - ff[i] * i]
                v += i
        swap(mut pos, mut npos)
    pos[k] != 0

var mi = 0
var ma = leaf_depth + 2
while ma - mi > 1:
    let md = (mi + ma) / 2
    if is_good(md): mi = md else: ma = md
print(mi)
