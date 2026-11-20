// COCI '21 Contest 1 #2 - Kamencici (DMOJ: coci21c1p2, 10pt, TL 1.0s, ML 512MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Row of N pebbles ('C' = red, 'P' = blue), with at least 2K - 1 red pebbles.
// Anton and Branko alternate removing a pebble from either end. First player to
// collect K red pebbles loses. Output "DA" if Anton wins, "NE" otherwise.
//
// State (l, r, my_red): remaining interval s[l..=r] and current player's red count.
// Opponent's red count is determined from prefix sums: opp_red = taken_red - my_red.
let (n, k): (1..=350, 1..=350) = read()
// WISH MG11: alphabet constraint {'C', 'P'} + existential count >= 2*k - 1 for cpl stress
let s: Str = read()
assert s.len == n and count(c == 'C' for c in s) >= 2 * k - 1

var pref = List.filled(n + 1, 0)
for i in 0..n:
    pref[i + 1] = pref[i] + Int(s[i] == 'C')
let tot = pref[n]

// WISH MG10: @memo over 3D bounded (l, r, my_red) with dense flat table + packed Bool/U8
@memo
fn win(l: Int, r: Int, my_red: Int) -> Bool:
    let taken = tot - (pref[r + 1] - pref[l])
    let opp_red = taken - my_red
    if opp_red >= k: return true
    if my_red >= k: return false
    if l > r: return false
    not win(l + 1, r, opp_red) or not win(l, r - 1, opp_red)

print(if win(0, n - 1, 0): "DA" else: "NE")
