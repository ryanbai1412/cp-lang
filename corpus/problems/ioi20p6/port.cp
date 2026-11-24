// Source: IOI 2020 Day 2 P3 (Stations), https://dmoj.ca/problem/ioi20p6
// limits: 1000 ms, 512 MB
// Problem: a tree of n <= 1000 stations. label gives them distinct labels in 0..k (k >= n - 1;
//   full score needs k = n - 1); find_next_station then gets only the label s of the current
//   station, the target label t and the sorted labels c of s's neighbours, and must return the
//   neighbour on the path to t.
// Solution: DFS numbering, a station at even depth as the walk enters it and one at odd depth
//   as it leaves, so labels 0..n-1 are a subtree interval with the station's own label at one
//   end. If s is below all its neighbours, s is entered-numbered: child i owns (c[i-1], c[i]],
//   starting just above s, and the parent is the last label; otherwise mirror that from the top.
//   O(n) labelling, O(deg) per hop.
// original: corpus/submissions/4725602.cpp (grader functions label / find_next_station)
// stdin here: n, the n - 1 links, q, then q queries `s t`; stdout: the station a packet at s
//   bound for t is sent to, per query.
// Bounds: n: 2..=1000; q: 1..=100_000; s, t: 0..n
let n: 2..=1000 = read()
domain Station = 0..n
domain Link = 1..n
domain Label = 0..n

fn random_tree() -> [Link](u: Station, v: Station):
    [Link] l => (Station(rand(0, l.label - 1)), Station(l.label))

let links: [Link](u: Station, v: Station) = read(gen: random_tree)
let net = Graph.undirected(links)
let tree = Rooted(net, Station.first!)

// number the stations as the walk goes: one at even depth as it enters, one at odd depth as
// it leaves. A subtree's labels are then consecutive, with its root's label at one end
var order: List<Station> = []
fn walk(s: Station):
    if tree.depth(s) % 2 == 0: order.push(s)
    for c in tree.children(s): walk(c)
    if tree.depth(s) % 2 == 1: order.push(s)
walk(tree.root)
let station: [Label]Station = [Label] l => order[Int(l)]
let label = station.inverse()

// near: the labels of s's neighbours, ascending
fn next_hop(s: Label, t: Label, near: List<Label>) -> Label:
    if s < near[0]:
        // s was numbered on the way in: each child closes its stretch of labels above s, and
        // the parent, if any, is last
        let kids = if s == Label.first!: near.len else: near.len - 1
        near.take(kids).find(c => s < t <= c) ?? near.last!
    else:
        // the mirror image: the parent first, each child opening a stretch below s
        near.drop(1).rev().find(c => c <= t < s) ?? near[0]

let q: 1..=100_000 = read()
for _ in 0..q:
    let (s, t): (Station, Station) = read()
    assert t != s
    print(station[next_hop(label[s], label[t], net[s].map(e => label[e.to]).sorted())])
