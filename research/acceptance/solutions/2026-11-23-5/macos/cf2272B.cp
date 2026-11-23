// B. Frogs Jump
// https://codeforces.com/contest/2272/problem/B
// judge: generic

@!multitest
let n: 1..=2e5 = read()
let (s, t): (Str, Str) = read()
print((s.count(c => c == 'f') + t.count(c => c == 'f')) % 2)
