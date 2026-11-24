// Source: IOI 2012 P6 (Jousting Tournament), https://dmoj.ca/problem/ioi12p6
// limits: 1000 ms, 256 MB
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
// Bounds: n: 1..=1e5; c, r, rank: 0..n
// original: corpus/submissions/4848321.cpp (the grader function GetBestPosition)

let n: 1..=1e5 = read()
let c: 0..n = read()
let r: 0..n = read()  // the late knight's rank
domain Pos = 0..n
domain Gap = 0..n - 1   // gap g is between positions g and g + 1
domain Round = 0..c
domain Node = 0..n + c  // the leaves, then the rounds
let rank: [Gap]0..n = read()  // the fixed knight who fills gap g
fn leaf(p: Pos) -> Node = Node(p.ord)
fn node(round: Round) -> Node = Node(n + round.ord)
var kids: [Node]List<Node> = []

// alive[p] = 1 while the group whose leftmost starting position is p is
// still in; `head[p]` is the tree node that group has become
var alive = Fenwick<Pos, Int>([Pos] _ => 1)
var head = [Pos] p => leaf(p)
for round in Round:
    let (s, e): (0..n, 0..n) = read()
    // the k-th survivor is a Fenwick descent (an order-statistics Set would name it)
    let first = alive.first_ge(s + 1)!.left!
    kids[node(round)].push(head[first])
    for _ in s..e:
        let gone = alive.first_ge(s + 2)!.left!
        kids[node(round)].push(head[gone])
        alive.add(gone, -1)
    head[first] = node(round)

// top[v]: the strongest fixed knight inside v's span of gaps; inner[v]: the
// strongest strictly between v's first and last position. -1 is below every
// rank, the identity of `max`.
var top: [Node]-1..n = -1
var inner: [Node]-1..n = -1
for g in Gap: top[leaf(Pos(g.ord))] = rank[g]
for round in Round:
    let v = node(round)
    for i in 0..kids[v].len:
        let k = kids[v][i]
        top[v] max= top[k]
        inner[v] max= if i + 1 < kids[v].len: top[k] else: inner[k]

// wins[v]: rounds the late knight wins on the way to v from below
var wins: [Node]Int = 0
for round in Round.rev:
    let v = node(round)
    for k in kids[v]: wins[k] = wins[v] + Int(r > inner[v])
print(Pos.argmax(by: p => wins[leaf(p)])!)
