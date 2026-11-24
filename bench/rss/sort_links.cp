let n: 1..=1_000_000 = read()
domain F = 0..n
domain G = 0..n
let f: [F]1..=100_000_000 = read()
let g: [G]1..=100_000_000 = read()
enum Link:
    Flight(f: F)
    Portal(g: G)
let links = [(f[x], Flight(x)) for x in F] + [(g[y], Portal(y)) for y in G]
var tot = 0
for (i, (c, l)) in links.sorted(by: l => l.0).items:
    match l:
        Flight(x) => tot += i * x.ord
        Portal(y) => tot -= i * y.ord
print(tot)
