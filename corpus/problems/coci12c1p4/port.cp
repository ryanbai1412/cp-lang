// Source: COCI 2012/2013 Contest 1 #4 (Ljubomora), https://dmoj.ca/problem/coci12c1p4
// Problem: m <= 3e5 colours with 1..=1e9 marbles each go to n <= 1e9 children (m <= n); a
//   child gets marbles of one colour only, or none. Print the least possible maximum number
//   of marbles one child gets.
// Solution: with at most x per child, colour c needs ceil(c / x) children, and that total only
//   falls as x grows: binary search (`first_true`) the least x in 1..=1e9 with
//   sum ceil(c / x) <= n, O(m log 1e9).
// original: corpus/submissions/5245322.cpp
let kids: 1..=1e9 = read()
let m: 1..=3e5 = read()
assert m <= kids
let marbles: [m]1..=1e9 = read()
assert marbles.sum() > kids  // the original divides by zero when the answer is 1

// with handfuls of at most x, colour c needs ceil(c / x) children
print((1..=1e9).first_true(x => sum(ceil_div(c, x) for c in marbles) <= kids)!)
