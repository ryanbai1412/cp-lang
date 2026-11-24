// Source: CTSC 2006 (Singing Kingdom), https://dmoj.ca/problem/ctsc06p1
// Problem: letters 1..m (m <= 1e5) are drawn uniformly at random until a given name appears
//   as a block of consecutive draws. For each of t <= 50 names (length <= 1e5), print the
//   last 4 digits of the expected number of draws (an integer), zero-padded.
// Solution: the expected wait is the sum of m^b over every border b of the name (a prefix
//   that is also a suffix), the whole name included. The failure function lists the proper
//   borders in O(len); add the powers mod 10^4 and print four digits.
// original: corpus/submissions/4528004.cpp
type Z = Mod<10_000>
let m: 1..=100_000 = read()
let t: 1..=50 = read()
for _ in 0..t:
    let name: List<1..=m> = read()
    let wait = Z(m) ** name.len + sum(Z(m) ** b for b in borders(name))
    print("{wait.v:04}")
