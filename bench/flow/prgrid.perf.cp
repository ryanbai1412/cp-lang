// Tuned: prgrid.cp's highest-label push-relabel written out on dense ids
// (source 0, sink 1, cell 2 + r * 400 + c), the same steps as
// `FlowGraph.push_relabel`: arcs in insertion order, arc a's reverse is a ^ 1.
let (rows, cols) = (400, 400)
let n = rows * cols + 2
domain V = 0..n
var head: [V]List<Int> = []
var to: List<0..160_002> = []
var left: List<Int> = []
fn add(u: Int, v: Int, c: Int):
    head[V(u)].push(to.len)
    to.push(v)
    left.push(c)
    head[V(v)].push(to.len)
    to.push(u)
    left.push(0)

fn cell(r: Int, c: Int) -> Int = 2 + r * cols + c
for r in 0..rows:
    for c in 0..cols:
        add(if r > 0: cell(r - 1, c) else: 0, cell(r, c), 1)
        add(cell(r, c), if c > 0: cell(r, c - 1) else: 1, 1)

var h: List<Int> = List.filled(n, 0)
var ex: List<Int> = List.filled(n, 0)
var nx: List<Int> = List.filled(n, 0)
var active: List<List<Int>> = [[] for _ in 0..2 * n]
var cnt: List<Int> = List.filled(2 * n, 0)
fn push(a: Int, f: Int):
    unchecked:
        let v = to[a]
        if ex[v] == 0 and f > 0: active[h[v]].push(v)
        left[a] -= f
        left[a ^ 1] += f
        ex[v] += f
        ex[to[a ^ 1]] -= f

h[0] = n
ex[1] = 1
cnt[0] = n - 1
for a in head[V(0)]: push(a, left[a])
var top = 0
var flow = 0
unchecked:
    loop:
        while active[top].len == 0 and top > 0: top -= 1
        if active[top].len == 0:
            flow = -ex[0]
            break
        let u = active[top].pop()!
        let arcs = head[V(u)]
        while ex[u] > 0:
            if nx[u] == arcs.len:
                h[u] = Int.MAX
                for k in 0..arcs.len:
                    let a = arcs[k]
                    if left[a] > 0 and h[to[a]] < h[u] - 1:
                        h[u] = h[to[a]] + 1
                        nx[u] = k
                cnt[h[u]] += 1
                cnt[top] -= 1
                if cnt[top] == 0 and top < n:
                    for v in 0..n:
                        if top < h[v] and h[v] < n:
                            cnt[h[v]] -= 1
                            h[v] = n + 1
                top = h[u]
            else:
                let a = arcs[nx[u]]
                if left[a] > 0 and h[u] == h[to[a]] + 1: push(a, min(ex[u], left[a]))
                else: nx[u] += 1
print(flow)
