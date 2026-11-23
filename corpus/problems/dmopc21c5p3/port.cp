// Source: DMOPC '21 Contest 5 P3, https://dmoj.ca/problem/dmopc21c5p3
// Problem: heights of n buildings in a row form a permutation p. A token starts on height n;
//   players alternately move it from i to j with p_i > p_j and |i - j| prime, and who can't
//   move loses. Print the lexicographically smallest p where the first player wins, or -1.
//   t <= 1e5 tests, n <= 1e5, sum of n <= 5e6.
// Solution: in the identity the token moves from i to any i - prime >= 1, a subtraction
//   game; win[d] for every distance d up to the largest n (sieve, then try primes until one
//   reaches a losing d). n <= 2 is -1; if win[n - 1], print 1..n. Otherwise swap the last
//   two values, except n = 11, 36, 311, whose last three values are n, n - 2, n - 1.
// Runnable port (proto/cp.py lib-run) of dmopc21c5p3.cp
// original: corpus/submissions/4218397.cpp

let t: Int = read()
var ns: List<Int> = []
for _ in 0..t:
    let n: Int = read()
    ns.push(n)
let limit = max(ns.max(), 2)
let sieve = Sieve.new(limit)

// win[i]: the player to move wins a pile of i when a move removes a prime.
var win: List<Bool> = [false for _ in 0..=limit]
for i in 1..=limit:
    for p in sieve.primes:
        if p > i: break
        if not win[i - p]:
            win[i] = true
            break

fn show(xs: List<Int>):
    var line = ""
    for x in xs:
        if line.len > 0: line += " "
        line += "{x}"
    print(line)

for n in ns:
    if n <= 2:
        print(-1)
    elif win[n - 1]:
        show(List(1..=n))
    else:
        // A losing pile: perturb the tail. Three sizes need the 3-cycle
        // (n, n-2, n-1); every other one is fixed by swapping the last two.
        var tail = [n, n - 1]
        if n == 11 or n == 36 or n == 311: tail = [n, n - 2, n - 1]
        let head = [i for i in 1..=(n - tail.len)]
        show(head + tail)
