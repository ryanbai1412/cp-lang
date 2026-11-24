// Source: NOI 2002 P2 (Naughty Kid), https://dmoj.ca/problem/noi02p2
// Problem: interactive. N + M + 1 <= 501 children: N Star (always truthful), M Moon (always lie)
//   and a referee (true on his odd-numbered answers, false on even). Ask(a, b, T) asks a whether
//   b is in team T (never about himself, each pair once). Name the referee and everyone's team,
//   asking each child fewer than three questions for full marks.
// Solution: seat everyone in a ring and ask each child "is your right neighbour Star?", then "is
//   your left one?". From a team member either answer means "same team", so both answers on an
//   edge agree, except the referee's first (true) one about his right neighbour: the first
//   disagreeing edge finds him. Walking away from him, each answer fixes the next child's team.
//   Two questions per child, O(N + M).
// This port simulates the interactor: stdin has N, M, the hidden roles (0 Moon, 1 Star,
//   2 referee) and the referee's answer about each child; it prints the recovered roles.
// original: corpus/submissions/4793917.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

fn main_solve():
    let cnt_n: Int = read()
    let cnt_m: Int = read()
    let n = cnt_n + cnt_m + 1
    var kind: List<Int> = []
    var wild_ans: List<Int> = []
    for _ in 0..n: kind.push(ri())
    for _ in 0..n: wild_ans.push(ri())

    // 0 = truth, 1 = liar, 2 = wild
    fn ask_ring(i: Int, j: Int) -> Int:
        if kind[i] == 2: return wild_ans[j]
        if kind[i] == 0: return Int(kind[j] == 0)
        return if kind[j] == 0: 0 else: 1

    var dir_r = [0..n] _ => 0
    var dir_l = [0..n] _ => 0
    for p in 0..n:
        dir_r[p] = ask_ring(p, (p + 1) % n)
        dir_l[p] = ask_ring(p, (p + n - 1) % n)

    var wild = 0
    for p in 0..n:
        if dir_r[p] != dir_l[(p + 1) % n]:
            wild = p
            break

    var ans = [0..n] _ => 0
    ans[wild] = 2
    var p = wild - 1
    while p >= 0:
        let next_is_zero = Int(ans[p + 1] == 0)
        ans[p] = Int(dir_r[p] != next_is_zero)
        p -= 1
    for q in wild + 1..n:
        let prev_is_zero = Int(ans[q - 1] == 0)
        ans[q] = Int(dir_l[q] != prev_is_zero)

    for i in 0..n: print(ans[i])

main_solve()
