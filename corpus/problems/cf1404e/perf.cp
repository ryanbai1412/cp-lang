// Tuned: orig.cpp's push-relabel written out with each vertex's arcs stored together
// (destination, reverse index and capacity in one struct), where std.flow keeps arc ids
// into parallel lists, and with plain Int vertices instead of an enum.
let (n, m): (1..=200, 1..=200) = read()
domain Row = 0..n
domain Col = 0..m
domain Square = (Row, Col)
let grid: [Row, Col]Char = read()
let black = [Row, Col] (r, c) => grid[r, c] == '#'
let count = n * m + 2
let (src, snk) = (n * m, n * m + 1)
struct Arc:
    dest: Int
    back: Int
    var cap: Int
var g = List.filled(count, List<Arc>())
fn add_arc(s: Int, t: Int):
    g[s].push(Arc(t, g[t].len, 1))
    g[t].push(Arc(s, g[s].len - 1, 0))
for (r, c) in Square if black[r, c]:
    let at = r.ord * m + c.ord
    add_arc(if let rp = r.prev and black[rp, c]: rp.ord * m + c.ord else: src, at)
    add_arc(at, if let cp = c.prev and black[r, cp]: r.ord * m + cp.ord else: snk)
var excess = List.filled(count, 0)
var cur = List.filled(count, 0)
var active = List.filled(2 * count, List<Int>())
var height = List.filled(count, 0)
fn add_flow(u: Int, k: Int, f: Int):
    unchecked:
        let e = g[u][k]
        if excess[e.dest] == 0 and f != 0: active[height[e.dest]].push(e.dest)
        g[u][k].cap -= f
        excess[e.dest] += f
        g[e.dest][e.back].cap += f
        excess[u] -= f
fn max_flow() -> Int:
    height[src] = count
    excess[snk] = 1
    var co = List.filled(2 * count, 0)
    co[0] = count - 1
    for k in 0..g[src].len: add_flow(src, k, g[src][k].cap)
    var hi = 0
    unchecked:
        while true:
            while active[hi].is_empty:
                if hi == 0: return -excess[src]
                hi -= 1
            let u = active[hi].pop()!
            while excess[u] > 0:
                if cur[u] == g[u].len:
                    height[u] = 1_000_000_000
                    for k in 0..g[u].len:
                        let e = g[u][k]
                        if e.cap > 0 and height[u] > height[e.dest] + 1:
                            height[u] = height[e.dest] + 1
                            cur[u] = k
                    co[height[u]] += 1
                    co[hi] -= 1
                    if co[hi] == 0 and hi < count:
                        for i in 0..count:
                            if hi < height[i] and height[i] < count:
                                co[height[i]] -= 1
                                height[i] = count + 1
                    hi = height[u]
                else:
                    let e = g[u][cur[u]]
                    if e.cap > 0 and height[u] == height[e.dest] + 1:
                        add_flow(u, cur[u], min(excess[u], e.cap))
                    else:
                        cur[u] += 1
    0
print(max_flow())
