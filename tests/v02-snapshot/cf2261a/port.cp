// Source: Codeforces 2261A (Hey, It's Me, Souvlakity),
//   https://codeforces.com/contest/2261/problem/A
// Problem: the integers 1..=n (n <= 100) are split by a binary string s (1 = Souvlakity, at least
//   one). For 10^100 turns, Souvlakity first, the player to move gives one own integer to the
//   other. Souvlakity's score is max - min of his set; he maximises it and Kalamakity minimises
//   it. Print the final score. Multitest.
// Solution: each pair of turns gives Souvlakity one integer and takes one, so he ends with
//   c = the number of 1s in s. c = 1 scores 0 and c = 2 scores 1 (Kalamakity can always answer
//   with a neighbour). With c >= 3 he keeps his outermost two and only passes middle ones
//   around, so the score is the span last 1 minus first 1. One scan, O(n).

@!multitest
let n: 1..=200_000 = read()
let s: [0..n]Char = read()
let cnt = s.count(c => c == '1')
if cnt <= 2:
    print(cnt - 1)
else:
    let lo = (0..n).find(i => s[i] == '1')!
    let hi = (0..n).rev.find(i => s[i] == '1')!
    print(hi - lo)
