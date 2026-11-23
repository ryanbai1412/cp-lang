let (n, m): (1..=200000, 1..=200000) = read()
var edges: List<(Int, Int)> = []
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    edges.push((u, v))
print(edges.len)
