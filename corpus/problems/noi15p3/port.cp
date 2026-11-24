// Source: NOI 2015 P3 (Sushi Dinner), https://dmoj.ca/problem/noi15p3
// limits: 1000 ms, 512 MB
// Problem: count pairs of subsets X, Y of {2..n} with every x in X coprime to every y in Y,
//   n <= 500, modulo a runtime p <= 1e9.
// Solution: every number has at most one prime > 19. Give each of the 8 small primes a side
//   (none, X, Y; 3^8 masks); per mask, numbers without a big prime pick a side among the
//   allowed ones, and each big prime's multiples go wholly to X, wholly to Y, or nowhere:
//   2^a + 2^b - 1 ways. A prime used by nobody is counted under all three digits, so
//   inclusion-exclusion with weight (-2)^|S| over small primes forced to "none" fixes it.
//   O(3^8 n) plus O(2^8 3^8).
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
