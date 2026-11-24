// Source: Mock CCC '21 S1 (Edit Distance), https://dmoj.ca/problem/nccc8s1
// Problem: an alphabet and a word s (|s| <= 100): print, sorted and without repeats, every word
//   other than s one edit away: delete a letter, replace one, or insert a letter of the
//   alphabet anywhere.
// Solution: generate every deletion, replacement and insertion, O(|s| |alphabet|) words of
//   length O(|s|), then sort, drop duplicates and skip s itself.
// original: corpus/submissions/3381111.java
let al: Str = read()
let s: Str = read()
let n = s.len
var near: List<Str> = []
for i in 0..=n:
    if i < n: near.push(s[0..i] + s[i + 1..n])
    for c in al:
        if i < n: near.push(s[0..i] + c + s[i + 1..n])
        near.push(s[0..i] + c + s[i..n])
for w in near.sorted().distinct():
    if w != s: print(w)
