// NOIP 2021 P1, Count Off: a number is skipped when it has a digit 7 or
// is a multiple of such a number. For each x: -1 if x is skipped, else the
// next number that is not.
// A sieve over "bad" numbers instead of primes: the original's second
// attempt; its first (factor every x, test every divisor) is still in the file.
// original: corpus/submissions/5445676.cpp
const LIMIT = 10000100                       // past the next survivor above 1e7

let t: 1..=2e5 = read()
let asks: [0..t]1..=1e7 = read()

fn has7(x: Int) -> Bool:
    var rest = x
    while rest > 0:
        if rest % 10 == 7: return true
        rest /= 10
    false

var skipped: [0..=LIMIT]Bool = false
for i in 1..=LIMIT:
    if not skipped[i] and has7(i):
        var j = i
        while j <= LIMIT:
            skipped[j] = true
            j += i

// after[x]: the smallest survivor above x
var after: [0..=LIMIT]Int = 0
var seen = 0
for x in (1..=LIMIT).rev:
    after[x] = seen
    if not skipped[x]: seen = x

for x in asks: print(if skipped[x]: -1 else: after[x])
