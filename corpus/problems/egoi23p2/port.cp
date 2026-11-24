// Source: EGOI 2023 Day 1 P2 (Padel Prize Pursuit), https://dmoj.ca/problem/egoi23p2
// limits: 3000 ms, 1024 MB
// Problem: N players, M matches (N, M <= 2e5); on day i, x_i beats y_i, gets a new medal and all
//   of y_i's medals. Each medal finally goes to whoever held it the most nights up to day M (ties:
//   smallest index). Print every player's final medal count.
// Solution: each match takes the medals of the last earlier win of each of its two players, so
//   the matches form a forest. Medal e sits with the winner of every match from e up to its root,
//   for (the next match up) - e nights (M - e at the root). A walk from each root adds those
//   nights on the way down and takes them back on the way up; an ordered set of
//   (nights, desc(player)) names the leader at each match. O((N + M) log N).
// original: corpus/submissions/5678050.cpp
// Bounds: n, m: 1..=2e5; matches.x, matches.y: 0..n

let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain Player = 0..n
domain Match = 0..m
let matches: [Match](x: Player, y: Player) = read()

var takes: [Match]List<Match> = []  // the last earlier win of each of its two players
var last_win: [Player]?Match = None
for e in Match:
    let (x, y) = matches[e]
    if let w = last_win[x]: takes[e].push(w)
    if let w = last_win[y]: takes[e].push(w)
    last_win[x] = e
    last_win[y] = None

var nights: [Player]Int = 0
var board = Set<(Int, Desc<Player>)>()  // the max is the leader: most nights, then least index
for p in Player: board.insert((0, desc(p)))

fn hold(p: Player, d: Int):
    board.remove((nights[p], desc(p)))
    nights[p] += d
    board.insert((nights[p], desc(p)))

var medals: [Player]Int = 0
fn walk(e: Match, until: Int) -> ():
    let x = matches[e].x
    hold(x, until - e.ord)
    medals[board.max()!.1.x] += 1
    for w in takes[e]: walk(w, e.ord)
    hold(x, e.ord - until)

for p in Player:
    if let root = last_win[p]: walk(root, m)
print(medals)
