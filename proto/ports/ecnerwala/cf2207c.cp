// Codeforces 2207 C: build a max-Cartesian tree over column heights and run a
// 2-state tree DP tracking the best 1-bucket and 2-bucket drained water volumes.
struct Node(l: Int, m: Int, r: Int, c0: ?Int, c1: ?Int)

@! multitest
let (n, h): (Int, Int) = read()
let a: [0..n]Int = read()

var nodes: [0..=2 * n]Node = Node(0, 0, 0, none, none)
var stk: List<Int> = []
var root = 0
for i in 0..=n:
    var cur = 2 * i
    nodes[cur] = Node(i, i - 1, i - 1, none, none)
    while let top = stk.last, (i == n or a[i] > a[nodes[top].m]):
        stk.pop()
        nodes[top].c1 = cur
        nodes[top].r = nodes[cur].r
        cur = top
    if i == n:
        root = cur
        break
    nodes[2 * i + 1] = Node(nodes[cur].l, i, i, cur, none)
    stk.push(2 * i + 1)

fn dfs(cur: Int, cur_h: Int) -> (Int, Int):
    if cur % 2 == 0: return (0, 0)
    let nh = a[cur / 2]
    let w = (cur_h - nh) * (nodes[cur].r - nodes[cur].l + 1)
    let (a0, a1) = dfs(nodes[cur].c0!, nh)
    let (b0, b1) = dfs(nodes[cur].c1!, nh)
    (max(a0, b0) + w, max(max(a0 + b0, a1), b1) + w)

print(dfs(root, h).1)
