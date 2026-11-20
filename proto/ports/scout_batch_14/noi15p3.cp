// NOI 2015 Sushi Dinner: partition a subset of 2..=n between two people so no
// prime divides numbers on both sides, modulo a runtime p. Uses a ternary state
// [Small]Side over the 8 primes <= sqrt(500) plus binary inclusion-exclusion.
// original: corpus/submissions/3782724.cpp
use std.mod.{ModR}                                                // WISH P7
let n: 2..=500 = read()
let p: 1..=1e9 = read()
let z = ModR(p)                                                   // WISH P7

// Primes up to n; the first 8 (2..19) are the ones <= sqrt(500).
let primes = [x for x in 2..=n if all(x % d != 0 for d in 2..x if d * d <= x)]
let small_primes = [2, 3, 5, 7, 11, 13, 17, 19]
domain Small = 0..8
enum Side:
    Neither
    A
    B

// A ternary assignment of each small prime to Neither, A, or B: 3^8 = 6561 states.
type Assign = [Small]Side                                         // WISH P5
var ways: [Assign]Int = 0                                         // WISH P5

fn allowed(x: Int, assign: Assign) -> (a: Bool, b: Bool):
    var (ok_a, ok_b) = (true, true)                               // WISH P2
    for i in Small if x % small_primes[i.idx] == 0:               // WISH P9
        if assign[i] != Side.A: ok_a = false
        if assign[i] != Side.B: ok_b = false
    (ok_a, ok_b)

for assign in Assign:                                             // WISH P5
    var cur = z.of(1)
    var vis: [0..=n]Bool = false
    for q in primes if q > 19:                                    // WISH P9
        var (acnt, bcnt) = (0, 0)                                 // WISH P2
        var x = q
        while x <= n:
            vis[x] = true
            let ok = allowed(x, assign)
            if ok.a: acnt += 1
            if ok.b: bcnt += 1
            x += q
        cur = z.mul(cur, z.sub(z.add(z.pow(2, acnt), z.pow(2, bcnt)), 1))
    for x in 2..=n if not vis[x]:                                 // WISH P9
        let ok = allowed(x, assign)
        cur = z.mul(cur, 1 + (if ok.a: 1 else: 0) + (if ok.b: 1 else: 0))
    ways[assign] = cur

// Inclusion-exclusion over the 8 small primes: sum ways[assign] disjoint from s.
type Mask = Subset<Small>
var ans = 0
for s in Mask:
    var total = 0
    for assign in Assign if all(assign[b] == Side.Neither for b in s):   // WISH P5, P9
        total = z.add(total, ways[assign])
    ans = z.add(ans, z.mul(total, z.pow(-2, s.len)))
print(ans)
