// Source: Baltic OI 2019 P6 Olympiads, https://dmoj.ca/problem/btoi19p6
// Problem: n <= 500 contestants each score 0..=1e6 in k <= 6 events. A team is k contestants
//   and scores the sum over events of its best member's score. Print the score of the c-th best
//   team (c <= 2000, teams differ as sets).
// Solution: Lawler-style partitioning. The best team under a ban set with a forced prefix of
//   slots takes, for each free slot j, the best unbanned unused contestant in event j. A max-heap
//   holds subproblems; popping one spawns, per free slot i, a child that keeps slots < i and
//   bans slot i's member. The c-th pop is the answer. O(c k^3 n) time and O(c k n) memory.
// original: corpus/submissions/4475247.cpp
// Subproblems live in parallel flat lists (team, banned) because the
// prototype heap only orders tuples of Int.
let n: 1..=500 = read()
let k: 1..=6 = read()
let c: 1..=2000 = read()
domain P = 0..n
domain Ev = 0..k
let score: [P, Ev]Int = read()

var teams: List<Int> = []  // K members per subproblem
var bans: List<Bool> = []  // N flags per subproblem
var forced: List<Int> = []
var count = 0
var heap = MinHeap<(Int, Int)>()

// Builds the best team keeping `keep` members of subproblem `src` (or none
// when src < 0), with src's bans plus `extra`. Pushes it if a team exists.
fn spawn(src: Int, keep: Int, extra: Int):
    let base = count * n
    for p in 0..n:
        var b = p == extra
        if src >= 0 and bans[src * n + p]:
            b = true
        bans.push(b)
    var team: List<Int> = []
    for i in 0..keep:
        team.push(teams[src * k + i])
    var ok = true
    for j in keep..k:
        var pick = -1
        for p in 0..n:
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
        for p in 0..n:
            bans.pop()
        return
    var value = 0
    for j in 0..k:
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
for round in 0..c:
    let top = heap.pop()!
    answer = -top.0
    let id = top.1
    if round + 1 < c:
        // child i keeps slots < i and bans the member in slot i
        for i in forced[id]..k:
            let out = teams[id * k + i]
            spawn(id, i, out)
print(answer)
