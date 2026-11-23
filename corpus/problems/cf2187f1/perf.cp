// Tuned: orig.cpp's O(n) permutation-tree build, one stack pass guided by each value's nearest
// earlier neighbours, in flat narrow arrays; std.tree's PermTree finds intervals with a lazy
// segment tree instead (O(n log n)).
@!multitest
let n: 2..=1_000_000 = read()
let a: [0..n]1..=n = read()
let b: [0..n]1..=n = read()
let m = 2 * n - 1
var pos_a: [0..n]0..n = 0
for i in 0..n: pos_a[a[i] - 1] = i
var p: [0..n]0..n = 0
for i in 0..n: p[i] = pos_a[b[n - 1 - i] - 1]
// `prv[v]` / `nxt[v]`: the nearest smaller / larger value at an earlier position than v's
var prv: [0..n]-1..n = 0
var nxt: [0..n]0..=n = 0
for v in 0..n: (prv[v], nxt[v]) = (v - 1, v + 1)
for i in (0..n).rev:
    let v = p[i]
    let (lo, hi) = (prv[v], nxt[v])
    if lo != -1: nxt[lo] = hi
    if hi != n: prv[hi] = lo
enum Kind: Leaf | Incr | Decr | Full | Partial
var kind: [0..m]Kind = Leaf
var kid0: [0..m]0..m = 0
var kid1: [0..m]0..m = 0
// the stack of open runs: first position, value range, the gap it may still grow into, node
var s_left: [0..n]0..n = 0
var s_lo: [0..n]0..n = 0
var s_hi: [0..n]0..n = 0
var s_lo_gap: [0..n]0..=n = 0
var s_hi_gap: [0..n]0..=n = 0
var s_node: [0..n]0..m = 0
var k = 0
fn join(partial: Bool):
    let (x, y) = (k - 2, k - 1)
    s_lo[x] = min(s_lo[x], s_lo[y])
    s_hi[x] = max(s_hi[x], s_hi[y])
    let v = 2 * s_left[y] - 1
    kid0[v] = s_node[x]
    kid1[v] = s_node[y]
    kind[v] = if partial: Partial elif s_lo[x] == s_lo[y]: Decr elif s_hi[x] == s_hi[y]: Incr else: Full
    s_node[x] = v
    k -= 1
for i in 0..n:
    let v = p[i]
    while k > 0 and (v < s_lo_gap[k - 1] or v > s_hi_gap[k - 1]): join(true)
    s_left[k] = i
    s_lo[k] = v
    s_hi[k] = v
    s_lo_gap[k] = prv[v] + 1
    s_hi_gap[k] = nxt[v] - 1
    s_node[k] = 2 * i
    k += 1
    while k >= 2 and max(s_hi[k - 1], s_hi[k - 2]) - min(s_lo[k - 1], s_lo[k - 2]) == i - s_left[k - 2]:
        join(false)
fn depth(v: Int) -> Int:
    if kind[v] == Leaf: return 1
    if kind[v] == Decr and kind[kid1[v]] == Leaf: return 1 + depth(kid0[v])
    max(depth(kid0[v]), depth(kid1[v]))
print(depth(s_node[0]))
