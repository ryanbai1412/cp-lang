// Source: Yet Another Contest 1 P3 (Fluke), https://dmoj.ca/problem/yac1p3
// limits: 4000 ms, 256 MB
// Problem: interactive; read N, T (both <= 100), then win T games on an empty N x N grid:
//   players take turns marking an unmarked cell not edge-adjacent to a marked one, and the
//   last to move wins. Each game print 1 or 2 to choose who moves first, then exchange moves
//   as r c (the judge sends 0 0 when it is stuck, -1 -1 after an invalid move).
// Solution: point symmetry through the centre. With N odd move first and take the
//   centre, with N even move second; then answer (x, y) with (N + 1 - x, N + 1 - y). The
//   position stays symmetric and a cell is never adjacent to its own mirror, so the mirror
//   is always legal and the judge runs out of moves first. O(1) per move.
// original: corpus/submissions/4380584.cpp

let (n, t) = read<(1..=100, 1..=100)>()

fn their_move() -> ?(1..=n, 1..=n):
    let (x, y) = read<(0..=n, 0..=n)>()
    if x == 0: None else: (x, y)

for _ in 0..t:
    if n % 2 == 1:
        print(1)
        print((n + 1) / 2, (n + 1) / 2)
    else:
        print(2)
    while let (x, y) = their_move(): print(n + 1 - x, n + 1 - y)
