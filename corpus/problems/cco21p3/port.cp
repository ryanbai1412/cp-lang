// Source: CCO 2021 P3 (Through Another Maze Darkly), https://dmoj.ca/problem/cco21p3
// Problem: a tree of n <= 8e5 rooms, each listing its neighbours clockwise, with a pointer at the
//   first. A step turns the current room's pointer clockwise to the next corridor and walks it.
//   For each of q <= 8e5 independent queries k <= 1e15, print the room reached after k steps
//   starting from room 1.
// Solution: the walk is a sequence of passes from room 1 back to room 1, each a sub-tour of one
//   fixed Euler tour of the doubled tree. Entering a room from its parent, the pointer next takes
//   the corridors after the parent's, which join the tour one pass later than the ones before it;
//   so one DFS gives every tour step the pass it joins at, and after n passes every step is in.
//   Prefix sums of pass sizes find a query's pass (past the last, the full tour repeats); offline
//   by pass, a Fenwick over tour steps holds the joined steps and `kth` finds the k-th one.
//   O((n + q) log n).
let (n, q): (2..=8e5, 1..=8e5) = read()
domain Room = 1..=n
// The pointer starts at the first corridor, so the first step from a room takes the second.
let around: [Room]List<Room> = [Room] _ => read<List<Room>>().rotated_left(1)
domain Pass = 0..=n

var tour: List<(pass: Pass, to: Room)> = []
fn walk(u: Room, from: ?Room, pass: Pass):
    let doors = around[u]
    let order = if let p = from:
        let back = (0..doors.len).find(i => doors[i] == p)!
        [(c, pass.next!) for c in doors[back + 1..]] + [(c, pass) for c in doors[..back]]
    else:
        [(c, pass) for c in doors]
    for (v, t) in order:
        tour.push((t, v))
        walk(v, Some(u), t)
        tour.push((t, u))
walk(Room.first!, None, Pass.first!)

domain Step = 0..tour.len
var joins: [Pass]List<Step> = [Pass] _ => []
for s in Step: joins[tour[s.ord].pass].push(s)
let active = joins.map(j => j.len).scan(+)
let before = ([Pass] p => active.prefix(p.r)).scan(+)

var asked: [Pass]List<(k: Int, i: Int)> = [Pass] _ => []
for i in 0..q:
    let k: 1..=1e15 = read()
    if let c = before.first_ge(k):
        let p = c.l!
        asked[p].push((k - before.prefix(p.l), i))
    else:
        asked[Pass.last!].push(((k - before.all) % Step.count, i))

var joined = Fenwick<Step, Int>.new()
var room = List.filled(q, Room.first!)
for p in Pass:
    for s in joins[p]: joined.add(s, 1)
    for (k, i) in asked[p] if k > 0: room[i] = tour[joined.kth(k)!.ord].to
for r in room: print(r)
