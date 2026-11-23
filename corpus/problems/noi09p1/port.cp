// Source: NOI 2009 (Transformed Sequence), https://dmoj.ca/problem/noi09p1
// Problem: given D[i] for i in 0..n (n <= 1e4), find a permutation T of 0..n with circular
//   distance min(|i - T[i]|, n - |i - T[i]|) = D[i] for every i; print the lexicographically
//   smallest one, or "No Answer".
// Solution: T[i] is (i - D[i]) mod n or (i + D[i]) mod n, so each position is an edge between
//   its two candidate values and T picks one endpoint per edge, each value once. A value of
//   degree 1 is forced: peel those with a queue. What is left must be disjoint cycles; going
//   through positions in order, the first unassigned edge of a cycle takes its smaller value,
//   which orients the whole cycle. O(n).
// original: corpus/submissions/5102817.cpp

let n: Int = read()
domain Pos = 0..n
domain Val = 0..n

let dist: [Pos]Int = read()
let ends: [Pos](Val, Val) = [Pos] i => (
    Val((i.to_idx - dist[i]) % n),
    Val((i.to_idx + dist[i]) % n),
)

var adj: [Val]List<(to: Val, edge: Pos)> = []
var deg: [Val]Int = 0
for i in Pos:
    let (u, v) = ends[i]
    adj[u].push((to: v, edge: i))
    adj[v].push((to: u, edge: i))
    deg[u] += 1
    deg[v] += 1

var ans: [Pos]?Val = None
var used_val: [Val]Bool = false
var impossible = false

fn assign(i: Pos, u: Val):
    if ans[i] != None or used_val[u]:
        impossible = true
        return
    ans[i] = u
    used_val[u] = true

// Peel forced degree-1 values.
var leaves = [u for u in Val if deg[u] == 1]
var head = 0
while head < leaves.len and not impossible:
    let u = leaves[head]
    head += 1
    for a in adj[u]:
        if ans[a.edge] == None:
            assign(a.edge, u)
            deg[u] -= 1
            deg[a.to] -= 1
            if deg[a.to] == 1: leaves.push(a.to)

// Orient each remaining cycle starting from its smallest position index.
for i in Pos:
    if impossible or ans[i] != None: continue
    let (u, v) = ends[i]
    let first = min(u, v)
    let other = if first == u: v else: u
    assign(i, first)
    var cur = other
    while cur != first and not impossible:
        var moved = false
        for a in adj[cur]:
            if ans[a.edge] == None:
                assign(a.edge, cur)
                cur = a.to
                moved = true
                break
        if not moved:
            impossible = true
            break

for i in Pos:
    if ans[i] == None: impossible = true
for u in Val:
    if not used_val[u]: impossible = true

if impossible:
    print("No Answer")
else:
    var line = ""
    for i in Pos:
        if i.to_idx > 0: line += " "
        line += "{ans[i]!.to_idx}"
    print(line)
