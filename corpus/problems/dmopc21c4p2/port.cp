// Source: DMOPC '21 Contest 4 P2 (Festive Groups), https://dmoj.ca/problem/dmopc21c4p2
// Problem: split N <= 2e6 numbers J_i <= 5e6 into the fewest groups such that each group's
//   minimum divides every member; print the number of groups.
// Solution: an element needs its own group exactly when no element before it in sorted order
//   divides it; otherwise it joins that divisor's group. Walk the values up; a present value not
//   yet covered starts a group and covers its multiples (a joined value's multiples are
//   covered by its group's minimum already). O(J log J).
// original: corpus/submissions/4125290.cpp
// Ascending order: keep x unless one of its divisors was kept already.

let n: 1..=2e6 = read()
let size: [n]1..=5e6 = read()
domain Size = 1..=size.max()!

var present: [Size]Bool = false
for s in size: present[Size(s)] = true
var covered: [Size]Bool = false
var groups = 0
for v in Size if present[v] and not covered[v]:
    groups += 1
    for m in (v.label..=Size.last!.label).step(v.label): covered[Size(m)] = true
print(groups)
