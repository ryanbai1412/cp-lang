// Source: IOI 1999 Day 1 P2 (Hidden Codes), https://dmoj.ca/problem/ioi99p2
// Problem: N <= 100 case-sensitive code words (length <= 100, IDs 1..=N) and a text of
//   length <= 1e6. A covering sequence of a word is a substring that starts and ends with
//   the word's first and last letters and contains it as a subsequence, of length <= 1000.
//   Choose non-overlapping covering sequences maximizing the total word length; print the
//   sum, then one "id start end" line per item (1-based, any order).
// Solution: for an end position the latest start is best. Scan the text once; for each
//   letter, the (word, index) pairs it matches (stored in reverse index order) update
//   start_pos[word, index], the latest start of that word prefix so far, in O(1). When a
//   word completes at i within the span limit, dp[i] = max(dp[i - 1], len + dp[start - 1]);
//   walk the choices back to print. O(|text| * matching letter slots).
// original: corpus/submissions/4896839.cpp
let m: 1..=1000 = read()
var by_char: [0..256]List<(Int, Int)> = []
var word_len: [0..m]Int = 0
var max_wlen = 0
for i in 0..m:
    let w: [domain W]Char = read()
    let wl = w.len
    word_len[i] = wl
    max_wlen max= wl
    for j in (0..wl).rev:
        by_char[Int(w[W(j)])].push((i, j))

let s: [domain S]Char = read()
let n = s.len

var start_pos: [0..m, 0..max_wlen]Int = -1
var dp: [0..n]Int = 0
var opt_word: [0..n]Int = -1
var opt_prev: [0..n]Int = -1

for i in 0..n:
    dp[i] = if i > 0: dp[i - 1] else: 0
    opt_word[i] = -1
    opt_prev[i] = i - 1
    for e in by_char[Int(s[S(i)])]:
        let wi = e.0
        let pos = e.1
        if pos == 0:
            start_pos[wi, 0] = i
        else:
            start_pos[wi, pos] = start_pos[wi, pos - 1]
        if pos == word_len[wi] - 1:
            let st = start_pos[wi, pos]
            if st != -1 and i - st <= 1000:
                let prev_dp = if st > 0: dp[st - 1] else: 0
                let cand = word_len[wi] + prev_dp
                if cand > dp[i]:
                    dp[i] = cand
                    opt_word[i] = wi
                    opt_prev[i] = st - 1

print(dp[n - 1])
var cur = n - 1
while cur >= 0:
    let wi = opt_word[cur]
    let nxt = opt_prev[cur]
    if wi != -1:
        print(wi + 1, nxt + 2, cur + 1)
    cur = nxt
