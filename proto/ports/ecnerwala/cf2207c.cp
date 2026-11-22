// Codeforces 2207 C — Max-Cartesian Tree Water Drainage
//
// Build a max-Cartesian tree over columns `C = 0..n` with heights `a[C]`,
// tracking each node's optional left/right children and covered column span
// `[span_l[u]..=span_r[u]]`.
//
// For a subtree rooted at `u` beneath ceiling height `cur_h`, draining `u`
// (the tallest column in its span, height `nh = a[u]`) drains the shared water
// layer `w = (cur_h - nh) * (span_r[u] - span_l[u] + 1)` above `nh`, after
// which the left and right subtrees below `nh` are independent. Letting
// `(a0, a1)` and `(b0, b1)` be `(best with 1 drain, best with up to 2 drains)`
// in the left and right subtrees:
// - `best1 = max(a0, b0) + w` (1 drain in this subtree)
// - `best2 = max(a0 + b0, a1, b1) + w` (up to 2 drains in this subtree)

@!multitest
let (n, h): (1..=200_000, 1..=10 ** 9) = read()
domain C = 0..n
let a: [C]0..=h = read()

var left: [C]?C = none
var right: [C]?C = none
var span_l = [C] i => i.ord
var span_r = [C] i => i.ord

var stk: List<C> = []
for i in C:
    var last_popped: ?C = none
    while let top = stk.last and a[i] > a[top]:
        stk.pop()
        right[top] = last_popped
        if let r = last_popped: span_r[top] = span_r[r]
        last_popped = top
    left[i] = last_popped
    if let l = last_popped: span_l[i] = span_l[l]
    stk.push(i)

var root: ?C = none
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
