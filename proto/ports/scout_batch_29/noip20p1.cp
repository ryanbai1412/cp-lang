// NOIP 2020 P1, Drainage: a DAG of n junctions. Junctions 1..=m each
// receive one ton of water; a junction splits what it holds evenly over
// its 1 to 5 outgoing pipes. Print what each junction without outgoing
// pipes ends up with, as a reduced fraction `p q`.
//
// A path has up to 11 splits, so a reduced denominator reaches 60^11,
// about 3.6e19: one factor past `Int`. Every denominator is 2^a 3^b 5^c,
// so an amount is a wide numerator over three exponents, and nothing
// needs a gcd.
// input: n m, then per junction its pipe count d and d targets
// original: corpus/submissions/5445672.cpp
use scout.wide.{Wide}

domain Small = 0..3
let primes = [Small] p => [2, 3, 5][p.ord]
type Amount = (num: Wide, exp: [Small]Int)

/// `x` rewritten over the denominator with exponents `exp`.
fn raised(x: Amount, exp: [Small]Int) -> Wide:
    var num = x.num
    for p in Small:
        for _ in x.exp[p]..exp[p]: num = num.times(primes[p])
    num

fn plus(x: Amount, y: Amount) -> Amount:
    let exp = [Small] p => max(x.exp[p], y.exp[p])
    (num: raised(x, exp).plus(raised(y, exp)), exp:)

/// `x` split over `d` pipes, `d` from 1 to 5.
fn shared(x: Amount, d: Int) -> Amount:
    var exp = [Small] p => x.exp[p]
    var rest = d
    for p in Small:
        while rest % primes[p] == 0:
            rest /= primes[p]
            exp[p] += 1
    (num: x.num, exp:)

let (n, m): (Int, Int) = read()
domain Junction = 1..=n
var pipes: [Junction]List<Junction> = [Junction] u => []
var waiting: [Junction]Int = [Junction] u => 0     // pipes in, not yet drained
for u in Junction:
    let d: Int = read()
    for _ in 0..d:
        let v: Junction = read()
        pipes[u].push(v)
        waiting[v] += 1

var holds: [Junction]Amount = [Junction] u => (num: Wide.of(if u.label <= m: 1 else: 0), exp: [Small] p => 0)
var ready = [u for u in Junction if waiting[u] == 0]
while ready.len > 0:
    let u = ready.pop()
    if pipes[u].len == 0: continue
    let part = shared(holds[u], pipes[u].len)
    for v in pipes[u]:
        holds[v] = plus(holds[v], part)
        waiting[v] -= 1
        if waiting[v] == 0: ready.push(v)

for u in Junction:
    if pipes[u].len > 0: continue
    var (num, den) = (holds[u].num, Wide.of(1))
    for p in Small:
        for _ in 0..holds[u].exp[p]:
            if num.rem(primes[p]) == 0: num = num.over(primes[p]) else: den = den.times(primes[p])
    print(num.show(), den.show())
