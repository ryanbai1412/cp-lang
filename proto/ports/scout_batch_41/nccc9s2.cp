// Mock CCC '22 Contest 1 S2 (nccc9s2): minimum adjacent swaps to move the n
// 'I' characters in a 2n-char 'I'/'U' string to even positions 0, 2, ..., 2n-2.
// original: corpus/submissions/4051671.cpp
let n: 1..=1_000_000 = read()
let s: Str = read()
let pos = [i for (i, c) in s.items if c == 'I']
print(sum(abs(2 * k - p) for (k, p) in pos.items))
