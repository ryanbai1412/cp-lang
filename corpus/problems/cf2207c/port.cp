// Source: Codeforces 2207C (Where's My Water?), https://codeforces.com/contest/2207/problem/C
// Problem: n columns (sum n <= 2000, multitest) under height h <= 1e9; column i has a_i >= 1
//   dirt tiles at the bottom and water above. A drain takes all water that reaches it moving
//   down, left or right without crossing dirt. Print the most water taken by at most 2 drains.
// Solution: the max-Cartesian tree of the heights. Under a ceiling cur_h, the subtree of the
//   tallest column u spans span_l..=span_r; any drain in it takes the layer of (cur_h - a[u])
//   times the span above u, and below a[u] its two child subtrees are independent. So
//   dfs(u) = (best with 1 drain, best with 2) = (max(a0, b0) + w, max(a0 + b0, a1, b1) + w)
//   from the children's pairs. Monotonic stack build and DFS, O(n).

@!multitest
let (n, h): (1..=200_000, 1..=10 ** 9) = read()
domain C = 0..n
let a: [C]0..=h = read()

var left: [C]?C = None
var right: [C]?C = None
var span_l = [C] i => i.ord
var span_r = [C] i => i.ord

var stk: List<C> = []
for i in C:
    var last_popped: ?C = None
    while let top = stk.last and a[i] > a[top]:
        stk.pop()
        right[top] = last_popped
        if let r = last_popped: span_r[top] = span_r[r]
        last_popped = top
    left[i] = last_popped
    if let l = last_popped: span_l[i] = span_l[l]
    stk.push(i)

var root: ?C = None
while let top = stk.pop():
    right[top] = root
    if let r = root: span_r[top] = span_r[r]
    root = top

fn dfs(u_opt: ?C, cur_h: Int) -> (Int, Int):
    if let u = u_opt:
        let nh = a[u]
        let w = (cur_h - nh) * (span_r[u] - span_l[u] + 1)
        let (a0, a1) = dfs(left[u], nh)
        let (b0, b1) = dfs(right[u], nh)
        (max(a0, b0) + w, max(a0 + b0, a1, b1) + w)
    else:
        (0, 0)

print(dfs(root, h).1)
