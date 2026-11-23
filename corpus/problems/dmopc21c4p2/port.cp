// Source: DMOPC '21 Contest 4 P2 (Festive Groups), https://dmoj.ca/problem/dmopc21c4p2
// Problem: split N <= 2e6 numbers J_i <= 5e6 into the fewest groups such that each group's
//   minimum divides every member; print the number of groups.
// Solution: an element needs its own group exactly when no element before it in sorted order
//   divides it; otherwise it joins that divisor's group. Sort, then for each x list its
//   divisors from a sieve factorization and keep x only if no divisor is kept already.
//   O(J log log J + sum of divisor counts).
// original: corpus/submissions/4125290.cpp

let n: Int = read()
domain I = 0..n
let raw: [I]Int = read()
let a = raw.sorted()
let sieve = Sieve.new(a.max()!)

fn divisors(x: Int) -> List<Int>:
    var ds = [1]
    for pe in sieve.factor(x):
        let base = ds.len
        var mult = 1
        for _ in 0..pe.1:
            mult *= pe.0
            for j in 0..base: ds.push(ds[j] * mult)
    ds

// Ascending order: keep x unless one of its divisors was kept already.
var kept = Set<Int>()
for x in a:
    var blocked = false
    for d in divisors(x):
        if d in kept: blocked = true
    if not blocked: kept.add(x)
print(kept.len)
