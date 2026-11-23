// Codeforces 2255 D: bucket the values by level(x) = ceil(log2 x). A greedy
// over the buckets in level order bounds the answer: after bucket i, carry
// values are still uncleared, so the steps reach at least carry + i, and one
// value is retired per level. That bound is at most one too large; check
// directly whether steps ans-2, .., 0 suffice, always spending step i on the
// largest bucket and splitting its smallest value when it sits exactly at
// level i + 1.
let max_a = 1e9

fn level(x: 1..=max_a) -> Int = bit_width(x - 1)
let top_level = level(max_a)

@!multitest
let n: 1..=2e5 = read()
var freq: [0..=top_level]Int = 0
var least: [0..=top_level]?Int = None

fn add(x: 1..=max_a):
    let b = level(x)
    freq[b] += 1
    least[b] = min(least[b] ?? x, x)

for x in read<[n]1..=max_a>(): add(x)

var ans = 0
var carry = 0
for i in 0..=top_level:
    carry += freq[i]
    if carry > 0:
        ans max= carry + i
        carry -= 1

fn fewer() -> Bool:
    var top = top_level
    for i in (0..ans - 1).rev:
        top = (0..=top).rev.find(b => freq[b] > 0) ?? return true
        if top > i + 1: return false
        freq[top] -= 1
        if top == i + 1:
            if freq[top] > 0: return false
            add(least[top]! - (1 << i))
    freq.all(f => f == 0)

print(if fewer(): ans - 1 else: ans)
