// COCI '21 Contest 1 #2 - Kamencici (runnable on proto/cp.py lib-run today)
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
