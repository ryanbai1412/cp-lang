// Codeforces 2178 E: interactive halving via binary search on prefix sums to
// isolate the maximum element in O(log^2 n) queries.

fn ask(lo: Int, hi: Int) -> Int:
    print("?", lo + 1, hi)
    flush()
    read()

@!multitest
let n: Int = read()
var (l, r) = (0, n)
var tot = ask(l, r)
while r - l > 1:
    let mid = bsearch_first(l + 1..r - 1, md => 2 * ask(l, md) >= tot) ?? (r - 1)
    tot /= 2
    if r - mid <= mid - l: l = mid
    else: r = mid
print("!", tot)
flush()
