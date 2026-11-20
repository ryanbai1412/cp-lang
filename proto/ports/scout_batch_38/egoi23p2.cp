// EGOI '23 P2 (Padel Prize Pursuit): offline time-tree rollback DFS tracking
// the player with maximum total holding duration (smallest index tie-break).
// original: corpus/submissions/5678050.cpp
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Player = 0..n
domain Event = 0..m

var winner: [Event]Player = Player.first
var children: [Event]List<Event> = []
var rep: [Player]?Event = none
for e in Event:
    let (u, v): (Player, Player) = read()
    winner[e] = u
    if let pu = rep[u]: children[e].push(pu)
    if let pv = rep[v]: children[e].push(pv)
    rep[u] = e
    rep[v] = none

// FRICTION 38.6: Set<(Int, Int)> forces negating `p.to_idx` for smallest-index
// tie-break and manual remove-modify-add on every rollback update.
var cnt: [Player]Int = 0
var board = Set<(Int, Int)>()
for p in Player: board.add((0, -p.to_idx))

fn add_time(p: Player, delta: Int):
    board.remove((cnt[p], -p.to_idx))
    cnt[p] += delta
    board.add((cnt[p], -p.to_idx))

var ans: [Player]Int = 0
fn dfs(u: Event, next_time: Int):
    let dt = next_time - u.to_idx
    add_time(winner[u], dt)
    let (_, neg_best) = board.max!
    ans[Player(-neg_best)] += 1
    for v in children[u]: dfs(v, u.to_idx)
    add_time(winner[u], -dt)

for p in Player:
    if let root = rep[p]: dfs(root, m)

print(ans)
