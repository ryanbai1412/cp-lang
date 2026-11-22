// IOI 1999 P2 Hidden Codes: given m dictionary words and a target ASCII text s,
// find a maximum-total-length set of non-overlapping occurrences of words as
// subsequences of s, where each occurrence spans at most 1001 characters
// (end - start <= 1000). Output total length and (word_id, start, end) triples.
// Index word character positions in reverse per ASCII byte (0..256) so one left-
// to-right scan updates the latest valid start for every word prefix in O(1),
// driving a linear DP with backtracking reconstruction.
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
