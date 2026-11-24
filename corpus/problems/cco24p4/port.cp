// Source: CCO 2024 Day 2 P1 (Infiltration), https://dmoj.ca/problem/cco24p4
// limits: 1000 ms, 1024 MB
// Problem: output-only strategy on a tree of N = 100 rooms (N - 1 edges given). Two agents
//   start in different unknown rooms u, v; Ondrej may step to a neighbour on odd minutes,
//   Edward on even ones. Print T <= 1440, then each agent's room at times 1..T for every
//   start room, so they always meet; f(u, v) = first common time. Partial score: the
//   smaller max f(u, v) / dist(u, v), the better (full marks at <= 15).
// Solution: both agents only ever climb toward a centre root (depth <= 50), on one fixed
//   schedule of phases where one climbs while the other waits: Edward 3, Ondrej 11, Edward
//   22, Ondrej 35, Edward 55, Ondrej 30 steps (constants tuned in the C++). Each has over 50 climbs, so they meet at the root at worst; the phases make the
//   lower one catch the other early. An idle minute is reused when the turn switches.
//   T = 308. Measured S = 13 (full marks need S <= 15) on random, path, star, caterpillar,
//   broom and binary trees; not proven for every tree. O(N^2) to find the root, O(N T) output.
// original: corpus/submissions/6435484.cpp
// Bounds: n: 100..=100; u, v: 0..n
let n: 100..=100 = read()
domain Room = 0..n
let around = Graph.undirected(read<[n - 1](Room, Room)>())

fn shallow(r: Room) -> Bool:
    let t = Rooted(around, r)
    Room.all(v => t.depth(v) <= 50)
let tree = Rooted(around, Room.rev.find(shallow)!)

domain Agent = 0..2
let phases: List<(Agent, Int)> = [(1, 3), (0, 11), (1, 22), (0, 35), (1, 55), (0, 30)]
var climbs: List<Bool> = []
for (who, steps) in phases:
    for _ in 0..steps:
        if who.ord == 1 and climbs.len > 0 and not climbs.last!: climbs.last! = true
        else: climbs.extend([who.ord == 0, who.ord == 1])

print(climbs.len)
for who in Agent:
    for start in Room:
        var at = start
        var row: List<Room> = []
        for (minute, climb) in climbs.items:
            if climb and minute % 2 == who.ord: at = tree.parent(at) ?? at
            row.push(at)
        print(row)
