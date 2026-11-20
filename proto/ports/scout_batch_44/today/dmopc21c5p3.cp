// Runnable port (proto/cp.py lib-run) of dmopc21c5p3.cp
// original: corpus/submissions/4218397.cpp
use std.math.{Sieve}

let t: Int = read()
var ns: List<Int> = []
for _ in 0..t:
    let n: Int = read()
    ns.push(n)
let limit = max(max(ns), 2)
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
        if line.len > 0: line = line + " "
        line = line + "{x}"
    print(line)

for n in ns:
    if n <= 2:
        print(-1)
    elif win[n - 1]:
        show([i for i in 1..=n])
    else:
        // A losing pile: perturb the tail. Three sizes need the 3-cycle
        // (n, n-2, n-1); every other one is fixed by swapping the last two.
        var tail = [n, n - 1]
        if n == 11 or n == 36 or n == 311: tail = [n, n - 2, n - 1]
        let head = [i for i in 1..=(n - tail.len)]
        show(head + tail)
