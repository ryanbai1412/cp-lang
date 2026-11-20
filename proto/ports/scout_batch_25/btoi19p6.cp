// Baltic OI '19 P6 - Olympiads (DMOJ: btoi19p6)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// N contestants, K events (K <= 6), score[i][j] for contestant i in event j.
// A team is K contestants; its score is the sum over events of the best
// member. Print the score of the C-th best team (C <= 2000).
//
// Fracturing search (Lawler). A subproblem is "these members are forced, these
// contestants are banned"; its best team is greedy: for every free slot j take
// the best allowed contestant for event j. Popping a team whose free slots are
// f..K splits the rest into K - f disjoint subproblems: keep slots < i, ban
// the member in slot i.
use std.heap.{MaxHeap}

let (N, K, C): (1..=500, 1..=6, 1..=2000) = read()
domain P = 0..N
domain Ev = 0..K
let score: [P, Ev]Int = read()

struct Sub(value: Int, team: List<P>, forced: Int, banned: Set<P>)

fn best(forced: List<P>, banned: Set<P>) -> ?Sub:
    var team = forced.copy()
    for j in Ev.from(forced.len):
        let pick = argmax(p for p in P if p not in banned and p not in team, by: p => score[p, j])?
        team.push(pick)
    let value = sum(max(score[p, j] for p in team) for j in Ev)
    Sub(value, team, forced.len, banned)

var heap = MaxHeap<Sub>(by: .value)
heap.push(best([], Set())!)
for _ in 1..C:
    let top = heap.pop()!
    for i in top.forced..K:
        if let sub = best(top.team[..i], top.banned | {top.team[i]}): heap.push(sub)
print(heap.top!.value)
