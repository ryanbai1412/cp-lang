// Source: NOI 2002 P2 (Naughty Kid), https://dmoj.ca/problem/noi02p2
// limits: 5000 ms, 512 MB
// Problem: interactive. N + M + 1 <= 501 children: N Star (always truthful), M Moon (always lie)
//   and a referee (true on his odd-numbered answers, false on even). Ask(a, b, T) asks a whether
//   b is in team T (never about himself, each pair once). Name the referee and everyone's team,
//   asking each child fewer than three questions for full marks.
// Solution: seat everyone in a ring and ask each child "is your right neighbour Star?", then "is
//   your left one?". From a team member either answer means "same team", so both answers on an
//   edge agree, except the referee's first (true) one about his right neighbour: the first
//   disagreeing edge finds him. Walking away from him, each answer fixes the next child's team.
//   Two questions per child, O(N + M).
// original: corpus/submissions/4793917.cpp
let (stars, moons): (0..=500, 0..=500) = ask("GetNM")
let n = stars + moons + 1
domain Child = 0..n
enum Role:
    Star
    Moon
    Referee

fn says_star(asker: Child, about: Child) -> Bool:
    let yes: 0..=1 = ask("Ask", Int(asker) + 1, Int(about) + 1, 0)
    yes == 1

// each child about its right neighbour, then its left: the referee's first answer is true
let right = [Child] p => says_star(p, Child.wrap(Int(p) + 1))
let left = [Child] p => says_star(p, Child.wrap(Int(p) - 1))

// both answers on an edge agree, except where the referee spoke first
let referee = Child.find(p => right[p] != left[Child.wrap(Int(p) + 1)])!

// walking away from him, each answer fixes the next child: a Star's answer is the truth
var role: [Child]Role = Referee
for p in (..referee).rev:
    role[p] = if right[p] == (role[p.next!] == Star): Star else: Moon
for p in Child.after(referee):
    role[p] = if left[p] == (role[p.prev!] == Star): Star else: Moon
for c in Child: print("Answer", role[c].ord)
