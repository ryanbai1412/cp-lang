// NOIP '19 P1, Gray Code (buildable today-form)
// original: corpus/submissions/5127723.cpp
// k goes up to 2^64 - 1 and the prototype has only the checked signed Int,
// so k is read as a digit string into two 32-bit halves.
let n: 1..=64 = read()
let digits: [domain Dg]Char = read()

let M32 = 4294967295         // not `(1 << 32) - 1`: emitted as a 32-bit shift today
var kh = 0
var kl = 0
for i in Dg:
    let lo = kl * 10 + (digits[i] - '0')
    kh = (kh * 10 + (lo >> 32)) & M32
    kl = lo & M32

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
