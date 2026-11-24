// Source: Cheerio Contest 1 S5 (Coin Game), https://dmoj.ca/problem/cheerio1s5
// Problem: a row of N <= 400 coins, each H or T; make K moves (1 <= K <= number of H's), each
//   flipping an H to T and scoring the length of the run of tails holding that coin after
//   the flip. Print the maximum total score.
// Solution: the flipped coins form blocks between unflipped H's; in a block the last flip
//   scores the whole block and splits it into two blocks cleared before. Interval DP
//   clear[l, r] = max over H at i of (r - l) + clear[l, i] + clear[i + 1, r], O(N^3); then
//   best[i, used] picks consecutive blocks over s[i..] holding at most K H's, O(N^2 K).
// Written in today's buildable form.
// original: corpus/submissions/5339756.cpp
let s: [domain S]Char = read()
let k: 0..=1000 = read()
let n = S.len
let head: List<Bool> = [s[i] == 'H' for i in S]
let heads = prefix([Int(head[i]) for i in 0..n])

// clear[l, r]: best score from the block l..r, every pick splitting what is left
var clear: [0..=n, 0..=n]Int = 0
for l in (0..n).rev:
    for r in l + 1..=n:
        for i in l..r:
            if head[i]: clear[l, r] max= r - l + clear[l, i] + clear[i + 1, r]

// best[i, used]: best score from s[i..] with `used` H's already inside chosen blocks
var best: [0..=n, 0..=k]Int = 0
for i in (0..n).rev:
    for used in 0..=k:
        best[i, used] = best[i + 1, used]
        for j in i + 1..=n:
            let now = used + heads.prefix(j) - heads.prefix(i)
            if now <= k: best[i, used] max= clear[i, j] + best[j, now]
print(best[0, 0])
