@!multitest
let n: Int = read()
var adj: [0..n]List<Int> = []
for _ in 0..(n - 1):
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
    adj[v - 1].push(u - 1)

var cur_deg: [0..n]Int = 0
var deg_freq: [0..n]Int = 0
for i in 0..n:
    cur_deg[i] = adj[i].len
    deg_freq[cur_deg[i]] += 1

var edge_cnt = 1
var ans = [1]
var leaves = [i for i in 0..n if cur_deg[i] == 1]
var alive: [0..n]Bool = true
var max_deg = n - 1
var colors = 2
while ans.len < n - 1:
    if colors % 2 == 0:
        while deg_freq[max_deg] == 0: max_deg -= 1
        edge_cnt += max_deg - 1
    else:
        edge_cnt -= max_deg - 1
        edge_cnt += leaves.len
        var nleaves: List<Int> = []
        for v in leaves:
            alive[v] = false
            deg_freq[cur_deg[v]] -= 1
            var neighbor = 0
            for nxt in adj[v]:
                if alive[nxt]: neighbor = nxt
            deg_freq[cur_deg[neighbor]] -= 1
            cur_deg[neighbor] -= 1
            deg_freq[cur_deg[neighbor]] += 1
            if cur_deg[neighbor] == 1: nleaves.push(neighbor)
        leaves = nleaves
    while ans.len < edge_cnt:
        ans.push(colors)
    colors += 1
print(ans)
