// Codeforces 2201 B: construct a length-2N array containing each value 1..N twice
// with exactly K distinct adjacent pair sums when N <= K <= 2N - 1.
@!multitest
let (n_in, k): (Int, Int) = read()
if not (n_in <= k <= 2 * n_in - 1):
    print("NO")
    return

print("YES")
let n = k - n_in + 1
var out: List<Int> = []
for v in (n + 1..=n_in).rev: out.extend([v, v])
if n == 1:
    out.extend([1, 1])
else:
    out.extend([1, 2])
    for i in 1..n - 1: out.extend([i + 2, i])
    out.extend([n - 1, n])
print(out)
