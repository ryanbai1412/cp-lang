// Source: CTSC 2006 (Singing Kingdom), https://dmoj.ca/problem/ctsc06p1
// Problem: letters 1..m (m <= 1e5) are drawn uniformly at random until a given name appears
//   as a block of consecutive draws. For each of t <= 50 names (length <= 1e5), print the
//   last 4 digits of the expected number of draws (an integer), zero-padded.
// Solution: the expected wait is the sum of m^b over every border b of the name (a prefix
//   that is also a suffix), the whole name included. The failure function lists the borders
//   in O(len); add the powers mod 10^4 and print four digits.
// original: corpus/submissions/4528855.cpp
use std.strings.{borders}

let (m, t): (Int, Int) = read()
type Z = Mod<10000>

fn digit(x: Int) -> Str = "0123456789"[x % 10..x % 10 + 1].to_str()

for _ in 0..t:
    let n: Int = read()
    domain I = 0..n
    let name: [I]Int = read()
    var wait = Z(m) ** n
    for b in borders(name):
        wait += Z(m) ** b
    let w = wait.v
    print(digit(w / 1000) + digit(w / 100) + digit(w / 10) + digit(w))
