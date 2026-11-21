// Codeforces 2255 D: bucket the values by ceil(log2 a). A greedy over the
// buckets gives an answer that is at most one too large; then check directly
// whether one step fewer is enough, always spending step i on the largest
// bucket and splitting its smallest value when it sits exactly at level i+1.
const B = 30

fn level(x: Int) -> Int = bit_width(x - 1)

@!multitest
let n: 1..=2e5 = read()
let a: [0..n]Int = read()
var freq: [0..=B]Int = 0
var least: [0..=B]Int = 1 << B
for x in a:
    let b = level(x)
    freq[b] += 1
    least[b] min= x

var ans = 0
var carry = 0
for i in 0..=B:
    carry += freq[i]
    if carry > 0:
        ans max= carry + i
        carry -= 1

// Can everything be cleared with steps ans-2, ans-3, .., 0?
fn fewer() -> Bool:
    var l = B
    var i = ans - 2
    while true:
        while l >= 0 and freq[l] == 0: l -= 1
        if l < 0: return true
        if i == -1 or l > i + 1: return false
        freq[l] -= 1
        if l == i + 1:
            if freq[l] > 0: return false
            let rest = least[l] - (1 << i)
            let b = level(rest)
            freq[b] += 1
            least[b] min= rest
        i -= 1

print(ans - Int(fewer()))
