// Source: COCI 2021/2022 Contest 1 #2 (Kamencici), https://dmoj.ca/problem/coci21c1p2
// Problem: n <= 350 pebbles in a row, red (C) or blue (P), at least 2k - 1 red. Two players
//   alternately take a pebble from either end; whoever first holds k red loses. Print DA if the
//   first player can force a win, else NE.
// Solution: the pebbles left are a range l..r, so reds taken so far come from prefix sums and the
//   state is (l, r, reds of the player to move); the other's reds follow. Memoized win/lose
//   recursion over taking either end: O(n^2 k) states, O(1) each.
let (n, k): (Int, Int) = read()
let s: Str = read()

var pref = List.filled(n + 1, 0)
for i in 0..n:
    let add = Int(s[i] == 'C')
    pref[i + 1] = pref[i] + add
let tot = pref[n]

let stride_r = n + 1
let stride_k = k + 1
var memo = List.filled((n + 1) * stride_r * stride_k, -1)

fn cell(l: Int, r: Int, my_red: Int) -> Int = (l * stride_r + r) * stride_k + my_red

fn win(l: Int, r: Int, my_red: Int) -> Bool:
    let taken = tot - (pref[r + 1] - pref[l])
    let opp_red = taken - my_red
    if opp_red >= k: return true
    if my_red >= k: return false
    if l > r: return false
    let c = cell(l, r, my_red)
    if memo[c] != -1: return memo[c] == 1
    let res = (not win(l + 1, r, opp_red)) or (not win(l, r - 1, opp_red))
    memo[c] = Int(res)
    res

print(if win(0, n - 1, 0): "DA" else: "NE")
