// Source: IOI 2012 P6 (Jousting Tournament), https://dmoj.ca/problem/ioi12p6
// Problem: n knights in a line, n - 1 of them with known distinct ranks in order; the late
//   knight of rank r takes any position. Each of c rounds (s, e) keeps only the strongest of
//   the knights now at positions s..=e. Print the position (smallest on ties) from which the
//   late knight wins the most rounds. n <= 1e5.
// Solution: the rounds form a tree over the n starting positions: a Fenwick tree of alive
//   slots finds the k-th survivor, so each round becomes a node whose children are the
//   survivors it merges. A late knight under round v (leaves s..e) meets exactly the fixed
//   knights of gaps s..e-1, so he wins v iff r beats their maximum; carry win counts from
//   the root down to the leaves. O((n + c) log n).
// input: n c r, the n - 1 ranks in order, then c lines `s e` (0-based)
// original: corpus/submissions/4848321.cpp (the grader function GetBestPosition)

let (n, c, r): (Int, Int, Int) = read()
domain Pos = 0..n
domain Gap = 0..n - 1        // gap g is between positions g and g + 1
domain Node = 0..n + c       // the leaves, then the rounds
let rank: [Gap]Int = read()  // the fixed knight who fills gap g
var kids: [Node]List<Node> = [Node] _ => []

// alive[p] = 1 while the group whose leftmost starting position is p is
// still in; `head[p]` is the tree node that group has become
var alive = Fenwick<Pos, Int>([Pos] _ => 1)
var head: [Pos]Node = [Pos] p => Node(p.to_idx)
for round in 0..c:
    let (s, e): (Int, Int) = read()
    let node = Node(n + round)
    // WISH C7: an order-statistics `Set` (`line.nth(s)`, `line.remove_range(s..=e)`)
    // is what the original uses; here the k-th survivor is a Fenwick descent.
    let first = alive.first_ge(s + 1)!.left!
    kids[node].push(head[first])
    for _ in s..e:
        let gone = alive.first_ge(s + 2)!.left!
        kids[node].push(head[gone])
        alive.add(gone, -1)
    head[first] = node

// top[v]: the strongest fixed knight inside v's span of gaps; inner[v]:
// the strongest strictly between v's first and last position
var top: [Node]Int = -1
var inner: [Node]Int = -1
var wins: [Node]Int = 0
for v in Node:
    if v.to_idx < n:
        if v.to_idx < n - 1: top[v] = rank[Gap(v.to_idx)]
        continue
    for i in 0..kids[v].len:
        let k = kids[v][i]
        top[v] max= top[k]
        inner[v] max= if i + 1 < kids[v].len: top[k] else: inner[k]
var best = (wins: -1, at: 0)
for v in Node.rev:
    if v.to_idx < n:
        if wins[v] >= best.wins: best = (wins: wins[v], at: v.to_idx)
        continue
    for k in kids[v]: wins[k] = wins[v] + Int(r > inner[v])
print(best.at)
