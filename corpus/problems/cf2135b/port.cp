// Source: Codeforces 2135B (For the Champion), https://codeforces.com/contest/2135/problem/B
// limits: 2000 ms, 250 MB
// Problem: interactive, multitest. n <= 100 known anchors, |x|, |y| <= 1e9; the robot starts at
//   an unknown (X, Y), |X|, |Y| <= 1e9. Each of at most 10 moves goes U/D/L/R by k <= 1e9 and
//   answers the least Manhattan distance to an anchor. Print "! X Y".
// Solution: after U, U, R, R by 1e9 the robot is north-east of every anchor, so the answer is
//   X + Y + 4e9 - max(x + y); after four L moves it is north-west of all of them, so the
//   answer is -X + Y + 4e9 - max(-x + y). Solve the two sums for X and Y: 8 moves, O(n).
// Bounds: n: 1..=100

let wire = Wire()

fn ask(d: Char, dist: Int) -> Int = wire.ask("? {d} {dist}")

@!multitest
let n: 1..=100 = read()
let anchors: [0..n](x: -10 ** 9..=10 ** 9, y: -10 ** 9..=10 ** 9) = read()
let step = 1_000_000_000

for d in "UUR": ask(d, step)
let d_ur = ask('R', step)
for _ in 0..3: ask('L', step)
let d_ul = ask('L', step)

let ur_pos = max(p.x + p.y for p in anchors)! + d_ur - 4 * step
let ul_pos = max(-p.x + p.y for p in anchors)! + d_ul - 4 * step
print("!", (ur_pos - ul_pos) / 2, (ur_pos + ul_pos) / 2)
