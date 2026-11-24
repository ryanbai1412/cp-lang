// Source: Codeforces 2150A (Incremental Path), https://codeforces.com/contest/2150/problem/A
// limits: 1000 ms, 256 MB
// Problem: cells 1..=1e9, m <= 1e5 of them black; a string s of n <= 1e5 commands, A (go to x + 1)
//   or B (go to the next white cell). For i = 1..=n a person starts at cell 1, runs the first i
//   commands and paints the final cell black. Print the black cells at the end (multitest).
// Solution: every painted cell is at the current front, so person i + 1 replays person i's walk
//   except that a B now skips the cell just painted. Keep cur, the position after s[0..i]: step
//   it (skipping black cells on a B), paint it, and on a B skip again to get the next person's
//   position. cur only moves right, so the skips total O(n + m) set operations.
// Bounds: n, m: 1..=100_000; a: 1..=10 ** 9

@!multitest
let (n, m): (1..=100_000, 1..=100_000) = read()
let s: [0..n]Char = read()
let a: [0..m]1..=10 ** 9 = read()
var black = Set(a)

var cur = 1
fn skip_black():
    while cur in black: cur += 1

for ch in s:
    cur += 1
    if ch == 'B': skip_black()
    black.insert(cur)
    if ch == 'B': skip_black()

print(black.len)
print(black)
