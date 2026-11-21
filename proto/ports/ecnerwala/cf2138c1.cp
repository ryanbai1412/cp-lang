@!multitest
let (n, k): (Int, Int) = read()
let par = [0..n] i => if i == 0: 0 else: read::<Int>() - 1

var depth: [0..n]Int = 0
for i in 1..n: depth[i] = depth[par[i]] + 1
var sz: [0..n]Int = 1
for i in (1..n).rev: sz[par[i]] += sz[i]

let leaf_depth = min(depth[i] for i in 0..n if sz[i] == 1)
var freq: [0..n]Int = 0
for d in depth: freq[d] += 1

fn is_good(l: Int) -> Bool:
    var ff: [0..=n]Int = 0
    for i in 0..l: ff[freq[i]] += 1
    for i in l..n: ff[1] += freq[i]
    var pos: [0..=n]Int = 0
    pos[0] = 1
    var npos: [0..=n]Int = 0
    for i in 1..=n if ff[i] > 0:
        for rem in 0..i:
            var ways = 0
            for v in rem..=n by i:
                ways += pos[v]
                npos[v] = Int(ways > 0)
                if v >= ff[i] * i: ways -= pos[v - ff[i] * i]
        swap(mut pos, mut npos)
    pos[k] != 0

print(bsearch_last(0..leaf_depth + 2, is_good)!)
