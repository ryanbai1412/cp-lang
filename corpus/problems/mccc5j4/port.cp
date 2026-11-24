// Source: Mock CCC '23 Contest 1 J4 (String Decryption), https://dmoj.ca/problem/mccc5j4
// limits: 1000 ms, 512 MB
// Problem: N <= 1e9 and a string S (|S| <= 1e5) of lowercase letters and asterisks. Replace
//   each asterisk with a letter so the sum of letter values (a = 1, ..., z = 26) is N; print
//   the lexicographically smallest result, or `Impossible`.
// Solution: with r left to spend on w asterisks it is possible iff w <= r <= 26w. Fill the
//   asterisks left to right, each with the smallest letter that leaves the rest feasible:
//   max(1, r - 26(w - 1)). O(|S|).
// Bounds: goal: 1..=1e9
// original: corpus/submissions/5333145.cpp
let goal: 1..=1_000_000_000 = read()
var s: [domain P]Char = read()
assert 1 <= P.count <= 100_000
fn value(c: Char) -> 1..=26 = c - 'a' + 1

// r left to spend on w asterisks is feasible iff w <= r <= 26w
var wild = s.count(c => c == '*')
var left = goal - sum(value(c) for c in s if c != '*')
if not (wild <= left <= 26 * wild):
    print("Impossible")
else:
    for p in P:
        if s[p] == '*':
            wild -= 1
            let d = max(1, left - 26 * wild)
            s[p] = 'a' + (d - 1)
            left -= d
    print(Str(s))
