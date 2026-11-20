// SDOI 2017 Round 1 P3, Sequence Counting: sequences of n numbers from
// 1..=m whose sum is a multiple of k and that contain at least one prime.
// Count all sequences, subtract those with no prime; each count is the
// n-th power of a vector of residue counts under cyclic convolution.
// original: corpus/submissions/5463991.cpp
const P = 20170408
type Z = Mod<P>

let n: 1..=1e9 = read()
let m: 1..=200 = read()        // the statement allows 2e7; kept small for the checker
let k: 1..=20 = read()         // the statement allows 100
domain R = 0..k
type Vec = [R]Z

// c[(i + j) mod k] += a[i] * b[j]
fn conv(a: Vec, b: Vec) -> Vec:
    var c: Vec = 0
    for i in R:
        for j in R: c[R((i.idx + j.idx) % k)] += a[i] * b[j]
    c

// x convolved with itself e times, e >= 1
fn power(x: Vec, e: Int) -> Vec:
    var base = x
    var acc = x
    var t = e - 1
    while t > 0:
        if t % 2 == 1: acc = conv(acc, base)
        base = conv(base, base)
        t /= 2
    acc

var composite: [0..=m]Bool = false
for i in 2..=m:
    if not composite[i]:
        var j = 2 * i
        while j <= m:
            composite[j] = true
            j += i

var all: Vec = 0
var plain: Vec = 0
for x in 1..=m:
    all[R(x % k)] += 1
    if x == 1 or composite[x]: plain[R(x % k)] += 1
print(power(all, n)[R(0)] - power(plain, n)[R(0)])
