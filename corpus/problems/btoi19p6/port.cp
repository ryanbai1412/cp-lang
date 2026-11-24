// Source: Baltic OI 2019 P6 (Olympiads), https://dmoj.ca/problem/btoi19p6
// Problem: n <= 500 contestants score 0..=1e6 in each of k <= 6 events. A team is k distinct
//   contestants scoring the sum over events of its best member there. Print the c-th best
//   team score (c <= 2000, teams differ as sets; at least c teams exist).
// Solution: Lawler's k-best partitioning. Slot j of a team is its best member for event j; the
//   best team with slots before `fixed` kept and a ban set greedily fills every other slot. A
//   max-heap of subproblems pops them best first; a popped team splits into one child per free
//   slot i that keeps slots before i and bans slot i's member. The c-th pop is the answer.
//   O(c k^2 n) time, O(c k n) memory.
// original: corpus/submissions/4475247.cpp
let n: 1..=500 = read()
let k: 1..=6 = read()
let c: 1..=2000 = read()
domain P = 0..n
domain Ev = 0..k
let score: [P, Ev]0..=1_000_000 = read()

// The best team keeping `team`'s members in their slots and avoiding `banned`.
// `banned` is a List indexed by `p.ord`: a `[P]Bool` field stops `Heap(by:)` ordering `Sub`
// (research/frictions/btoi19p6.md).
struct Sub(value: Int, team: List<P>, fixed: Int, banned: List<Bool>)

fn best(keep: List<P>, banned: List<Bool>) -> ?Sub:
    var team = keep.clone()
    for e in Ev:
        if e.ord >= keep.len:
            team.push(P.all.filter(p => not banned[p.ord] and not team.contains(p)).max(by: p => score[p, e])?)
    let value = sum(team.map(p => score[p, e]).max()! for e in Ev)
    Some(Sub(value, team, keep.len, banned))

var subs = Heap<Sub>.new(by: s => desc(s.value))
if let s = best([], List.filled(n, false)):
    subs.push(s)
repeat c - 1:
    let top = subs.pop()!
    for i in top.fixed..k:
        var banned = top.banned.clone()
        banned[top.team[i].ord] = true
        if let s = best(top.team[0..i].collect(), banned):
            subs.push(s)
print(subs.peek!.value)
