// Source: EGOI 2023 Day 1 P2 (Padel Prize Pursuit), https://dmoj.ca/problem/egoi23p2
// limits: 3000 ms, 1024 MB
// Problem: N players, M matches (N, M <= 2e5); on day i, x_i beats y_i, gets a new medal and all
//   of y_i's medals. Each medal finally goes to whoever held it the most nights up to day M (ties:
//   smallest index). Print every player's final medal count.
// Solution: make each match the parent of the last earlier win of each of its two players (whose
//   medals it collects). Medal i then sits with winner[e] for every e from match i up to its root,
//   for parent(e) - e nights (M - e at the root). A DFS from each root adds those nights on the
//   way down and takes them back on the way up; an ordered set of (nights, -player) names the
//   leader at each match. O((N + M) log N).
// Runnable on proto/cpc today.
// original: corpus/submissions/5678050.cpp
// Bounds: n, m: 1..=2e5
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Player = 0..n
domain Event = 0..m

var winner: [Event]Player = Player.first!
var children: [Event]List<Event> = []
var rep: [Player]?Event = None
for e in Event:
    let u: Player = read()
    let v: Player = read()
    winner[e] = u
    if let pu = rep[u]: children[e].push(pu)
    if let pv = rep[v]: children[e].push(pv)
    rep[u] = e
    rep[v] = None

var cnt: [Player]Int = 0
var board = Set<(Int, Int)>()
for p in Player: board.insert((0, -p.idx))

fn add_time(p: Player, delta: Int):
    board.remove((cnt[p], -p.idx))
    cnt[p] += delta
    board.insert((cnt[p], -p.idx))

var ans: [Player]Int = 0
fn dfs(u: Event, next_time: Int) -> ():
    let dt = next_time - u.idx
    let w = winner[u]
    add_time(w, dt)
    let best = board.max()!
    ans[Player(-best.1)] += 1
    for v in children[u]: dfs(v, u.idx)
    add_time(w, -dt)

for p in Player:
    if let root = rep[p]: dfs(root, m)

print(ans)
