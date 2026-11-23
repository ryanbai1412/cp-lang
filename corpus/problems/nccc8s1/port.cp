// Mock CCC '21 S1: given an alphabet and a word s, print in sorted order,
// without repeats, every word other than s one edit away: delete a letter,
// replace a letter, or insert a letter of the alphabet anywhere.
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
