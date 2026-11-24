// Source: Mock CCC '21 S1 (Edit Distance), https://dmoj.ca/problem/nccc8s1
// limits: 250 ms, 1024 MB
// Problem: an alphabet and a word s (|s| <= 100): print, sorted and without repeats, every word
//   other than s one edit away: delete a letter, replace one, or insert a letter of the
//   alphabet anywhere.
// Solution: generate every deletion, replacement and insertion, O(|s| |alphabet|) words of
//   length O(|s|), then sort, drop duplicates and skip s itself.
// original: corpus/submissions/3381111.java
let alphabet: Str = read()
let s: Str = read()
assert 1 <= s.len <= 100
let n = s.len
let deleted = [s[0..i] + s[i + 1..n] for i in 0..n]
let replaced = [s[0..i] + c + s[i + 1..n] for i in 0..n for c in alphabet]
let inserted = [s[0..i] + c + s[i..n] for i in 0..=n for c in alphabet]
for w in (deleted + replaced + inserted).sorted().distinct():
    if w != s: print(w)
