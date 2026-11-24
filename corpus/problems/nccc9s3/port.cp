// Source: Mock CCC '22 Contest 1 S3 (IU), https://dmoj.ca/problem/nccc9s3
// limits: 250 ms, 1024 MB
// Problem: a string of n I's and n U's, n <= 1e5. One operation picks up a letter and puts it
//   anywhere. Print the fewest operations that turn it into IU repeated n times (-1 if
//   impossible, which never happens).
// Solution: count the adjacent UU pairs (plus a leading U) and the adjacent II pairs; one move
//   breaks at most one of each, and max of the two counts moves suffice. One scan, O(n).
// original: corpus/submissions/4051697.cpp
// Bounds: n: 1..=1e5
let n: 1..=1e5 = read()
domain P = 0..2 * n
let s: [P]Char = read()
let uu = Int(s[P.first!] == 'U') + s.adjacent.count(p => p == ('U', 'U'))
let ii = s.adjacent.count(p => p == ('I', 'I'))
print(max(uu, ii))
