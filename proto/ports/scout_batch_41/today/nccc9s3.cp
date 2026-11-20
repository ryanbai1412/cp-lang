// Mock CCC '22 Contest 1 S3 (nccc9s3), buildable under proto/cpc today. See ../nccc9s3.cp.
// original: corpus/submissions/4051697.cpp
let n: 1..=1_000_000 = read()
let s: [0..2 * n]Char = read()
var uu = Int(s[0] == 'U')
var ii = 0
for i in 0..(2 * n - 1):
    if s[i] == s[i + 1]:
        if s[i] == 'U': uu += 1
        else: ii += 1
print(max(uu, ii))
