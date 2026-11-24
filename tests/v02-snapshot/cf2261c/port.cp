// Source: Codeforces 2261C (I Know About a Million Things),
//   https://codeforces.com/contest/2261/problem/C
// Problem: n <= 1e6 integers in 1..=1e6 (sum n <= 1e6, multitest). Moggity and Larpity alternate,
//   Moggity first, removing a non-empty set that is one element or has an even sum; whoever
//   cannot move loses. Print the winner.
// Solution: only parities matter. With an even number of odds (or n = 1) Moggity removes it all.
//   Otherwise the lost positions are exactly (evens, odds) = (1, 1) and (0, 3), and from any other
//   Moggity reaches one by an even-sum removal (keep one even and one odd, or three odds). So
//   Moggity loses iff evens + odds / 2 == 1: count the odds, O(n).

@!multitest
let n: 1..=1e6 = read()
let a: [0..n]1..=1e6 = read()
let odd = a.count(x => x % 2 == 1)
let even = n - odd
let win = odd % 2 == 0 or n == 1 or even + odd / 2 != 1
print(if win: "Moggity" else: "Larpity")
