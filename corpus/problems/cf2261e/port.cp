// Source: Codeforces 2261E (What Does the Cow Say?), https://codeforces.com/contest/2261/problem/E
// limits: 2000 ms, 256 MB
// Problem: a regular bracket sequence of length n <= 4e5 and k <= n/2. One operation deletes an
//   adjacent "()" and wraps the rest as "(" + s + ")". Print the most consecutive '(' reachable
//   with at most k operations. Multitest, sum of n <= 4e5.
// Solution: try each '(' at i as the end of the run, with the stack of unmatched '(' before it.
//   If the matched characters before i number <= 2k, deleting them all leaves the stack plus k
//   wrapped '(': k + stack size. Else keep the longest top part of the stack whose span holds
//   <= 2k matched characters; that count falls along the stack, so binary search it. Answer
//   capped at n/2. O(n log n).
// Bounds: n: 2..=4e5; k: 0..=n / 2

@!multitest
let n: 2..=4e5 = read()
let k: 0..=n / 2 = read()
let s: [0..n]Char = read()

var open: List<Int> = []
var ans = 0
for (i, c) in s.items:
    if c == ')':
        open.pop()
        continue
    open.push(i)
    if i + 1 - open.len <= 2 * k:
        ans max= k + open.len
    else:
        let from = (0..open.len).first_true(j => (i + 1 - open[j]) - (open.len - j) <= 2 * k)!
        ans max= open.len - from
print(min(ans, n / 2))
