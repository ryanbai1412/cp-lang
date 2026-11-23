// NOI 2003 P4, the detour: longest dist(c, nearer of a/b) + dist(a, b) on a weighted tree (buildable today-form)
// original: corpus/submissions/5032732.cpp
// The walk is hand-rolled: the prototype's bfs has no edge weights.
let n: 1..=2e5 = read()
let m: 0..=2e5 = read()
domain Home = 1..=n
var adj: [Home]List<(Home, Int)> = []
for _ in 0..m:
    let u: Home = read()
    let v: Home = read()
    let w: 1..=1e9 = read()
    adj[u].push((v, w))
    adj[v].push((u, w))

fn dists(src: Home) -> [Home]Int:
    var d: [Home]Int = -1
    var stack: List<Home> = [src]
    d[src] = 0
    while let u = stack.pop():
        for (v, w) in adj[u]:
            if d[v] < 0:
                d[v] = d[u] + w
                stack.push(v)
    d

fn farthest(d: [Home]Int) -> Home:
    var best = Home.first
    for v in Home:
        if d[v] > d[best]: best = v
    best

// a and b are the two ends of a diameter, and the third home is whoever is
// farthest from the nearer of the two
let a = farthest(dists(Home.first!))
let da = dists(a)
let b = farthest(da)
let db = dists(b)
var detour = 0
for c in Home: detour max= min(da[c], db[c])
print(da[b] + detour)
