// Source: IOI 1999 Day 1 P2 (Hidden Codes), https://dmoj.ca/problem/ioi99p2
// limits: 1000 ms, 1024 MB
// Problem: 1 <= N <= 100 case-sensitive code words (letters, length <= 100, IDs 1..=N) and
//   a text of length 1..=1e6. A covering sequence of a word is a substring that starts and
//   ends with the word's first and last letters and contains it as a subsequence, of length
//   <= 1000.
//   Choose non-overlapping covering sequences maximizing the total word length; print the
//   sum, then one "id start end" line per item (1-based, any order).
// Solution: for an end position the latest start is best. Scan the text once; for each
//   letter, the (word, index) pairs it matches (stored in reverse index order) update
//   start_pos[word, index], the latest start of that word prefix so far, in O(1). When a
//   word completes at i within the span limit, dp[i] = max(dp[i - 1], len + dp[start - 1]);
//   a span limit of 1000 letters means i - start < 1000. Walk the choices back to print.
//   O(|text| * matching letter slots).
// original: corpus/submissions/4896839.cpp
const SPAN = 1000
let m: 1..=100 = read()
domain Word = 0..m
let words: [Word]Str = read()
assert all(1 <= words[w].len <= 100 for w in Word)
let text: [domain T]Char = read()
assert 1 <= T.count <= 1_000_000

// per letter, the (word, index) slots it fills, each word's indices descending so a prefix
// grows from the previous letter's value before this letter overwrites it
let slots = [(w:, j:) for w in Word for j in (0..words[w].len).rev].group(by: s => words[s.w][s.j])

// start[w][j]: the latest start of a covering of w's first j + 1 letters ending here
var start: [Word]List<?T> = [Word] w => [None for _ in 0..words[w].len]
// best[c]: the most word length coverable in the text before cut c, and the last pick
var best: [T.cuts]Int = 0
var took: [T.cuts]?(w: Word, from: T.cuts) = None
for i in T:
    best[i.r] = best[i.l]
    for s in slots.get(text[i]) ?? []:
        start[s.w][s.j] = if s.j == 0: Some(i) else: start[s.w][s.j - 1]
        if s.j == words[s.w].len - 1:
            if let from = start[s.w][s.j] and Int(i) - Int(from) < SPAN and words[s.w].len + best[from.l] > best[i.r]:
                best[i.r] = words[s.w].len + best[from.l]
                took[i.r] = Some((w: s.w, from: from.l))

print(best[T.cuts.last!])
var at = T.cuts.last!
while at != T.cuts.first!:
    if let t = took[at]:
        print(Int(t.w) + 1, Int(t.from) + 1, Int(at))
        at = t.from
    else:
        at = T.cuts.at(Int(at) - 1)!
