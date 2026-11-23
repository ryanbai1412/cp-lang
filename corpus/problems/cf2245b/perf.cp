// Tuned: select the median in place (introselect, O(n)) instead of sorting, as orig.cpp's
// nth_element does; only the split matters, not the order within each half.
@!multitest
let (n, c): (1..=200_000, -10 ** 9..=10 ** 9) = read()
var a: [0..n]-10 ** 9..=10 ** 9 = read()
let mid = n / 2
a.select(mid)
print(sum(max(x - c, 0) for x in a[0..mid]) + sum(x - c for x in a[mid..n]))
