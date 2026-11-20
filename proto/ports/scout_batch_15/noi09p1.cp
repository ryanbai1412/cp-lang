// NOI 2009 Transformation Sequence: given distances d[0..n], each index i
// must map to either (i - d[i]) mod n or (i + d[i]) mod n so the outputs form
// a permutation of 0..n. Print the lexicographically smallest such permutation,
// or "No Answer".
//
// Each position i in `Pos` has two candidate values in `Val`, so positions are
// edges on vertex set `Val`. Any value of degree 1 has a forced partner (leaf
// peeling); remaining unpeeled components must be simple cycles, where the
// smallest-index edge picks `min(u, v)` and forces the rest of its cycle.
// Separate `Pos` and `Val` domains prevent mixing left and right sides; CP
// Lang's Euclidean `%` avoids `(i - d + n) % n`.
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

var ans: [Pos]?Val = none
var used_val: [Val]Bool = false
var impossible = false

fn assign(i: Pos, u: Val):
    if ans[i] != none or used_val[u]:
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
        if ans[a.edge] == none:
            assign(a.edge, u)
            deg[u] -= 1
            deg[a.to] -= 1
            if deg[a.to] == 1: leaves.push(a.to)

// Orient each remaining cycle starting from its smallest position index.
for i in Pos:
    if impossible or ans[i] != none: continue
    let (u, v) = ends[i]
    let first = min(u, v)
    let other = if first == u: v else: u
    assign(i, first)
    var cur = other
    while cur != first and not impossible:
        var moved = false
        for a in adj[cur]:
            if ans[a.edge] == none:
                assign(a.edge, cur)
                cur = a.to
                moved = true
                break
        if not moved:
            impossible = true
            break

for i in Pos:
    if ans[i] == none: impossible = true
for u in Val:
    if not used_val[u]: impossible = true

if impossible:
    print("No Answer")
else:
    var line = ""
    for i in Pos:
        if i.to_idx > 0: line = line + " "
        line += "{ans[i]!.to_idx}"
    print(line)
