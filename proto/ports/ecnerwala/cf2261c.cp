// Codeforces 2261 C: the first player wins at once when the count of odd
// numbers is even or there is one number; otherwise they lose only when the
// evens plus half the odds leave exactly one forced position.
@!multitest
let n: 1..=2e5 = read()
let a: [0..n]Int = read()
let odd = count(x % 2 == 1 for x in a)
let win = odd % 2 == 0 or n == 1 or n - odd + odd / 2 != 1
print(if win: "Moggity" else: "Larpity")
