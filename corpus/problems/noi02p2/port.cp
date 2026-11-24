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

fn says_star(asker: Int, about: Int) -> 0..=1 = ask("Ask", asker + 1, about + 1, 0)

// each child about its right neighbour, then its left: the referee's first answer is true
var right: List<Int> = []
var left: List<Int> = []
for p in 0..n:
    right.push(says_star(p, (p + 1) % n))
    left.push(says_star(p, (p + n - 1) % n))

// both answers on an edge agree, except where the referee spoke first
var wild = 0
for p in 0..n:
    if right[p] != left[(p + 1) % n]:
        wild = p
        break

// 0 Star, 1 Moon, 2 referee: walking away from him, each answer fixes the next child
var role = [0..n] _ => 0
role[wild] = 2
var p = wild - 1
while p >= 0:
    role[p] = Int(right[p] != Int(role[p + 1] == 0))
    p -= 1
for q in wild + 1..n:
    role[q] = Int(left[q] != Int(role[q - 1] == 0))
for i in 0..n: print("Answer", role[i])
