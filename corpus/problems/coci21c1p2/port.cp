// Source: COCI 2021/2022 Contest 1 #2 (Kamencici), https://dmoj.ca/problem/coci21c1p2
// limits: 1000 ms, 512 MB
// Problem: n <= 350 pebbles in a row, red (C) or blue (P), at least 2k - 1 red. Two players
//   alternately take a pebble from either end; whoever first holds k red loses. Print DA if the
//   first player can force a win, else NE.
// Solution: the pebbles left lie between two cuts, so the reds taken so far come from prefix
//   sums and the state is (cuts, reds of the player to move); the other's reds follow. A
//   memoized win/lose recursion over taking either end: O(n^2 k) states, O(1) each.
// Bounds: n, k: 1..=350
let n: 1..=350 = read()
let k: 1..=350 = read()
domain Pebble = 0..n
let s: [Pebble]Char = read()
domain Held = 0..=k
let reds = prefix([Pebble] i => Int(s[i] == 'C'))

// The player to move faces the pebbles between lo and hi and holds `mine` reds; the other
// reds taken so far are the other player's.
@memo(array)
fn wins(lo: Pebble.cuts, hi: Pebble.cuts, mine: Held) -> Bool:
    let theirs = reds.all - reds.fold(lo..hi) - mine.ord
    if theirs >= k: return true
    if mine.ord >= k: return false
    let (Some(l), Some(r)) = (lo.r, hi.l) else: return false
    if l > r: return false
    not wins(l.r, hi, Held(theirs)) or not wins(lo, r.l, Held(theirs))

print(if wins(Pebble.cuts.first!, Pebble.cuts.last!, Held(0)): "DA" else: "NE")
