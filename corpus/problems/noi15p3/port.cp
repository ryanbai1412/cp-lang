// NOI 2015 Sushi Dinner: partition a subset of 2..=n between two people so no
// prime divides numbers on both sides, modulo a runtime p. Uses base-3 masks
// over the 8 primes <= sqrt(500) plus binary inclusion-exclusion.
// original: corpus/submissions/3782724.cpp
let n: 2..=500 = read()
let p: 1..=1000000000 = read()

// Sieve primes up to n and index them 1..=count.
var prime_id: [0..=n]Int = 0
var pcnt = 0
for i in 2..=n:
    var is_p = true
    var d = 2
    while d * d <= i:
        if i % d == 0:
            is_p = false
            break
        d += 1
    if is_p:
        pcnt += 1
        prime_id[i] = pcnt

var fact: [0..=n]List<Int> = []
for i in 2..=n:
    if prime_id[i] > 0:
        var j = i
        while j <= n:
            fact[j].push(i)
            j += i

fn mod_pow(base0: Int, exp0: Int) -> Int:
    var ans = 1 % p
    var base = base0 % p
    var exp = exp0
    while exp > 0:
        if (exp & 1) == 1: ans = (ans * base) % p
        base = (base * base) % p
        exp = exp / 2
    ans

// 8 small primes (2..19); 3^8 = 6561 ternary assignments (0=neither, 1=A, 2=B).
let pow3 = [1, 3, 9, 27, 81, 243, 729, 2187, 6561]
let sqn = 8
let total3 = pow3[sqn]
var ways: [0..total3]Int = 0

for m3 in 0..total3:
    var cur_way = 1 % p
    var vis: [0..=n]Bool = false
    for j in 2..=n:
        if vis[j]: continue
        if prime_id[j] > sqn:
            var acnt = 0
            var bcnt = 0
            var k = j
            while k <= n:
                vis[k] = true
                var ok_a = true
                var ok_b = true
                for c in fact[k]:
                    if c == j: continue
                    let digit = (m3 / pow3[prime_id[c] - 1]) % 3
                    if digit == 0:
                        ok_a = false
                        ok_b = false
                    else:
                        if digit != 1: ok_a = false
                        if digit != 2: ok_b = false
                if ok_a: acnt += 1
                if ok_b: bcnt += 1
                k += j
            let factor = (mod_pow(2, acnt) + mod_pow(2, bcnt) - 1) % p
            cur_way = (cur_way * factor) % p
        else:
            var ok_a = true
            var ok_b = true
            for c in fact[j]:
                let digit = (m3 / pow3[prime_id[c] - 1]) % 3
                if digit == 0:
                    ok_a = false
                    ok_b = false
                else:
                    if digit != 1: ok_a = false
                    if digit != 2: ok_b = false
            let factor = 1 + Int(ok_a) + Int(ok_b)
            cur_way = (cur_way * factor) % p
    ways[m3] = cur_way

// Inclusion-exclusion over the 8 small primes: sum over m3 disjoint from mask.
domain Small = 0..sqn
type Mask = Subset<Small>
var ans = 0
for s in Mask:
    var total = 0
    for m3 in 0..total3:
        var clash = false
        for b in s:
            if (m3 / pow3[b.idx]) % 3 != 0:
                clash = true
                break
        if not clash: total = (total + ways[m3]) % p
    let coeff = mod_pow(-2, s.len)
    ans = (ans + total * coeff) % p
print(ans)
