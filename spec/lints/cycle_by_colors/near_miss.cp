let (n, m): (1..=100000, 0..=200000) = read()
var adj: List<List<Int>> = [[] for _ in 0..=n]
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    adj[u].push(v)
    adj[v].push(u)
var color: List<Int> = [0 for _ in 0..=n]
var odd = false
fn paint(u: Int, c: Int):
    color[u] = c
    for v in adj[u]:
        if color[v] == c:
            odd = true
        elif color[v] == 0:
            paint(v, 3 - c)
for u in 1..=n:
    if color[u] == 0:
        paint(u, 1)
print(yes(not odd))
