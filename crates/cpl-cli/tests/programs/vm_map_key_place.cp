let n: Int = read()
var adj: Map<Int, List<Int>> = []
var hadj: HashMap<Int, List<Int>> = []
var grid: Map<Int, List<Int>> = []
for k in 0..4:
    adj[k] = []
    hadj[k] = [k]
    grid[k] = [0, 0, 0]
for i in 0..n:
    adj[i % 4].push(i)
    hadj[(i * 7) % 4].extend([i, -i])
    grid[i % 4][i % 3] += i
    grid[i % 2][0] = grid[i % 2][0] * 2 % 1000
hadj[2].swap(0, 1)
adj[3].reverse()
print(adj, grid)
for k in 0..4: print(k, hadj[k], adj[k].len)
