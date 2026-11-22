// Mock CCC '22 Contest 1 S2 (nccc9s2), buildable under proto/cpc today. See ../nccc9s2.cp.
// original: corpus/submissions/4051671.cpp
let n: 1..=1_000_000 = read()
let s: [0..2 * n]Char = read()
var ans = 0
var k = 0
for i in 0..(2 * n):
    if s[i] == 'I':
        ans += abs(2 * k - i)
        k += 1
print(ans)
