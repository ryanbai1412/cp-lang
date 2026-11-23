// Baltic OI '19 P6, Olympiads (buildable today-form)
// original: corpus/submissions/4475247.cpp
// Subproblems live in parallel flat lists (team, banned) because the
// prototype heap only orders tuples of Int.
let N: 1..=500 = read()
let K: 1..=6 = read()
let C: 1..=2000 = read()
domain P = 0..N
domain Ev = 0..K
let score: [P, Ev]Int = read()

var teams: List<Int> = []  // K members per subproblem
var bans: List<Bool> = []  // N flags per subproblem
var forced: List<Int> = []
var count = 0
var heap = MinHeap<(Int, Int)>()

// Builds the best team keeping `keep` members of subproblem `src` (or none
// when src < 0), with src's bans plus `extra`. Pushes it if a team exists.
fn spawn(src: Int, keep: Int, extra: Int):
    let base = count * N
    for p in 0..N:
        var b = p == extra
        if src >= 0 and bans[src * N + p]:
            b = true
        bans.push(b)
    var team: List<Int> = []
    for i in 0..keep:
        team.push(teams[src * K + i])
    var ok = true
    for j in keep..K:
        var pick = -1
        for p in 0..N:
            var free = not bans[base + p]
            for q in team:
                if q == p:
                    free = false
            if free and (pick < 0 or score[P(p), Ev(j)] > score[P(pick), Ev(j)]):
                pick = p
        if pick < 0:
            ok = false
        else:
            team.push(pick)
    if not ok:
        // keep the lists aligned: drop the bans just written
        for p in 0..N:
            bans.pop()
        return
    var value = 0
    for j in 0..K:
        var m = 0
        for q in team:
            m max= score[P(q), Ev(j)]
        value += m
    for q in team:
        teams.push(q)
    forced.push(keep)
    heap.push((-value, count))
    count += 1

spawn(-1, 0, -1)
var answer = 0
for round in 0..C:
    let top = heap.pop()!
    answer = -top.0
    let id = top.1
    if round + 1 < C:
        // child i keeps slots < i and bans the member in slot i
        for i in forced[id]..K:
            let out = teams[id * K + i]
            spawn(id, i, out)
print(answer)
