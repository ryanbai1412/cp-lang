// CF 1933F Turtle Mission: in the rocks' frame (row + time), rocks stand still, "down" moves
// two rows and "right" moves one row down and one column right; "up" only matters in the
// rock-free last column, where the remaining rows are walked directly.
@!multitest
let n: 3..=1e3 = read()
let m: 3..=1e3 = read()
let rock = read<[n][m]0..=1>()
var dist = [n * m].filled(-1)
var queue = List<Int>()
dist[0] = 0
queue.push(0)
var head = 0
while head < queue.len:
    let s = queue[head]
    head += 1
    let x = s / m
    let y = s % m
    let down = (x + 2) % n
    if rock[(x + 1) % n][y] == 0 and rock[down][y] == 0 and dist[down * m + y] == -1:
        dist[down * m + y] = dist[s] + 1
        queue.push(down * m + y)
    if y + 1 < m and rock[(x + 1) % n][y + 1] == 0:
        let r = ((x + 1) % n) * m + y + 1
        if dist[r] == -1:
            dist[r] = dist[s] + 1
            queue.push(r)
var best: ?Int = None
for x in 0..n:
    let t = dist[x * m + m - 1]
    if t >= 0:
        let gap = (n - 1 - (x - t)) % n
        best min= t + min(gap, n - gap)
print(best ?? -1)
