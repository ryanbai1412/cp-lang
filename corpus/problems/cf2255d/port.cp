// Source: Codeforces 2255D, https://codeforces.com/contest/2255/problem/D
// limits: 2000 ms, 256 MB
// Problem: multitest; n <= 2e5 integers 1 <= a_i <= 1e9 (sum n <= 2e5). Each second pick p:
//   a_p becomes floor(a_p / 2), every other a_i becomes ceil(a_i / 2). Print the fewest seconds
//   until all are 0.
// Solution: bucket the values by level(x) = ceil(log2 x). A greedy over the levels in order
//   bounds the answer by max(carry + i), retiring one value per level, and that bound is at
//   most one too large. fewer() checks ans - 1 directly: step i goes to the largest non-empty
//   level, splitting its smallest value when it sits at level i + 1.
//   O(n + log^2 max a) per test.
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
