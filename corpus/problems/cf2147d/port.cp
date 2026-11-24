// Source: Codeforces 2147D (Game on Array), https://codeforces.com/contest/2147/problem/D
// Problem: n positive integers a_i <= 1e9 (multitest, sum of n <= 2e5). Alice and Bob
//   alternate, Alice first; a move picks a value x > 0 in the array, scores its number of
//   copies and decrements all of them. Print both scores under optimal play.
// Solution: an element x is worth x points over the game. Its even part 2 * floor(x / 2)
//   splits evenly (the other player mirrors moves on it), so both get sum floor(x / 2). Each
//   odd distinct value leaves one unpaired move worth its frequency; the players take these
//   in descending order, alternating. A tally and a sort, O(n log n).
// Bounds: n: 1..=200_000; a: 1..=10 ** 9

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()

let base = sum(x / 2 for x in a)
let extras = [cnt for (x, cnt) in a.tally().items if x % 2 == 1].sorted(by: x => desc(x))
var score: [0..2]Int = base
for (k, c) in extras.items: score[k % 2] += c
print(score[0], score[1])
