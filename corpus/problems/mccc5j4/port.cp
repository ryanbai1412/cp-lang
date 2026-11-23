// Source: Mock CCC '23 Contest 1 J4 (String Decryption), https://dmoj.ca/problem/mccc5j4
// Problem: N <= 1e9 and a string S (|S| <= 1e5) of lowercase letters and asterisks. Replace
//   each asterisk with a letter so the sum of letter values (a = 1, ..., z = 26) is N; print
//   the lexicographically smallest result, or `Impossible`.
// Solution: with r left to spend on w asterisks it is possible iff w <= r <= 26w. Fill the
//   asterisks left to right, each with the smallest letter that leaves the rest feasible:
//   max(1, r - 26(w - 1)). O(|S|).
// Note: runnable on proto/cp.py lib-run today.
// original: corpus/submissions/5333145.cpp

let k: Int = read()
var s: Str = read()

var rem = k
var wild = 0
for i in 0..s.len:
    let ch = s[i]
    if ch == '*':
        wild += 1
    else:
        rem -= ch - 'a' + 1

if rem < wild or rem > 26 * wild:
    print("Impossible")
else:
    for i in 0..s.len:
        if s[i] == '*':
            wild -= 1
            let d = max(1, rem - 26 * wild)
            s[i] = 'a' + (d - 1)
            rem -= d
    print(s)
