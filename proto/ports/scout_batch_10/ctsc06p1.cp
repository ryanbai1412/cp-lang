// CTSC 2006 Singing Kingdom: letters 1..m are drawn uniformly at random until
// a given name appears as a block of consecutive draws. Print the expected
// number of draws modulo 10^4, as four digits, for each of t names.
//
// The original solves a recurrence over the failure function. The closed
// form is shorter: the expected wait is the sum of m^b over every border b
// of the name, the whole name included (a border is a prefix that is also a
// suffix).
// original: corpus/submissions/4528855.cpp
use std.strings.{prefix_function}

let (m, t): (Int, Int) = read()
type Z = Mod<10000>

fn digit(x: Int) -> Str = "0123456789"[x % 10..x % 10 + 1]

for _ in 0..t:
    let n: Int = read()
    domain I = 0..n
    let name: [I]Int = read()
    let border = prefix_function([name[i] for i in I])
    var (wait, b) = (Z.zero, n)
    while b > 0:
        wait += Z(m).pow(b)
        b = border[b - 1]
    let w = wait.v
    print(digit(w / 1000) + digit(w / 100) + digit(w / 10) + digit(w))
