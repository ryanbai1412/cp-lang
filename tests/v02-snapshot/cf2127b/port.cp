// Source: Codeforces 2127B (Hamiiid, Haaamid... Hamid?),
//   https://codeforces.com/contest/2127/problem/B
// Problem: a 1 x n grid (n <= 2e5; multitest) of walls '#' and empty '.', Hamid on empty cell x.
//   Each day Mani walls an empty cell other than Hamid's, then Hamid picks a side: no wall there
//   and he escapes, else he moves onto the nearest wall and destroys it. Hamid minimises the days
//   to escape, Mani maximises; print the days.
// Solution: at 0-based p, the left side costs 1 day if open, else l + 2 for its nearest wall l;
//   the right side 1 or n - r + 1. Hamid takes the cheaper side. On day 1 Mani can wall the empty
//   cell next to Hamid, making that side cost p + 1 (left) or n - p (right) when that is more;
//   the answer is the best of Mani's options. O(n).

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let x: Pos = read()
let s: [0..n]Char = read()

let p = x.ord
let left_escape = if let l = (0..p).rev.find(i => s[i] == '#'): l + 2 else: 1
let right_escape = if let r = (p + 1..n).find(i => s[i] == '#'): n - r + 1 else: 1
var ans = min(left_escape, right_escape)
if left_escape < p + 1: ans max= min(p + 1, right_escape)
if right_escape < n - p: ans max= min(left_escape, n - p)
print(ans)
