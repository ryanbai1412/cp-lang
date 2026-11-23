let (n, m): (1..=1000, 0..=1000) = read()
var adj: List<List<Int>> = [[] for _ in 0..n]
for _ in 0..m:
    let (a, b): (0..n, 0..n) = read()
    adj[a].push(b)
    adj[b].push(a)
var seen: List<Bool> = [false for _ in 0..n]
fn visit(u: Int):
    seen[u] = true
    for v in adj[u] if not seen[v]:
        visit(v)
for u in 0..n:
    if not seen[u]:
        visit(u)
print(yes(seen[0]))
