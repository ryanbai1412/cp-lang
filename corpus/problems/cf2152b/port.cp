// Source: Codeforces 2152B (Catching the Krug), https://codeforces.com/contest/2152/problem/B
// Problem: on the (n+1) x (n+1) grid of cells 0..=n, n <= 1e9, the Krug (moves to a side
//   neighbour or stays) starts at (rK, cK) and Doran (king moves or stays) at (rD, cD); the Krug
//   moves first. Print the number of Doran's turns until he catches her under optimal play,
//   or -1 if she survives forever. Multitest, t <= 1e4.
// Solution: king distance splits by axis, so the Krug can only gain by running straight away
//   from Doran on an axis where she is not level with him: to row 0 when rK < rD (Doran needs
//   rD turns to get there), to row n when rK > rD (n - rD turns), 0 when level; columns
//   likewise. The answer is the larger of the two axes, O(1); -1 is never printed.

@!multitest
let n: 1..=10 ** 9 = read()
let (rk, ck, rd, cd): (0..=10 ** 9, 0..=10 ** 9, 0..=10 ** 9, 0..=10 ** 9) = read()
let dx = if rd > rk: rd elif rd < rk: n - rd else: 0
let dy = if cd > ck: cd elif cd < ck: n - cd else: 0
print(max(dx, dy))
