// Source: NOIP 2019 P1 (CSP-S 2019, Gray Code), https://dmoj.ca/problem/noip19p1
// Problem: the n-bit Gray code (n <= 64) is 0, 1 for n = 1; for n + 1 bits it is the n-bit list
//   prefixed by 0, then the n-bit list reversed and prefixed by 1. Print string number k,
//   0 <= k < 2^n.
// Solution: string number k is k ^ (k >> 1): print bit i of k xor bit i + 1, most significant
//   first. O(n).
// original: corpus/submissions/5127723.cpp
// k goes up to 2^64 - 1 and the prototype has only the checked signed Int,
// so k is read as a digit string into two 32-bit halves.
let n: 1..=64 = read()
let digits: [domain Dg]Char = read()

let m32 = 4294967295  // not `(1 << 32) - 1`: emitted as a 32-bit shift today
var kh = 0
var kl = 0
for i in Dg:
    let lo = kl * 10 + (digits[i] - '0')
    kh = (kh * 10 + (lo >> 32)) & m32
    kl = lo & m32

fn bit(i: Int) -> Int:
    if i >= 64:
        return 0
    if i >= 32:
        return (kh >> (i - 32)) & 1
    (kl >> i) & 1

// bit i of k ^ (k >> 1), most significant first
var out: List<Char> = []
for i in (0..n).rev:
    if bit(i) != bit(i + 1):
        out.push('1')
    else:
        out.push('0')
print(out)
