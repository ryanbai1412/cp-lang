// Mock CCC '22 Contest 1 S3 (nccc9s3): prepend 'U' to a 2n-char 'U'/'I' string
// and return max(adjacent 'UU' count, adjacent 'II' count).
// original: corpus/submissions/4051697.cpp
let n: 1..=1_000_000 = read()
let s: Str = read()
let t = "U" + s
let uu = count(1 for (a, b) in t.adjacent if a == 'U' and b == 'U')
let ii = count(1 for (a, b) in t.adjacent if a == 'I' and b == 'I')
print(max(uu, ii))
