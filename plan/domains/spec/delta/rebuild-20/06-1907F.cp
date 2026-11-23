// CF 1907F Shift and Reverse. Every reachable array is a rotation of a or of
// rev a. If a read from s is non-decreasing: s shifts right n - s times, or
// reverse, shift s, reverse (s + 2). If it is non-increasing: shift n - s and
// reverse (n - s + 1), or reverse and shift s (s + 1). The windows of a ++ a
// equal to sorted a (or its reverse) are found with KMP.
@!multitest
let n: 1..=1e5 = read()
let a = read<[n]1..=1e9>()
let b = List(chain(a, a))
let up = a.sorted()
let down = a.sorted(by: desc)
var best: ?Int = None
for s in b.find_all(up) if s < n:
    best min= if s == 0: 0 else: min(n - s, s + 2)
for s in b.find_all(down) if s < n:
    best min= min(n - s + 1, s + 1)
print(best ?? -1)
