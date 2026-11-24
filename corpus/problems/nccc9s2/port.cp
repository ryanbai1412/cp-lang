// Source: Mock CCC '22 Contest 1 S2 (IU), https://dmoj.ca/problem/nccc9s2
// Problem: a string of n I's and n U's, n <= 1e6. Print the fewest swaps of adjacent letters
//   that turn it into IU repeated n times (-1 if impossible, which never happens).
// Solution: the k-th I (0-based) never needs to pass another I, so it goes to position 2k;
//   the answer is the sum of |2k - pos_k| over the I's. One scan, O(n).
// original: corpus/submissions/4051671.cpp
// Bounds: n: 1..=1_000_000
let n: 1..=1_000_000 = read()
domain P = 0..2 * n
let s: [P]Char = read()
let at = [p for p in P if s[p] == 'I']
print(sum(abs(2 * k - Int(p)) for (k, p) in at.items))
