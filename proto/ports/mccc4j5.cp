// cp: 0.1 8ef64b9+dirty
let n: 1..=2e5 = read()
domain V = 1..=n
let tasks: [V](time: 0..=1e4, deps: List<V>) = read()

var users: [V]List<V> = []
for u in V:
    for v in tasks[u].deps:
        users[v].push(u)
var left: [V]Int = [V] u => tasks[u].deps.len

var pq = MinHeap<(Int, V)>()
for u in V:
    if left[u] == 0:
        pq.push((tasks[u].time, u))
var ans = 0
while let (d, u) = pq.pop():
    ans = d
    for v in users[u]:
        left[v] -= 1
        if left[v] == 0:
            pq.push((d + tasks[v].time, v))
print(ans)
