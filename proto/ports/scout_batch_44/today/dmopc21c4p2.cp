// Runnable port (proto/cp.py lib-run) of dmopc21c4p2.cp
// original: corpus/submissions/4125290.cpp

let n: Int = read()
domain I = 0..n
let raw: [I]Int = read()
let a = sorted(raw)
let sieve = Sieve.new(max(a))

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
