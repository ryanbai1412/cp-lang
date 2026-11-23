// Codeforces 2261 C — Odd-Even Parity Game
//
// If `odd` is even or `n == 1`, Moggity wins immediately. Otherwise `odd` is
// odd and `n >= 2`: pairing odd elements gives `odd / 2` pairs alongside
// `even = n - odd` even elements, leaving `even + odd / 2` free moves.
// Moggity loses iff `even + odd / 2 == 1` (the two forced losing states
// `(n, odd) in {(2, 1), (3, 3)}`), and wins in every other case.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=2 ** 30 - 1 = read()
let odd = a.count(x => x % 2 == 1)
let even = n - odd
let win = odd % 2 == 0 or n == 1 or even + odd / 2 != 1
print(if win: "Moggity" else: "Larpity")
