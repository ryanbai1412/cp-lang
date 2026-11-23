// Source: Mock CCC '22 Contest 1 S3 (IU), https://dmoj.ca/problem/nccc9s3
// Problem: a string of n I's and n U's, n <= 1e5. One operation picks up a letter and puts it
//   anywhere. Print the fewest operations that turn it into IU repeated n times (-1 if
//   impossible, which never happens).
// Solution: count the adjacent UU pairs (plus a leading U) and the adjacent II pairs; one move
//   breaks at most one of each, and max of the two counts moves suffice. One scan, O(n).
// Buildable under proto/cpc today. See ../nccc9s3.cp.
// original: corpus/submissions/4051697.cpp
let n: 1..=1e5 = read()
let s: [0..2 * n]Char = read()
var uu = Int(s[0] == 'U')
var ii = 0
for i in 0..(2 * n - 1):
    if s[i] == s[i + 1]:
        if s[i] == 'U': uu += 1
        else: ii += 1
print(max(uu, ii))
