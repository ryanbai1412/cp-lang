// Source: CCO 2001 P4 (Breaking Level, Standard I/O), https://dmoj.ca/problem/cco01p4io
// limits: 2000 ms, 64 MB
// Problem: interactive. A tower of N <= 100 levels has a breaking level H: an object breaks
//   when dropped from H or higher. With two samples and at most D drops (GetN, GetD, Drop t
//   answering 1 if it breaks), find H and print Decide H.
// Solution: drops[s], the fewest drops that always settle s + 1 candidate levels: drop the
//   first sample i levels above the lowest; if it breaks, test the i levels below one by one
//   with the second, else settle the levels above. drops[s] = min over i of
//   1 + max(i, drops[s - i - 1]), O(N^2) with the best i remembered, then follow it.
//   Asserts drops <= D.
let n: 1..=100 = ask("GetN")
let d: 1..=100 = ask("GetD")

domain Span = 0..n  // one less than the candidate levels left
var drops: [Span]Int = 0
var up: [Span]Int = 0  // where the first sample goes, above the lowest candidate
for s in Span if s.idx > 0:
    let (cost, i) = min((1 + max(i, drops[Span(s.idx - i - 1)]), i) for i in 0..s.idx)!
    drops[s] = cost
    up[s] = i
assert drops[Span.last!] <= d

fn breaks(level: Int) -> Bool:
    let broke: 0..=1 = ask("Drop", level)
    broke == 1

var (lo, hi) = (1, n)  // the breaking level is in lo..=hi
while lo < hi:
    let at = lo + up[Span(hi - lo)]
    if breaks(at): lo = (lo..at).find(breaks) ?? at; hi = lo
    else: lo = at + 1
print("Decide", lo)
