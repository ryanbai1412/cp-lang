// Source: IOI 2020 Day 2 P3 (Stations), https://dmoj.ca/problem/ioi20p6
// Problem: a tree of n <= 1000 stations. label gives them distinct labels in 0..k (k >= n - 1;
//   full score needs k = n - 1); find_next_station then gets only the label s of the current
//   station, the target label t and the sorted labels c of s's neighbours, and must return the
//   neighbour on the path to t.
// Solution: DFS numbering, a station at even depth as the walk enters it and one at odd depth
//   as it leaves, so labels 0..n-1 are a subtree interval with the station's own label at one
//   end. If s is below all its neighbours, s is entered-numbered: child i owns (c[i-1], c[i]],
//   starting just above s, and the parent is the last label; otherwise mirror that from the top.
//   O(n) labelling, O(deg) per hop.
// original: proto/ports/scout_batch_9/harness/ioi20p6.cpp
// Here: label the tree, then for each query print the station a packet
// at s bound for t is sent to.
let n: 2..=1000 = read()
domain Station = 0..n
domain Link = 1..n

fn random_tree() -> [Link](u: Station, v: Station):
    [Link] l => (Station(rand(0, l.label - 1)), Station(l.label))

let links: [Link](u: Station, v: Station) = read(gen: random_tree)
let net = Graph.undirected(links)
let tree = Rooted(net, Station.first!)

// Walk the tree and number the stations: one at even depth as the walk
// enters it, one at odd depth as the walk leaves it. Then the labels inside
// a subtree are consecutive, and a station's own label is at one end of them.
var label: [Station]Int = 0
var station: [Station]Station = Station.first!  // station[label[s].idx] == s
var used = 0
fn number(s: Station):
    label[s] = used
    station[Station(used)] = s
    used += 1
fn walk(s: Station):
    if tree.depth(s) % 2 == 0: number(s)
    for c in tree.children(s): walk(c)
    if tree.depth(s) % 2 == 1: number(s)
walk(Station.first!)

// near: the labels of s's neighbours, ascending.
fn next_hop(s: Int, t: Int, near: List<Int>) -> Int:
    if s < near[0]:
        // s was numbered on the way in. Each child closes its own stretch
        // of labels after s, and the parent, if there is one, comes last.
        let kids = if s == 0: near.len else: near.len - 1
        for i in 0..kids:
            if s < t and t <= near[i]: return near[i]
        near.last!
    else:
        // The mirror image: the parent first, each child opening a stretch.
        for i in (1..near.len).rev:
            if near[i] <= t and t < s: return near[i]
        near[0]

let q: 1..=1e5 = read()
for _ in 0..q:
    let s: Station = read()
    let t: Station = read()
    assert t != s
    let near = sorted([label[x.to] for x in net[s]])
    print(station[Station(next_hop(label[s], label[t], near))])
